const fetch = require('node-fetch')
const fs = require('fs')
const ProcessManager = require('pm2-promise')

const {
  Web3,
  nearAPI,
  verifyAccountGently,
  normalizeEthKey
} = require('rainbow-bridge-utils')

// Verdicts
const Ok = 'ok'
const Info = 'info'
const Warn = 'warn'
const Error = 'error'

const Running = 'running'
const Valid = 'valid'
const Deployed = 'deployed'
const Unknown = 'unknown'
const Unreachable = 'unreachable'
const Invalid = 'invalid'
const NotVerified = 'not verified'
const UsingMaster = 'using data from master account'
const RecordNotFound = 'record not found'
const ABINotFound = 'abi not found'
const ContractNotFound = 'contract not found'
const InternalError = 'internal error'

const NoColor = '\x1B[39m'

// TODO please refactor to avoid async/await
const request = async (url) => {
  try {
    const response = await fetch(url)
    const json = await response.json()
    return [
      Ok,
      JSON.stringify(json.version),
      JSON.stringify(json.sync_info.latest_block_height)
    ]
  } catch (err) {
    return [Error, Unreachable, Unknown]
  }
}

class Status {
  constructor (value, verdict = Error, explanation = null) {
    this.value = value
    this.verdict = verdict
    this.explanation = explanation
  }
}

class NearContracts {
  // TODO put it into constructor if possible
  async init (near, { nearMasterAccount, nearClientAccount, nearProverAccount, nearTokenFactoryAccount }) {
    if (!nearMasterAccount) {
      return
    }
    this.client = await this.checkContract(
      near,
      nearMasterAccount,
      nearClientAccount
    )
    this.prover = await this.checkContract(
      near,
      nearMasterAccount,
      nearProverAccount
    )
    this.funToken = await this.checkContract(
      near,
      nearMasterAccount,
      nearTokenFactoryAccount
    )
  }

  async checkContract (near, masterAccount, contractAccount) {
    if (!contractAccount) {
      return new Status(Unknown, Error, ContractNotFound)
    }
    try {
      const nearAccount = new nearAPI.Account(near.connection, masterAccount)
      const contract = new nearAPI.Contract(nearAccount, contractAccount, {
        changeMethods: ['boo'],
        viewMethods: []
      })
      // TODO #270 implement `initialized` method to NEAR contracts
      // TODO #257 check the code deployed if possible
      try {
        await contract.boo()
      } catch (err) {
        if (
          err.message &&
          err.message.indexOf('Contract method is not found') >= 0
        ) {
          return new Status(contract.contractId, Ok, Deployed)
        } else {
          return new Status(Unknown, Info, NotVerified)
        }
      }
    } catch (err) {
      return new Status(Unknown, Error, InternalError)
    }
  }
}

class NearStatus {
  // TODO put it into constructor if possible
  async init ({
    nearNetworkId,
    nearNodeUrl,
    nearMasterAccount,
    nearMasterSk,
    nearClientAccount,
    nearClientSk,
    nearProverAccount,
    nearProverSk
  }) {
    this.networkLocation = nearNetworkId
      ? new Status(nearNetworkId, Info)
      : new Status(Unknown)

    // Init with basic data
    this.masterAccount = nearMasterAccount
      ? new Status(nearMasterAccount, Info, NotVerified)
      : new Status(Unknown)
    this.masterKey = nearMasterSk
      ? new Status(nearMasterSk, Info)
      : new Status(Unknown)
    this.clientAccount = nearClientAccount
      ? new Status(nearClientAccount, Info, NotVerified)
      : new Status(Unknown, Info, UsingMaster)
    this.clientKey = nearClientSk
      ? new Status(nearClientSk, Info)
      : new Status(Unknown, Info, UsingMaster)
    this.proverAccount = nearProverAccount
      ? new Status(nearProverAccount, Info, NotVerified)
      : new Status(Unknown, Info, UsingMaster)
    this.proverKey = nearProverSk
      ? new Status(nearProverSk, Info)
      : new Status(Unknown, Info, UsingMaster)

    if (nearNodeUrl) {
      const [verdict, explanation, lastBlock] = await request(nearNodeUrl + '/status')
      this.networkConnection = new Status(nearNodeUrl, verdict, explanation)
      this.networkLastBlock = new Status(lastBlock, verdict)
      if (verdict === Ok) {
        // Connected to NEAR node
        if (nearMasterAccount && nearMasterSk) {
          const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
          await keyStore.setKey(
            nearNetworkId,
            nearMasterAccount,
            nearAPI.KeyPair.fromString(nearMasterSk)
          )
          if (nearClientAccount && nearClientSk) {
            await keyStore.setKey(
              nearNetworkId,
              nearClientAccount,
              nearAPI.KeyPair.fromString(nearClientSk)
            )
          }
          if (nearProverAccount && nearProverSk) {
            await keyStore.setKey(
              nearNetworkId,
              nearProverAccount,
              nearAPI.KeyPair.fromString(nearProverSk)
            )
          }
          const near = await nearAPI.connect({
            nodeUrl: nearNodeUrl,
            nearNetworkId,
            masterAccount: nearMasterAccount,
            keyStore
          })
          this.masterAccount = (await verifyAccountGently(near, nearMasterAccount))
            ? new Status(nearMasterAccount, Ok, Valid)
            : new Status(nearMasterAccount, Error, Invalid)
          this.masterKey.verdict = this.masterAccount.verdict
          if (nearClientAccount && nearClientSk) {
            this.nearClientAccount = (await verifyAccountGently(
              near,
              nearClientAccount
            ))
              ? new Status(nearClientAccount, Ok, Valid)
              : new Status(nearClientAccount, Error, Invalid)
            this.clientKey.verdict = this.clientAccount.verdict
          }
          if (nearProverAccount && nearProverSk) {
            this.nearProverAccount = (await verifyAccountGently(
              near,
              nearProverAccount
            ))
              ? new Status(nearProverAccount, Ok, Valid)
              : new Status(nearProverAccount, Error, Invalid)
            this.proverKey.verdict = this.proverAccount.verdict
          }

          const nearContracts = new NearContracts()
          await nearContracts.init(near)
          this.contracts = nearContracts
        }
      }
    } else {
      this.networkConnection = new Status(Unknown)
      this.networkLastBlock = new Status(Unknown)
    }
  }
}

class EthContracts {
  // TODO put it into constructor if possible
  async init (
    web3,
    {
      ethEd25519AbiPath,
      ethEd25519Address,
      ethErc20AbiPath,
      ethErc20Address,
      ethLockerAbiPath,
      ethLockerAddress,
      ethClientAbiPath,
      ethClientAddress,
      ethProverAbiPath,
      ethProverAbiAddress
    }) {
    this.ed25519 = await this.checkContract(web3, ethEd25519AbiPath, ethEd25519Address)
    this.erc20 = await this.checkContract(web3, ethErc20AbiPath, ethErc20Address)
    this.locker = await this.checkContract(web3, ethLockerAbiPath, ethLockerAddress)
    this.client = await this.checkContract(web3, ethClientAbiPath, ethClientAddress)
    this.prover = await this.checkContract(web3, ethProverAbiPath, ethProverAbiAddress)
  }

  async checkContract (web3, abiPath, address) {
    if (!abiPath) {
      return new Status(Unknown, Error, ABINotFound)
    }
    if (!address) {
      return new Status(Unknown, Error, ContractNotFound)
    }
    try {
      const abi = JSON.parse(fs.readFileSync(abiPath))
      const contract = await new web3.eth.Contract(abi, address)
      if (contract.options.address === address) {
        // TODO #257 check deployed code equality if possible
        return new Status(contract.options.address, Ok, Deployed)
      } else {
        return new Status(address, Error, Invalid)
      }
    } catch (err) {
      return new Status(Unknown, Error, InternalError)
    }
  }
}

class EthStatus {
  // TODO put it into constructor if possible
  async init ({ ethNodeUrl, ethMasterSk }) {
    // Init with basic data
    this.masterAccount = new Status(Unknown, Info, NotVerified)
    this.masterKey = ethMasterSk
      ? new Status(normalizeEthKey(ethMasterSk), Info)
      : new Status(Unknown)

    if (ethNodeUrl) {
      try {
        const web3 = await new Web3(ethNodeUrl)
        const chain = web3.eth.defaultChain ? web3.eth.defaultChain : 'local'
        this.networkLocation = new Status(chain, Info)
        try {
          const version = 'version ' + (await web3.eth.getProtocolVersion())
          const lastBlock = await web3.eth.getBlockNumber()
          this.networkConnection = new Status(ethNodeUrl, Ok, version)
          this.networkLastBlock = new Status(lastBlock, Ok)
        } catch (err) {
          this.networkConnection = new Status(ethNodeUrl, Error, Unreachable)
          this.networkLastBlock = new Status(Unknown)
        }
        try {
          const masterAccount = web3.eth.accounts.privateKeyToAccount(
            normalizeEthKey(ethMasterSk)
          )
          const accounts = await web3.eth.getAccounts()
          if (accounts.includes(masterAccount.address)) {
            this.masterAccount = new Status(masterAccount.address, Ok, Valid)
            this.masterKey.verdict = Ok
          } else {
            this.masterAccount = new Status(
              masterAccount.address,
              Error,
              Invalid
            )
            this.masterKey.verdict = Error
          }
        } catch (err) { }

        const ethContracts = new EthContracts()
        await ethContracts.init(web3)
        this.contracts = ethContracts

        try {
          web3.currentProvider.connection.close()
        } catch (err) { }
      } catch (err) {
        this.networkLocation = new Status(Unknown)
        this.networkConnection = new Status(ethNodeUrl, Error, Unreachable)
        this.networkLastBlock = new Status(Unknown)
      }
    } else {
      this.networkLocation = new Status(Unknown)
      this.networkConnection = new Status(Unknown)
      this.networkLastBlock = new Status(Unknown)
    }
  }
}

class ServicesStatus {
  async init () {
    this.eth2nearRelay = await this.running('eth2near-relay')
    this.near2ethRelay = await this.running('near2eth-relay')
    this.watchdog = await this.running('bridge-watchdog')
  }

  processArgs (args) {
    const res = []
    for (let i = 0; i + 1 < args.length; i++) {
      if (args[i].startsWith('--')) {
        res.push(args[i].substring(2, args[i].length) + '=' + args[i + 1])
      }
    }
    return res.join(', ')
  }

  async running (serviceName) {
    let status
    try {
      const process = await ProcessManager.describe(serviceName)
      if (process.length) {
        status = new Status(
          Running,
          Ok,
          this.processArgs(process[0].pm2_env.args)
        )
      } else {
        status = new Status(Unknown, Error, Unreachable)
      }
    } catch (err) {
      status = new Status(Unknown, Error, InternalError)
    }
    return status
  }
}

// TODO put it into StatusCommand class if possible
function printHeader (text) {
  console.log('\x1B[33m' + text + NoColor)
}

function printLine (field, status = null) {
  if (!status) {
    status = new Status(Unknown, Error, RecordNotFound)
  }
  let color = '\x1B[35m'
  switch (status.verdict) {
    case Ok:
      color = '\x1B[32m'
      break
    case Info:
      color = '\x1B[39m'
      break
    case Warn:
      color = '\x1B[33m'
      break
    case Error:
      color = '\x1B[35m'
      break
  }
  const explanation = status.explanation ? '(' + status.explanation + ')' : ''
  let line = field + ':'
  while (line.length < 50) {
    line = line + ' '
  }
  console.log(line, color + status.value, explanation + NoColor)
}

function printFooter () {
  console.log()
}

class StatusCommand {
  static async execute ({
    nearNetworkId,
    nearNodeUrl,
    nearMasterAccount,
    nearMasterSk,
    nearClientAccount,
    nearClientSk,
    nearProverAccount,
    nearProverSk,
    ethNodeUrl,
    ethMasterSk
  }) {
    const consoleError = console.error
    // A cool hack to avoid annoying Web3 printing to stderr
    console.error = function () { }

    const nearStatus = new NearStatus()
    await nearStatus.init({
      nearNetworkId,
      nearNodeUrl,
      nearMasterAccount,
      nearMasterSk,
      nearClientAccount,
      nearClientSk,
      nearProverAccount,
      nearProverSk
    })

    const ethStatus = new EthStatus()
    await ethStatus.init({
      ethNodeUrl,
      ethMasterSk
    })

    const servicesStatus = new ServicesStatus()
    await servicesStatus.init()

    // Return console.error back
    console.error = consoleError

    printHeader('NEAR node status')
    printLine('Location', nearStatus.networkLocation)
    printLine('Connection', nearStatus.networkConnection)
    printLine('Latest block height', nearStatus.networkLastBlock)
    printLine('Bridge master account', nearStatus.masterAccount)
    printLine('Master account secret key', nearStatus.masterKey)
    printLine(
      'Account used for Client contract deployment',
      nearStatus.clientAccount
    )
    printLine('Client account secret key', nearStatus.clientKey)
    printLine(
      'Account used for Prover contract deployment',
      nearStatus.proverAccount
    )
    printLine('Prover account secret key', nearStatus.proverKey)
    printFooter()

    printHeader('NEAR contracts status')
    if (nearStatus.contracts) {
      printLine('fungible token', nearStatus.contracts.funToken)
      printLine('client', nearStatus.contracts.client)
      printLine('prover', nearStatus.contracts.prover)
    } else {
      printLine('Contracts')
    }
    printFooter()

    printHeader('ETH node status')
    printLine('Location', ethStatus.networkLocation)
    printLine('Connection', ethStatus.networkConnection)
    printLine('Latest block height', ethStatus.networkLastBlock)
    printLine('Bridge master account', ethStatus.masterAccount)
    printLine('Master account secret key', ethStatus.masterKey)
    printFooter()

    printHeader('ETH contracts status')
    if (ethStatus.contracts) {
      printLine('ed25519', ethStatus.contracts.ed25519)
      printLine('erc20', ethStatus.contracts.erc20)
      printLine('locker', ethStatus.contracts.locker)
      printLine('client', ethStatus.contracts.client)
      printLine('prover', ethStatus.contracts.prover)
    } else {
      printLine('Contracts')
    }
    printFooter()

    printHeader('Services')
    printLine('ETH-2-NEAR relay', servicesStatus.eth2nearRelay)
    printLine('NEAR-2-ETH relay', servicesStatus.near2ethRelay)
    printLine('Bridge Watchdog', servicesStatus.watchdog)

    ProcessManager.disconnect()
  }
}

exports.StatusCommand = StatusCommand

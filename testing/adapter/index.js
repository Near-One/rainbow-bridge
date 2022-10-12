const fs = require('fs')
const bs58 = require('bs58')
const { toBuffer } = require('eth-util-lite')
const {
  nearAPI,
  RobustWeb3,
  remove0x,
  normalizeEthKey,
  JSONreplacer,
  borshifyOutcomeProof
} = require('rainbow-bridge-utils')
const { exit } = require('process')

// === JS Adapter function agreement ===
//
// All functions return void type because there is no external usage from JS expected.
// Execution results should be printed to stdout and they will be parsed later in pytest.
// To distinguish timeout from successful function execution, all changing functions must return 'OK' in the first line of output.
//
// Unfortunately, in case of reverting txs web3 not closes the connection even after calling `close()`.
// That's why all functions must finish with exit(0) to make sure not getting stuck by any reason.
//
// Please follow the agreement when adding new functions. XOXO

// Change
async function mintErc20 ({ ethAccountAddress, amount, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      remove0x(ethErc20Address)
    )
    await ethContract.methods.mint(ethAccountAddress, Number(amount)).send({ from: ethAccountAddress, gas: 5000000 })
    console.log('OK')
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// View
function getAddressBySecretKey ({ ethSecretKey, ethNodeUrl }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethAccount = web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethSecretKey)
    )
    console.log(ethAccount.address)
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// View
async function getErc20Balance ({ ethAccountAddress, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      remove0x(ethErc20Address)
    )
    const balance = await ethContract.methods.balanceOf(remove0x(ethAccountAddress)).call()
    console.log(balance)
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// Change
async function ethToNearApprove ({ ethAccountAddress, amount, ethNodeUrl, ethErc20Address, ethErc20AbiPath, ethLockerAddress }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      remove0x(ethErc20Address)
    )
    await robustWeb3.callContract(
      ethContract,
      'approve',
      [ethLockerAddress, Number(amount)],
      {
        from: ethAccountAddress,
        gas: 5000000
      }
    )
    console.log('OK')
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// Change
async function ethToNearLock ({ ethAccountAddress, amount, nearAccountName, ethNodeUrl, ethErc20Address, ethLockerAbiPath, ethLockerAddress }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethTokenLockerContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethLockerAbiPath)),
      ethLockerAddress
    )
    const transaction = await robustWeb3.callContract(
      ethTokenLockerContract,
      'lockToken',
      [ethErc20Address, Number(amount), nearAccountName],
      {
        from: ethAccountAddress,
        gas: 5000000
      }
    )
    const lockedEvent = transaction.events.Locked
    console.log('OK')
    console.log(JSON.stringify(lockedEvent, JSONreplacer))
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// View
async function getClientBlockHeightHash ({
  ethNodeUrl,
  ethClientAddress,
  ethClientArtifactPath,
  ethMasterAccount
}) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const clientContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethClientArtifactPath)).abi,
      ethClientAddress,
      {
        from: ethMasterAccount,
        handleRevert: true
      }
    )
    const clientState = await clientContract.methods.bridgeState().call()
    const clientBlockHash = bs58.encode(
      toBuffer(
        await clientContract.methods.blockHashes(clientState.currentHeight).call()
      )
    )
    console.log(clientState.currentHeight, clientBlockHash)
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// Change
async function nearToEthUnlock ({
  blockHeight,
  proof,
  ethNodeUrl,
  ethMasterSk,
  ethLockerAddress,
  ethLockerAbiPath,
  ethGasMultiplier
}) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethMasterAccount = web3.eth.accounts.privateKeyToAccount(normalizeEthKey(ethMasterSk)).address
    const ethTokenLockerContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethLockerAbiPath)),
      ethLockerAddress
    )
    const borshProof = borshifyOutcomeProof(JSON.parse(proof))
    blockHeight = Number(blockHeight)
    await robustWeb3.callContract(
      ethTokenLockerContract,
      'unlockToken',
      [borshProof, blockHeight],
      {
        from: ethMasterAccount,
        gas: 5000000,
        handleRevert: true,
        gasPrice: Number(await robustWeb3.web3.eth.getGasPrice() * ethGasMultiplier)
      }
    )
    console.log('OK')
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
  exit(0)
}

// View
async function getBridgeOnNearBalance ({
  nearReceiverAccount,
  nearErc20Account,
  nearNetworkId,
  nearNodeUrl
}) {
  try {
    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    const near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      masterAccount: nearReceiverAccount,
      keyStore
    })

    const nearAccount = new nearAPI.Account(
      near.connection,
      nearReceiverAccount
    )

    const nearTokenContract = new nearAPI.Contract(
      nearAccount,
      nearErc20Account,
      {
        changeMethods: [],
        viewMethods: ['ft_balance_of']
      }
    )

    const balance = await nearTokenContract.ft_balance_of({
      account_id: nearReceiverAccount
    })
    console.log(
      `[Rainbow-Bridge on Near] Balance of ${nearReceiverAccount} is ${balance}`
    )
  } catch (error) {
    console.log('Failed', error.toString())
  }
  exit(0)
}

exports.ethToNearApprove = ethToNearApprove
exports.ethToNearLock = ethToNearLock
exports.nearToEthUnlock = nearToEthUnlock
exports.mintErc20 = mintErc20
exports.getErc20Balance = getErc20Balance
exports.getAddressBySecretKey = getAddressBySecretKey
exports.getClientBlockHeightHash = getClientBlockHeightHash
exports.getBridgeOnNearBalance = getBridgeOnNearBalance

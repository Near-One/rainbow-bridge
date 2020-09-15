const utils = require('ethereumjs-util')
const BN = require('bn.js')
const fs = require('fs')
const nearlib = require('near-api-js')
const {
  EthProofExtractor,
  receiptFromWeb3,
  logFromWeb3,
} = require('../eth-proof-extractor')
const { verifyAccount } = require('../rainbow/helpers')
const { NearMintableToken } = require('../near-mintable-token')
const { RainbowConfig } = require('../config')
const { EthOnNearClientContract } = require('../eth-on-near-client')
const { sleep, RobustWeb3, normalizeEthKey } = require('../rainbow/robust')

let initialCmd

class TransferETHERC20ToNear {
  static showRetryAndExit() {
    console.log('Retry with command:')
    console.log(initialCmd)
    process.exit(1)
  }

  static async approve({
    robustWeb3,
    ethERC20Contract,
    amount,
    ethSenderAccount,
  }) {
    // Approve tokens for transfer.
    const lockerAddress = RainbowConfig.getParam('eth-locker-address')
    try {
      console.log(
        `Approving token transfer to ${lockerAddress} ${Number(amount)}.`
      )
      await robustWeb3.callContract(
        ethERC20Contract,
        'approve',
        [lockerAddress, Number(amount)],
        {
          from: ethSenderAccount,
          gas: 5000000,
        }
      )
      console.log('Approved token transfer.')
      TransferETHERC20ToNear.recordTransferLog({ finished: 'approve' })
    } catch (txRevertMessage) {
      console.log('Failed to approve.')
      console.log(txRevertMessage.toString())
      TransferETHERC20ToNear.showRetryAndExit()
    }
  }

  static async lock({
    robustWeb3,
    ethTokenLockerContract,
    amount,
    nearReceiverAccount,
    ethSenderAccount,
  }) {
    try {
      console.log(
        `Transferring tokens from the ERC20 account to the token locker account ${Number(
          amount
        )}.`
      )
      const transaction = await robustWeb3.callContract(
        ethTokenLockerContract,
        'lockToken',
        [
          RainbowConfig.getParam('eth-erc20-address'),
          Number(amount),
          nearReceiverAccount,
        ],
        {
          from: ethSenderAccount,
          gas: 5000000,
        }
      )
      console.log(transaction)
      const lockedEvent = transaction.events.Locked
      console.log('Success tranfer to locker')
      TransferETHERC20ToNear.recordTransferLog({
        finished: 'lock',
        lockedEvent,
      })
    } catch (txRevertMessage) {
      console.log('Failed to lock account.')
      console.log(txRevertMessage.toString())
      TransferETHERC20ToNear.showRetryAndExit()
    }
  }

  static async findProof({ extractor, lockedEvent, web3 }) {
    const receipt = await extractor.extractReceipt(lockedEvent.transactionHash)
    const block = await extractor.extractBlock(receipt.blockNumber)
    const tree = await extractor.buildTrie(block)
    const proof = await extractor.extractProof(
      web3,
      block,
      tree,
      receipt.transactionIndex
    )
    let txLogIndex = -1

    let logFound = false
    let log
    for (let receiptLog of receipt.logs) {
      txLogIndex++
      const blockLogIndex = receiptLog.logIndex
      if (blockLogIndex === lockedEvent.logIndex) {
        logFound = true
        log = receiptLog
        break
      }
    }
    if (logFound) {
      TransferETHERC20ToNear.recordTransferLog({
        finished: 'find-proof',
        proof,
        log,
        txLogIndex,
        receipt,
        lockedEvent,
        block,
      })
    } else {
      console.log(`Failed to find log for event ${lockedEvent}`)
      TransferETHERC20ToNear.showRetryAndExit()
    }
  }

  static async waitBlockSafe({
    log,
    proof,
    receipt,
    txLogIndex,
    lockedEvent,
    block,
    ethOnNearClientContract,
  }) {
    const log_entry_data = logFromWeb3(log).serialize()
    const receipt_index = proof.txIndex
    const receipt_data = receiptFromWeb3(receipt).serialize()
    const header_data = proof.header_rlp
    const _proof = []
    for (const node of proof.receiptProof) {
      _proof.push(utils.rlp.encode(node))
    }

    const proof_locker = {
      log_index: txLogIndex,
      log_entry_data: log_entry_data,
      receipt_index: receipt_index,
      receipt_data: receipt_data,
      header_data: header_data,
      proof: _proof,
    }

    const new_owner_id = lockedEvent.returnValues.accountId
    const amount = lockedEvent.returnValues.amount
    console.log(
      `Transferring ${amount} tokens from ${lockedEvent.returnValues.token} ERC20. From ${lockedEvent.returnValues.sender} sender to ${new_owner_id} recipient`
    )

    const blockNumber = block.number
    // Wait until client accepts this block number.
    while (true) {
      // @ts-ignore
      const last_block_number = (
        await ethOnNearClientContract.last_block_number()
      ).toNumber()
      const is_safe = await ethOnNearClientContract.block_hash_safe(blockNumber)
      if (!is_safe) {
        const delay = 10
        console.log(
          `Near Client contract is currently at block ${last_block_number}. Waiting for block ${blockNumber} to be confirmed. Sleeping for ${delay} sec.`
        )
        await sleep(delay * 1000)
      } else {
        break
      }
    }
    TransferETHERC20ToNear.recordTransferLog({
      finished: 'block-safe',
      proof_locker,
      new_owner_id,
    })
  }

  static async deposit({
    proof_locker,
    nearFactoryContract,
    nearFactoryContractBorsh,
    nearTokenContract,
    new_owner_id,
  }) {
    // @ts-ignore
    const old_balance = await nearTokenContract.get_balance({
      owner_id: new_owner_id,
    })
    console.log(
      `Balance of ${new_owner_id} before the transfer is ${old_balance}`
    )
    // @ts-ignore
    try {
      await nearFactoryContractBorsh.deposit(
        proof_locker,
        new BN('300000000000000'),
        // We need to attach tokens because minting increases the contract state, by <600 bytes, which
        // requires an additional 0.06 NEAR to be deposited to the account for state staking.
        // Note technically 0.0537 NEAR should be enough, but we round it up to stay on the safe side.
        new BN('100000000000000000000').mul(new BN('600'))
      )
      console.log('Transferred')
    } catch (e) {
      console.log('Deposit failed with error:')
      console.log(e)
      TransferETHERC20ToNear.showRetryAndExit()
    }

    // @ts-ignore
    const new_balance = await nearTokenContract.get_balance({
      owner_id: new_owner_id,
    })
    console.log(
      `Balance of ${new_owner_id} after the transfer is ${new_balance}`
    )
    TransferETHERC20ToNear.deleteTransferLog()
  }

  static recordTransferLog(obj) {
    fs.writeFileSync('transfer-eth-erc20-to-near.log.json', JSON.stringify(obj))
  }

  static parseBuffer(obj) {
    for (let i in obj) {
      if (obj[i] && obj[i].type === 'Buffer') {
        obj[i] = Buffer.from(obj[i].data)
      } else if (obj[i] && typeof obj[i] === 'object') {
        obj[i] = TransferETHERC20ToNear.parseBuffer(obj[i])
      }
    }
    return obj
  }

  static loadTransferLog() {
    try {
      let log =
        JSON.parse(
          fs.readFileSync('transfer-eth-erc20-to-near.log.json').toString()
        ) || {}
      return TransferETHERC20ToNear.parseBuffer(log)
    } catch (e) {
      return {}
    }
  }

  static deleteTransferLog() {
    try {
      fs.unlinkSync('transfer-eth-erc20-to-near.log.json')
    } catch (e) {
      console.log('Warning: failed to remove tranfer log')
    }
  }

  static async execute(command) {
    initialCmd = command.parent.rawArgs.join(' ')
    let transferLog = TransferETHERC20ToNear.loadTransferLog()
    const amount = command.amount
    const ethSenderSk = command.ethSenderSk
    const nearReceiverAccount = command.nearReceiverAccount

    // @ts-ignore
    let robustWeb3 = new RobustWeb3(RainbowConfig.getParam('eth-node-url'))
    let web3 = robustWeb3.web3
    let ethSenderAccount = web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethSenderSk)
    )
    web3.eth.accounts.wallet.add(ethSenderAccount)
    web3.eth.defaultAccount = ethSenderAccount.address
    ethSenderAccount = ethSenderAccount.address

    const ethERC20Contract = new web3.eth.Contract(
      // @ts-ignore
      JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-erc20-abi-path'))),
      RainbowConfig.getParam('eth-erc20-address')
    )

    const nearMasterAccountId = RainbowConfig.getParam('near-master-account')
    console.log(nearMasterAccountId)
    // @ts-ignore
    const keyStore = new nearlib.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      RainbowConfig.getParam('near-network-id'),
      nearMasterAccountId,
      nearlib.KeyPair.fromString(RainbowConfig.getParam('near-master-sk'))
    )
    const near = await nearlib.connect({
      nodeUrl: RainbowConfig.getParam('near-node-url'),
      networkId: RainbowConfig.getParam('near-network-id'),
      masterAccount: nearMasterAccountId,
      deps: { keyStore: keyStore },
    })
    const nearMasterAccount = new nearlib.Account(
      near.connection,
      nearMasterAccountId
    )
    await verifyAccount(near, nearMasterAccountId)

    const nearFactoryContract = new nearlib.Contract(
      nearMasterAccount,
      RainbowConfig.getParam('near-fun-token-account'),
      {
        changeMethods: ['deposit'],
        viewMethods: [],
      }
    )
    const nearTokenContract = new nearlib.Contract(
      nearMasterAccount,
      RainbowConfig.getParam('near-erc20-account'),
      {
        changeMethods: [],
        viewMethods: ['get_balance'],
      }
    )
    const nearFactoryContractBorsh = new NearMintableToken(
      nearMasterAccount,
      RainbowConfig.getParam('near-fun-token-account')
    )
    await nearFactoryContractBorsh.accessKeyInit()

    const extractor = new EthProofExtractor()
    extractor.initialize(RainbowConfig.getParam('eth-node-url'))

    const ethTokenLockerContract = new web3.eth.Contract(
      // @ts-ignore
      JSON.parse(
        fs.readFileSync(RainbowConfig.getParam('eth-locker-abi-path'))
      ),
      RainbowConfig.getParam('eth-locker-address')
    )

    const clientAccount = RainbowConfig.getParam('near-client-account')
    const ethOnNearClientContract = new EthOnNearClientContract(
      nearMasterAccount,
      clientAccount
    )

    if (transferLog.finished === undefined) {
      // TODO fix before using
      // Mint tokens first???
      /*await ethERC20Contract.methods
        .mint(ethSenderAccount, Number(amount))
        .send({ from: ethSenderAccount, gas: 5000000 })*/
      console.log(
        'Balance: ',
        await ethERC20Contract.methods.balanceOf(ethSenderAccount).call()
      )
      await TransferETHERC20ToNear.approve({
        robustWeb3,
        ethERC20Contract,
        amount,
        ethSenderAccount,
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'approve') {
      await TransferETHERC20ToNear.lock({
        robustWeb3,
        ethTokenLockerContract,
        amount,
        nearReceiverAccount,
        ethSenderAccount,
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'lock') {
      await TransferETHERC20ToNear.findProof({
        extractor,
        lockedEvent: transferLog.lockedEvent,
        web3,
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'find-proof') {
      await TransferETHERC20ToNear.waitBlockSafe({
        ethOnNearClientContract,
        ...transferLog,
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'block-safe') {
      await TransferETHERC20ToNear.deposit({
        nearFactoryContract,
        nearFactoryContractBorsh,
        nearTokenContract,
        ...transferLog,
      })
    }

    try {
      // Only WebSocket provider can close.
      web3.currentProvider.connection.close()
    } catch (e) {}
    process.exit(0)
  }
}

exports.TransferETHERC20ToNear = TransferETHERC20ToNear

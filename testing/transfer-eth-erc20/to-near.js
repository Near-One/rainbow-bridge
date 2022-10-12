const utils = require('ethereumjs-util')
const BN = require('bn.js')
const fs = require('fs')
const crypto = require('crypto')
const {
  nearAPI,
  sleep,
  RobustWeb3,
  normalizeEthKey,
  verifyAccount
} = require('rainbow-bridge-utils')
const {
  EthProofExtractor,
  EthOnNearClientContract,
  receiptFromWeb3,
  logFromWeb3
} = require('rainbow-bridge-eth2near-block-relay')
const { NearMintableToken } = require('./near-mintable-token')

let initialCmd
const txLogFilename = Date.now() + '-' + crypto.randomBytes(4).toString('hex') + '-transfer-eth-erc20-from-near.log.json'

class TransferETHERC20ToNear {
  static showRetryAndExit () {
    console.log('Retry with command:')
    console.log(initialCmd)
    process.exit(1)
  }

  static async approve ({
    robustWeb3,
    ethERC20Contract,
    amount,
    ethSenderAccount,
    ethLockerAddress
  }) {
    // Approve tokens for transfer.
    try {
      console.log(
        `Approving token transfer to ${ethLockerAddress} ${(new BN(amount)).toString()}.`
      )
      await robustWeb3.callContract(
        ethERC20Contract,
        'approve',
        [ethLockerAddress, new BN(amount)],
        {
          from: ethSenderAccount,
          gas: 5000000
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

  static async lock ({
    robustWeb3,
    amount,
    ethTokenLockerContract,
    ethErc20Address,
    ethSenderAccount,
    nearReceiverAccount
  }) {
    try {
      console.log(
        `Transferring tokens from the ERC20 account to the token locker account ${(new BN(
          amount
        )).toString()}.`
      )
      const transaction = await robustWeb3.callContract(
        ethTokenLockerContract,
        'lockToken',
        [ethErc20Address, new BN(amount), nearReceiverAccount],
        {
          from: ethSenderAccount,
          gas: 5000000
        }
      )
      console.log(transaction)
      const lockedEvent = transaction.events.Locked
      console.log('Success tranfer to locker')
      TransferETHERC20ToNear.recordTransferLog({
        finished: 'lock',
        lockedEvent
      })
    } catch (txRevertMessage) {
      console.log('Failed to lock account.')
      console.log(txRevertMessage.toString())
      TransferETHERC20ToNear.showRetryAndExit()
    }
  }

  static async findProof ({ extractor, lockedEvent, web3 }) {
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
    for (const receiptLog of receipt.logs) {
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
        block
      })
    } else {
      console.log(`Failed to find log for event ${lockedEvent}`)
      TransferETHERC20ToNear.showRetryAndExit()
    }
  }

  static async waitBlockSafe ({
    log,
    proof,
    receipt,
    txLogIndex,
    lockedEvent,
    block,
    ethOnNearClientContract
  }) {
    const logEntryData = logFromWeb3(log).serialize()
    const receiptIndex = proof.txIndex
    const receiptData = receiptFromWeb3(receipt).serialize()
    const headerData = proof.header_rlp
    const _proof = []
    for (const node of proof.receiptProof) {
      _proof.push(utils.rlp.encode(node))
    }

    const proofLocker = {
      log_index: txLogIndex,
      log_entry_data: logEntryData,
      receipt_index: receiptIndex,
      receipt_data: receiptData,
      header_data: headerData,
      proof: _proof
    }

    const newOwnerId = lockedEvent.returnValues.accountId
    const amount = lockedEvent.returnValues.amount
    console.log(
      `Transferring ${amount} tokens from ${lockedEvent.returnValues.token} ERC20. From ${lockedEvent.returnValues.sender} sender to ${newOwnerId} recipient`
    )

    const blockNumber = block.number
    // Wait until client accepts this block number.
    while (true) {
      const lastBlockNumber = (
        await ethOnNearClientContract.last_block_number()
      ).toNumber()
      const isSafe = await ethOnNearClientContract.block_hash_safe(blockNumber)
      if (!isSafe) {
        const delay = 10
        console.log(
          `Near Client contract is currently at block ${lastBlockNumber}. Waiting for block ${blockNumber} to be confirmed. Sleeping for ${delay} sec.`
        )
        await sleep(delay * 1000)
      } else {
        break
      }
    }
    TransferETHERC20ToNear.recordTransferLog({
      finished: 'block-safe',
      proofLocker,
      newOwnerId
    })
  }

  static async deposit ({
    proofLocker,
    nearFactoryContractBorsh,
    nearTokenContract,
    newOwnerId
  }) {
    const oldBalance = await nearTokenContract.ft_balance_of({
      account_id: newOwnerId
    })
    console.log(
      `Balance of ${newOwnerId} before the transfer is ${oldBalance}`
    )
    try {
      await nearFactoryContractBorsh.deposit(
        proofLocker,
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

    const newBalance = await nearTokenContract.ft_balance_of({
      account_id: newOwnerId
    })
    console.log(
      `Balance of ${newOwnerId} after the transfer is ${newBalance}`
    )
  }

  static recordTransferLog (obj) {
    fs.writeFileSync(txLogFilename, JSON.stringify(obj))
  }

  static parseBuffer (obj) {
    for (const i in obj) {
      if (obj[i] && obj[i].type === 'Buffer') {
        obj[i] = Buffer.from(obj[i].data)
      } else if (obj[i] && typeof obj[i] === 'object') {
        obj[i] = TransferETHERC20ToNear.parseBuffer(obj[i])
      }
    }
    return obj
  }

  static loadTransferLog () {
    try {
      const log =
        JSON.parse(
          fs.readFileSync(txLogFilename).toString()
        ) || {}
      console.log('Transfer log found', log)
      return TransferETHERC20ToNear.parseBuffer(log)
    } catch (e) {
      console.log("Coudn't find transfer log at ", txLogFilename)
      return {}
    }
  }

  static async execute ({
    parent: { args },
    amount,
    ethSenderSk,
    nearReceiverAccount,
    nearMasterAccount: nearMasterAccountId,
    nearNetworkId,
    nearNodeUrl,
    nearMasterSk,
    nearTokenFactoryAccount,
    nearClientAccount,
    nearErc20Account,
    ethNodeUrl,
    ethErc20AbiPath,
    ethErc20Address,
    ethLockerAbiPath,
    ethLockerAddress
  }) {
    initialCmd = args.join(' ')
    let transferLog = TransferETHERC20ToNear.loadTransferLog()
    console.log(`Using ETH address ${ethErc20Address}`)

    const robustWeb3 = new RobustWeb3(ethNodeUrl)
    const web3 = robustWeb3.web3
    let ethSenderAccount = web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethSenderSk)
    )
    web3.eth.accounts.wallet.add(ethSenderAccount)
    web3.eth.defaultAccount = ethSenderAccount.address
    ethSenderAccount = ethSenderAccount.address

    const ethERC20Contract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      ethErc20Address
    )

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      nearNetworkId,
      nearMasterAccountId,
      nearAPI.KeyPair.fromString(nearMasterSk)
    )
    const near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      masterAccount: nearMasterAccountId,
      keyStore
    })
    const nearMasterAccount = new nearAPI.Account(
      near.connection,
      nearMasterAccountId
    )
    await verifyAccount(near, nearMasterAccountId)

    const nearTokenContract = new nearAPI.Contract(
      nearMasterAccount,
      nearErc20Account,
      {
        changeMethods: [],
        viewMethods: ['ft_balance_of']
      }
    )
    const nearFactoryContractBorsh = new NearMintableToken(
      nearMasterAccount,
      nearTokenFactoryAccount
    )
    await nearFactoryContractBorsh.accessKeyInit()

    const extractor = new EthProofExtractor()
    extractor.initialize(ethNodeUrl)

    const ethTokenLockerContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethLockerAbiPath)),
      ethLockerAddress
    )

    const ethOnNearClientContract = new EthOnNearClientContract(
      nearMasterAccount,
      nearClientAccount
    )

    if (transferLog.finished === undefined) {
      // TODO fix before using
      // Mint tokens first???
      /* await ethERC20Contract.methods
        .mint(ethSenderAccount, Number(amount))
        .send({ from: ethSenderAccount, gas: 5000000 }) */
      console.log(
        'Balance: ',
        await ethERC20Contract.methods.balanceOf(ethSenderAccount).call()
      )
      await TransferETHERC20ToNear.approve({
        robustWeb3,
        ethERC20Contract,
        amount,
        ethSenderAccount,
        ethLockerAddress
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'approve') {
      await TransferETHERC20ToNear.lock({
        robustWeb3,
        ethTokenLockerContract,
        ethErc20Address,
        amount,
        nearReceiverAccount,
        ethSenderAccount
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'lock') {
      await TransferETHERC20ToNear.findProof({
        extractor,
        lockedEvent: transferLog.lockedEvent,
        web3
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'find-proof') {
      await TransferETHERC20ToNear.waitBlockSafe({
        ethOnNearClientContract,
        ...transferLog
      })
      transferLog = TransferETHERC20ToNear.loadTransferLog()
    }
    if (transferLog.finished === 'block-safe') {
      await TransferETHERC20ToNear.deposit({
        nearFactoryContractBorsh,
        nearTokenContract,
        ...transferLog
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

const BN = require('bn.js')
const fs = require('fs')
const bs58 = require('bs58')
const crypto = require('crypto')
const { toBuffer } = require('eth-util-lite')
const {
  nearAPI,
  verifyAccount,
  borshifyOutcomeProof,
  sleep,
  RobustWeb3,
  remove0x,
  normalizeEthKey,
  backoff,
  nearJsonContractFunctionCall
} = require('rainbow-bridge-utils')
const ethers = require('ethers')
const { NearMintableToken } = require('./near-mintable-token')

let initialCmd
const txLogFilename = Date.now() + '-' + crypto.randomBytes(8).toString('hex') + '-transfer-eth-erc20-from-near.log.json'

class TransferEthERC20FromNear {
  static showRetryAndExit () {
    console.log('Retry with command:')
    console.log(initialCmd)
    process.exit(1)
  }

  static parseBuffer (obj) {
    for (const i in obj) {
      if (obj[i] && obj[i].type === 'Buffer') {
        obj[i] = Buffer.from(obj[i].data)
      } else if (obj[i] && typeof obj[i] === 'object') {
        obj[i] = TransferEthERC20FromNear.parseBuffer(obj[i])
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
      return TransferEthERC20FromNear.parseBuffer(log)
    } catch (e) {
      console.log("Coudn't find transfer log at ", txLogFilename)
      return {}
    }
  }

  static recordTransferLog (obj) {
    fs.writeFileSync(
      txLogFilename,
      JSON.stringify(obj)
    )
  }

  static async withdraw ({
    nearTokenContract,
    nearSenderAccountId,
    nearErc20Account,
    amount,
    ethReceiverAddress,
    nearSenderAccount
  }) {
    // Withdraw the token on Near side.
    try {
      const oldBalance = await backoff(10, () =>
        nearTokenContract.ft_balance_of({
          account_id: nearSenderAccountId
        })
      )
      console.log(
        `Balance of ${nearSenderAccountId} before withdrawing: ${oldBalance}`
      )

      console.log(
        `Withdrawing ${amount} tokens on NEAR blockchain in favor of ${ethReceiverAddress}.`
      )
      const txWithdraw = await nearJsonContractFunctionCall(
        nearErc20Account,
        nearSenderAccount,
        'withdraw',
        { amount: amount, recipient: ethReceiverAddress },
        new BN('300000000000000'),
        new BN(1)
      )
      console.log(`tx withdraw: ${JSON.stringify(txWithdraw)}`)

      TransferEthERC20FromNear.recordTransferLog({
        finished: 'withdraw',
        txWithdraw
      })
    } catch (txRevertMessage) {
      console.log('Failed to withdraw.')
      console.log(txRevertMessage.toString())
      TransferEthERC20FromNear.showRetryAndExit()
    }
  }

  static async findWithdrawInBlock ({ txWithdraw, nearSenderAccountId, near }) {
    try {
      let txReceiptId
      let txReceiptBlockHash
      let idType

      // Getting 1st tx
      const receipts = txWithdraw.transaction_outcome.outcome.receipt_ids
      if (receipts.length === 1) {
        txReceiptId = receipts[0]
        idType = 'receipt'
      } else {
        throw new Error(
          `Fungible token transaction call is expected to produce only one receipt, but produced: ${JSON.stringify(
            txWithdraw
          )}`
        )
      }

      // Getting 2nd tx
      try {
        txReceiptId = txWithdraw.receipts_outcome.find(
          (el) => el.id === txReceiptId
        ).outcome.status.SuccessReceiptId
        txReceiptBlockHash = txWithdraw.receipts_outcome.find(
          (el) => el.id === txReceiptId
        ).block_hash
      } catch (e) {
        throw new Error(`Invalid tx withdraw: ${JSON.stringify(txWithdraw)}`, e)
      }

      // Get block in which the receipt was processed.
      const receiptBlock = await backoff(10, () =>
        near.connection.provider.block({
          blockId: txReceiptBlockHash
        })
      )
      // Now wait for a final block with a strictly greater height. This block (or one of its ancestors) should hold the outcome, although this is not guaranteed if there are multiple shards.
      const outcomeBlock = await backoff(10, async () => {
        while (true) {
          const block = await near.connection.provider.block({
            finality: 'final'
          })
          if (
            Number(block.header.height) <= Number(receiptBlock.header.height)
          ) {
            await sleep(1000)
            continue
          }
          return block
        }
      })
      TransferEthERC20FromNear.recordTransferLog({
        finished: 'find-withdraw',
        txReceiptBlockHash,
        txReceiptId,
        outcomeBlock,
        idType
      })
    } catch (txRevertMessage) {
      console.log('Failed to find withdraw in block.')
      console.log(txRevertMessage.toString())
      TransferEthERC20FromNear.showRetryAndExit()
    }
  }

  static async waitBlock ({
    clientContract,
    outcomeBlock,
    robustWeb3,
    nearSenderAccountId,
    nearTokenContract,
    amount,
    idType,
    txReceiptId
  }) {
    // Wait for the block with the given receipt/transaction in Near2EthClient.
    try {
      const outcomeBlockHeight = Number(outcomeBlock.header.height)
      let clientBlockHeight
      let clientBlockHash
      while (true) {
        const clientState = await clientContract.methods.bridgeState().call()
        clientBlockHeight = Number(clientState.currentHeight)
        const clientBlockValidAfter = Number(clientState.nextValidAt)
        clientBlockHash = bs58.encode(
          toBuffer(
            await clientContract.methods.blockHashes(clientBlockHeight).call()
          )
        )

        console.log(
          `Current light client head is: hash=${clientBlockHash}, height=${clientBlockHeight}`
        )

        if (clientBlockHeight > outcomeBlockHeight) {
          console.log(
            `The block at height ${outcomeBlockHeight} is already available to the client.`
          )
          break
        } else {
          let delay =
            clientBlockValidAfter === 0
              ? await clientContract.methods.lockDuration().call()
              : clientBlockValidAfter -
                (await robustWeb3.getBlock('latest')).timestamp
          delay = Math.max(delay, 1)
          console.log(
            `Block ${outcomeBlockHeight} is not yet available. Sleeping for ${delay} seconds.`
          )
          await sleep(delay * 1000)
        }
      }
      console.log(`Withdrawn ${JSON.stringify(amount)}`)
      const newBalance = await backoff(10, () =>
        nearTokenContract.ft_balance_of({
          account_id: nearSenderAccountId
        })
      )
      console.log(
        `Balance of ${nearSenderAccountId} after withdrawing: ${newBalance}`
      )
      TransferEthERC20FromNear.recordTransferLog({
        finished: 'wait-block',
        clientBlockHashB58: clientBlockHash,
        idType,
        txReceiptId,
        clientBlockHeight
      })
    } catch (txRevertMessage) {
      console.log('Failed to wait for block occur in near on eth contract')
      console.log(txRevertMessage.toString())
      TransferEthERC20FromNear.showRetryAndExit()
    }
  }

  static async getProof ({
    idType,
    near,
    txReceiptId,
    nearSenderAccountId,
    clientBlockHashB58,
    clientBlockHeight
  }) {
    try {
      // Get the outcome proof only use block merkle root that we know is available on the Near2EthClient.
      let proofRes
      if (idType === 'transaction') {
        proofRes = await near.connection.provider.sendJsonRpc(
          'light_client_proof',
          {
            type: 'transaction',
            transaction_hash: txReceiptId,
            // TODO: Use proper sender.
            receiver_id: nearSenderAccountId,
            light_client_head: clientBlockHashB58
          }
        )
      } else if (idType === 'receipt') {
        proofRes = await near.connection.provider.sendJsonRpc(
          'light_client_proof',
          {
            type: 'receipt',
            receipt_id: txReceiptId,
            // TODO: Use proper sender.
            receiver_id: nearSenderAccountId,
            light_client_head: clientBlockHashB58
          }
        )
      } else {
        throw new Error('Unreachable')
      }
      TransferEthERC20FromNear.recordTransferLog({
        finished: 'get-proof',
        proofRes,
        clientBlockHeight
      })
    } catch (txRevertMessage) {
      console.log('Failed to get proof.')
      console.log(txRevertMessage.toString())
      TransferEthERC20FromNear.showRetryAndExit()
    }
  }

  static async unlock ({
    proverContract,
    proofRes,
    clientBlockHeight,
    ethErc20Address,
    ethReceiverAddress,
    ethTokenLockerContract,
    ethMasterAccount,
    ethGasMultiplier,
    robustWeb3
  }) {
    try {
      // Check that the proof is correct.
      const borshProofRes = borshifyOutcomeProof(proofRes)
      clientBlockHeight = new BN(clientBlockHeight)

      await proverContract.methods
        .proveOutcome(borshProofRes, clientBlockHeight)
        .call()

      const erc20 = new ethers.Contract(ethErc20Address, [
        'function balanceOf(address owner) view returns (uint256)'
      ], new ethers.providers.JsonRpcProvider(robustWeb3.ethNodeUrl))
      const oldBalance = await erc20.balanceOf(ethReceiverAddress)
      console.log(
        `ERC20 balance of ${ethReceiverAddress} before the transfer: ${oldBalance}`
      )

      await robustWeb3.callContract(
        ethTokenLockerContract,
        'unlockToken',
        [borshProofRes, clientBlockHeight],
        {
          from: ethMasterAccount,
          gas: 5000000,
          handleRevert: true,
          gasPrice: new BN(await robustWeb3.web3.eth.getGasPrice()).mul(new BN(ethGasMultiplier))
        }
      )

      const newBalance = await erc20.balanceOf(ethReceiverAddress)
      console.log(
        `ERC20 balance of ${ethReceiverAddress} after the transfer: ${newBalance}`
      )
    } catch (txRevertMessage) {
      console.log('Failed to unlock.')
      console.log(txRevertMessage.toString())
      TransferEthERC20FromNear.showRetryAndExit()
    }
  }

  static async execute ({
    parent: { args },
    amount,
    nearSenderAccount: nearSenderAccountId,
    ethReceiverAddress,
    nearNetworkId,
    nearNodeUrl,
    nearSenderSk,
    nearErc20Account,
    ethNodeUrl,
    ethMasterSk,
    ethClientArtifactPath,
    ethClientAddress,
    ethProverArtifactPath,
    ethProverAddress,
    ethLockerAbiPath,
    ethLockerAddress,
    ethErc20AbiPath,
    ethErc20Address,
    ethGasMultiplier
  }) {
    initialCmd = args.join(' ')
    ethReceiverAddress = remove0x(ethReceiverAddress)
    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      nearNetworkId,
      nearSenderAccountId,
      nearAPI.KeyPair.fromString(nearSenderSk)
    )
    const near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      masterAccount: nearSenderAccountId,
      keyStore
    })
    const nearSenderAccount = new nearAPI.Account(
      near.connection,
      nearSenderAccountId
    )
    await verifyAccount(near, nearSenderAccountId)

    const nearTokenContract = new nearAPI.Contract(
      nearSenderAccount,
      nearErc20Account,
      {
        changeMethods: ['new', 'withdraw'],
        viewMethods: ['ft_balance_of']
      }
    )
    const nearTokenContractBorsh = new NearMintableToken(
      nearSenderAccount,
      nearErc20Account
    )
    await nearTokenContractBorsh.accessKeyInit()

    const robustWeb3 = new RobustWeb3(ethNodeUrl)
    const web3 = robustWeb3.web3
    let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(normalizeEthKey(ethMasterSk))
    web3.eth.accounts.wallet.add(ethMasterAccount)
    web3.eth.defaultAccount = ethMasterAccount.address
    ethMasterAccount = ethMasterAccount.address
    const clientContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethClientArtifactPath)).abi,
      ethClientAddress,
      {
        from: ethMasterAccount,
        handleRevert: true
      }
    )
    const proverContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethProverArtifactPath)).abi,
      ethProverAddress,
      {
        from: ethMasterAccount,
        handleRevert: true
      }
    )
    const ethTokenLockerContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethLockerAbiPath)),
      ethLockerAddress,
      {
        from: ethMasterAccount,
        handleRevert: true
      }
    )

    let transferLog = TransferEthERC20FromNear.loadTransferLog()
    if (transferLog.finished === undefined) {
      await TransferEthERC20FromNear.withdraw({
        nearTokenContract,
        nearSenderAccountId,
        nearErc20Account,
        amount,
        ethReceiverAddress,
        nearSenderAccount
      })
      transferLog = TransferEthERC20FromNear.loadTransferLog()
    }
    if (transferLog.finished === 'withdraw') {
      await TransferEthERC20FromNear.findWithdrawInBlock({
        txWithdraw: transferLog.txWithdraw,
        nearSenderAccountId,
        near
      })
      transferLog = TransferEthERC20FromNear.loadTransferLog()
    }
    if (transferLog.finished === 'find-withdraw') {
      await TransferEthERC20FromNear.waitBlock({
        clientContract,
        robustWeb3,
        outcomeBlock: transferLog.outcomeBlock,
        nearSenderAccountId,
        nearTokenContract,
        amount,
        idType: transferLog.idType,
        txReceiptId: transferLog.txReceiptId
      })
      transferLog = TransferEthERC20FromNear.loadTransferLog()
    }
    if (transferLog.finished === 'wait-block') {
      await TransferEthERC20FromNear.getProof({
        idType: transferLog.idType,
        near,
        txReceiptId: transferLog.txReceiptId,
        nearSenderAccountId,
        clientBlockHashB58: transferLog.clientBlockHashB58,
        clientBlockHeight: transferLog.clientBlockHeight
      })
      transferLog = TransferEthERC20FromNear.loadTransferLog()
    }
    if (transferLog.finished === 'get-proof') {
      await TransferEthERC20FromNear.unlock({
        proverContract,
        proofRes: transferLog.proofRes,
        clientBlockHeight: transferLog.clientBlockHeight,
        ethErc20Address,
        ethReceiverAddress,
        ethTokenLockerContract,
        ethMasterAccount,
        ethGasMultiplier,
        robustWeb3
      })
    }

    process.exit(0)
  }
}

exports.TransferEthERC20FromNear = TransferEthERC20FromNear

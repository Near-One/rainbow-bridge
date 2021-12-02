const BN = require('bn.js')
const blockFromRpc = require('@ethereumjs/block/dist/from-rpc')
const Common = require('@ethereumjs/common').default
const got = require('got')
const {
  Web3,
  BorshContract,
  hexToBuffer,
  readerToHex,
  sleep
} = require('rainbow-bridge-utils')
const roots = require('./dag_merkle_roots.json')

/// Get Ethereum block by number from RPC, and returns raw json object.
async function getEthBlock (number, RobustWeb3) {
  let attempts = 10
  let blockData

  while (attempts > 0) {
    /// Need to call RPC directly, since function `blockFromRpc` works
    /// when all fields returned by RPC are present. After EIP1559 was introduced
    /// tools that abstract this calls are missing the field `baseFeePerGas`
    blockData = await got.post(RobustWeb3.ethNodeUrl, {
      json: {
        id: 0,
        jsonrpc: '2.0',
        method: 'eth_getBlockByNumber',
        params: [
          '0x' + number.toString(16),
          false
        ]
      },
      responseType: 'json'
    })

    /// When the block to be queried is the last one produced, RPC can return null.
    /// Retrying fix this problem.
    if (blockData.body.result === null) {
      attempts -= 1
      await sleep(800)
    } else {
      break
    }
  }
  return blockData.body.result
}

/// bridgeId matches nearNetworkId. It is one of two strings [testnet / mainnet]
function web3BlockToRlp (blockData, bridgeId, validateHeaderMode) {
  if (validateHeaderMode === 'bsc') {
    const block = blockFromRpc.default(blockData, [], {})
    return block.header.serialize()
  }

  let chain
  if (bridgeId === 'testnet') {
    chain = 'ropsten'
  } else {
    chain = 'mainnet'
  }
  const common = new Common({ chain })

  /// baseFeePerGas was introduced after london hard fork.
  /// TODO: Use better way to detect current hard fork.
  if (blockData.baseFeePerGas !== undefined) {
    common.setHardfork('london')
    common.setEIPs([1559])
  }

  const block = blockFromRpc.default(blockData, [], { common })
  return block.header.serialize()
}

const borshSchema = {
  bool: {
    kind: 'function',
    ser: (b) => Buffer.from(Web3.utils.hexToBytes(b ? '0x01' : '0x00')),
    deser: (z) => readerToHex(1)(z) === '0x01'
  },
  initInput: {
    kind: 'struct',
    fields: [
      ['validate_header', 'bool'],
      ['validate_header_mode', 'string'],
      ['dags_start_epoch', 'u64'],
      ['dags_merkle_roots', ['H128']],
      ['first_header', ['u8']],
      ['hashes_gc_threshold', 'u64'],
      ['finalized_gc_threshold', 'u64'],
      ['num_confirmations', 'u64'],
      ['trusted_signer', '?AccountId'],
      ['chain_id', 'u64']
    ]
  },
  dagMerkleRootInput: {
    kind: 'struct',
    fields: [['epoch', 'u64']]
  },
  addBlockHeaderInput: {
    kind: 'struct',
    fields: [
      ['block_header', ['u8']],
      ['dag_nodes', ['DoubleNodeWithMerkleProof']]
    ]
  },
  DoubleNodeWithMerkleProof: {
    kind: 'struct',
    fields: [
      ['dag_nodes', ['H512']],
      ['proof', ['H128']]
    ]
  },
  H128: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(16)
  },
  H256: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(32)
  },
  H512: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(64)
  },
  '?H256': {
    kind: 'option',
    type: 'H256'
  },
  '?AccountId': {
    kind: 'option',
    type: 'string'
  }
}

class EthOnNearClientContract extends BorshContract {
  constructor (account, contractId) {
    super(borshSchema, account, contractId, {
      viewMethods: [
        {
          methodName: 'initialized',
          inputFieldType: null,
          outputFieldType: 'bool'
        },
        {
          methodName: 'dag_merkle_root',
          inputFieldType: 'dagMerkleRootInput',
          outputFieldType: 'H128'
        },
        {
          methodName: 'last_block_number',
          inputFieldType: null,
          outputFieldType: 'u64'
        },
        {
          methodName: 'block_hash',
          inputFieldType: 'u64',
          outputFieldType: '?H256'
        },
        {
          methodName: 'known_hashes',
          inputFieldType: 'u64',
          outputFieldType: ['H256']
        },
        {
          methodName: 'block_hash_safe',
          inputFieldType: 'u64',
          outputFieldType: '?H256'
        }
      ],

      changeMethods: [
        {
          methodName: 'init',
          inputFieldType: 'initInput',
          outputFieldType: null
        },
        {
          methodName: 'add_block_header',
          inputFieldType: 'addBlockHeaderInput',
          outputFieldType: null
        }
      ]
    })
  }

  // Call initialization methods on the contract.
  // If validateHeader is true will do header validation otherwise it won't.
  async maybeInitialize (hashesGcThreshold, finalizedGcThreshold, numConfirmations, validateHeader, validateHeaderMode, trustedSigner, chainID, robustWeb3, bridgeId) {
    await this.accessKeyInit()
    let initialized = false
    try {
      initialized = await this.initialized()
    } catch (e) { }
    if (!initialized) {
      console.log('EthOnNearClient is not initialized, initializing...')
      let lastBlockNumber = await robustWeb3.getBlockNumber()
      let blockData

      // if validateHeaderMode is bsc(POSA) we have to get the last epoch header
      if (validateHeaderMode === 'bsc' && lastBlockNumber % 200 !== 0) {
        lastBlockNumber = lastBlockNumber - lastBlockNumber % 200
        blockData = await robustWeb3.getBlock(lastBlockNumber)
      } else {
        blockData = await getEthBlock(lastBlockNumber, robustWeb3)
      }

      const blockRlp = web3BlockToRlp(blockData, bridgeId, validateHeaderMode)
      console.log(blockRlp)
      await this.init(
        {
          validate_header: validateHeader,
          validate_header_mode: validateHeaderMode,
          dags_start_epoch: 0,
          dags_merkle_roots: roots.dag_merkle_roots,
          first_header: blockRlp,
          hashes_gc_threshold: hashesGcThreshold,
          finalized_gc_threshold: finalizedGcThreshold,
          num_confirmations: numConfirmations,
          trusted_signer: trustedSigner,
          chain_id: chainID
        },
        new BN('300000000000000')
      )
      console.log('EthOnNearClient initialized')
    }

    console.log('Checking EthOnNearClient initialization.')
    const firstRoot = await this.dag_merkle_root({
      epoch: 0
    })
    const lastRoot = await this.dag_merkle_root({
      epoch: 511
    })
    if (
      !(
        firstRoot === '0x55b891e842e58f58956a847cbbf67821' &&
        lastRoot === '0x7a9010568819de327a24fa495029adcb'
      )
    ) {
      console.log(
        `EthOnNearClient initialization error! The first and last roots are ${firstRoot} and ${lastRoot}`
      )
      process.exit(1)
    }
  }
}

exports.EthOnNearClientContract = EthOnNearClientContract
exports.web3BlockToRlp = web3BlockToRlp
exports.getEthBlock = getEthBlock
exports.borshSchema = borshSchema

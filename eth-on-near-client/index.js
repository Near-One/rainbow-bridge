const { web3BlockToRlp } = require('../eth2near-relay')

const Web3 = require('web3')
const BN = require('bn.js')
const { BorshContract, hexToBuffer, readerToHex } = require('../rainbow/borsh')
const roots = require('./dag_merkle_roots.json')

const borshSchema = {
  bool: {
    kind: 'function',
    // @ts-ignore
    ser: (b) => Buffer.from(Web3.utils.hexToBytes(b ? '0x01' : '0x00')),
    deser: (z) => readerToHex(1)(z) === '0x01',
  },
  initInput: {
    kind: 'struct',
    fields: [
      ['validate_ethash', 'bool'],
      ['dags_start_epoch', 'u64'],
      ['dags_merkle_roots', ['H128']],
      ['first_header', ['u8']],
      ['hashes_gc_threshold', 'u64'],
      ['finalized_gc_threshold', 'u64'],
      ['num_confirmations', 'u64'],
      ['trusted_signer', '?AccountId']
    ],
  },
  dagMerkleRootInput: {
    kind: 'struct',
    fields: [['epoch', 'u64']],
  },
  addBlockHeaderInput: {
    kind: 'struct',
    fields: [
      ['block_header', ['u8']],
      ['dag_nodes', ['DoubleNodeWithMerkleProof']],
    ],
  },
  DoubleNodeWithMerkleProof: {
    kind: 'struct',
    fields: [
      ['dag_nodes', ['H512']],
      ['proof', ['H128']],
    ],
  },
  H128: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(16),
  },
  H256: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(32),
  },
  H512: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(64),
  },
  '?H256': {
    kind: 'option',
    type: 'H256',
  },
  '?AccountId': {
    kind: 'option',
    type: 'string',
  }
}

class EthOnNearClientContract extends BorshContract {
  constructor(account, contractId) {
    super(borshSchema, account, contractId, {
      viewMethods: [
        {
          methodName: 'initialized',
          inputFieldType: null,
          outputFieldType: 'bool',
        },
        {
          methodName: 'dag_merkle_root',
          inputFieldType: 'dagMerkleRootInput',
          outputFieldType: 'H128',
        },
        {
          methodName: 'last_block_number',
          inputFieldType: null,
          outputFieldType: 'u64',
        },
        {
          methodName: 'block_hash',
          inputFieldType: 'u64',
          outputFieldType: '?H256',
        },
        {
          methodName: 'known_hashes',
          inputFieldType: 'u64',
          outputFieldType: ['H256'],
        },
        {
          methodName: 'block_hash_safe',
          inputFieldType: 'u64',
          outputFieldType: '?H256',
        },
      ],

      changeMethods: [
        {
          methodName: 'init',
          inputFieldType: 'initInput',
          outputFieldType: null,
        },
        {
          methodName: 'add_block_header',
          inputFieldType: 'addBlockHeaderInput',
          outputFieldType: null,
        },
      ],
    })
  }

  // Call initialization methods on the contract.
  // If validate_ethash is true will do ethash validation otherwise it won't.
  async maybeInitialize(validate_ethash, trusted_signer, robustWeb3) {
    await this.accessKeyInit()
    let initialized = false
    try {
      // @ts-ignore
      initialized = await this.initialized()
    } catch (e) { }
    if (!initialized) {
      console.log('EthOnNearClient is not initialized, initializing...')
      const last_block_number = await robustWeb3.getBlockNumber()
      const blockRlp = web3BlockToRlp(
        await robustWeb3.getBlock(last_block_number)
      )
      // @ts-ignore
      await this.init(
        {
          validate_ethash: validate_ethash,
          dags_start_epoch: 0,
          dags_merkle_roots: roots.dag_merkle_roots,
          first_header: blockRlp,
          hashes_gc_threshold: 40000,
          finalized_gc_threshold: 500,
          num_confirmations: 10,
          trusted_signer,
        },
        new BN('300000000000000')
      )
      console.log('EthOnNearClient initialized')
    }

    console.log('Checking EthOnNearClient initialization.')
    // @ts-ignore
    const first_root = await this.dag_merkle_root({
      epoch: 0,
    })
    // @ts-ignore
    const last_root = await this.dag_merkle_root({
      epoch: 511,
    })
    if (
      !(
        first_root === '0x55b891e842e58f58956a847cbbf67821' &&
        last_root === '0x4aa6ca6ebef942d8766065b2e590fd32'
      )
    ) {
      console.log(
        `EthOnNearClient initialization error! The first and last roots are ${first_root} and ${last_root}`
      )
      process.exit(1)
    }
  }
}

exports.EthOnNearClientContract = EthOnNearClientContract

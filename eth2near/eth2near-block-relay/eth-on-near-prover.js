const {
  BorshContract,
  hexToBuffer,
  readerToHex
} = require('rainbow-bridge-utils')

const borshSchema = {
  initInput: {
    kind: 'struct',
    fields: [['bridge_smart_contract', 'string']]
  },
  assertEthbridgeHashInput: {
    kind: 'struct',
    fields: [
      ['block_number', 'u64'],
      ['expected_block_hash', 'H256']
    ]
  },
  H256: {
    kind: 'function',
    ser: hexToBuffer,
    deser: readerToHex(32)
  },
  verifyLogEntry: {
    kind: 'struct',
    fields: [
      ['log_index', 'u64'],
      ['log_entry_data', ['u8']],
      ['receipt_index', 'u64'],
      ['receipt_data', ['u8']],
      ['header_data', ['u8']],
      ['proof', [['u8']]],
      ['skip_bridge_call', 'bool']
    ]
  }
}

class EthOnNearProverContract extends BorshContract {
  constructor (account, contractId) {
    super(borshSchema, account, contractId, {
      viewMethods: [],
      changeMethods: [
        {
          methodName: 'verify_log_entry',
          inputFieldType: 'verifyLogEntry',
          outputFieldType: 'bool'
        },
        {
          methodName: 'init',
          inputFieldType: 'initInput',
          outputFieldType: null
        },
        {
          methodName: 'assert_ethbridge_hash',
          inputFieldType: 'assertEthbridgeHashInput',
          outputFieldType: 'bool'
        }
      ]
    })
  }

  async maybeInitialize (ethClientAccId) {
    await this.accessKeyInit()

    try {
      await this.init({
        bridge_smart_contract: ethClientAccId
      })
      console.log('ETH2NEARProver initialized')
    } catch (e) {
      if (
        e.message &&
        e.message.includes('The contract is already initialized')
      ) {
        console.log('Contract is already initialized')
      } else {
        throw e
      }
    }
  }
}

exports.EthOnNearProverContract = EthOnNearProverContract
// For debugging;
exports.borshSchema = borshSchema

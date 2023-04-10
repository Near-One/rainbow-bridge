const bs58 = require('bs58')

const BN = require('bn.js')
const { TextDecoder } = require('util')
const { borshifyOutcomeProof } = require('./borshify-proof')
const {
  Web3,
  nearAPI,
  sleep,
  backoff,
  signAndSendTransaction
} = require('./robust')

const VALIDATOR_STAKE_V1 = 0
const VALIDATOR_STAKE_V2 = 1

class BorshError extends Error {
  constructor (message) {
    super(message)

    this.name = this.constructor.name
  }
}

function serializeField (schema, value, fieldType, writer) {
  if (fieldType === 'u8') {
    writer.writeU8(value)
  } else if (fieldType === 'u64') {
    writer.writeU64(value)
  } else if (fieldType === 'u128') {
    writer.writeU128(value)
  } else if (fieldType === 'bool') {
    return writer.writeU8(value ? 1 : 0)
  } else if (fieldType === 'string') {
    return writer.writeString(value)
  } else if (fieldType instanceof Array) {
    if (typeof fieldType[0] === 'number') {
      if (value.length !== fieldType[0]) {
        throw new Error(
          `Expecting byte array of length ${fieldType[0]}, but got ${value.length} bytes`
        )
      }
      writer.writeFixedArray(value)
    } else {
      writer.writeArray(value, (item) => {
        serializeField(schema, item, fieldType[0], writer)
      })
    }
  } else {
    const structSchema = schema[fieldType]
    if (!structSchema) {
      throw new Error(`Schema type ${fieldType} is missing in schema`)
    }
    if (structSchema.kind === 'option') {
      if (value === null) {
        writer.writeU8(0)
      } else {
        writer.writeU8(1)
        serializeField(schema, value, structSchema.type, writer)
      }
    } else if (structSchema.kind === 'struct') {
      structSchema.fields.map(([fieldName, fieldType]) =>
        serializeField(schema, value[fieldName], fieldType, writer)
      )
    } else if (structSchema.kind === 'function') {
      writer.writeBuffer(structSchema.ser(value))
    } else {
      throw new Error(
        `Unexpected schema kind: ${structSchema.kind} for ${fieldType}`
      )
    }
  }
}

function deserializeField (schema, fieldType, reader) {
  if (fieldType === 'u8') {
    return reader.readU8()
  } else if (fieldType === 'u64') {
    return reader.readU64()
  } else if (fieldType === 'u128') {
    return reader.readU128()
  } else if (fieldType === 'bool') {
    return !!reader.readU8()
  } else if (fieldType === 'string') {
    return reader.readString()
  } else if (fieldType instanceof Array) {
    if (typeof fieldType[0] === 'number') {
      return reader.readFixedArray(fieldType[0])
    } else {
      return reader.readArray(() =>
        deserializeField(schema, fieldType[0], reader)
      )
    }
  } else {
    const structSchema = schema[fieldType]
    if (!structSchema) {
      throw new Error(`Schema type ${fieldType} is missing in schema`)
    }
    if (structSchema.kind === 'option') {
      const optionRes = reader.readU8()
      if (optionRes === 0) {
        return null
      } else if (optionRes === 1) {
        return deserializeField(schema, structSchema.type, reader)
      } else {
        throw new Error(`Unexpected option flag: ${optionRes}`)
      }
    } else if (structSchema.kind === 'struct') {
      const result = {}
      for (const [fieldName, fieldType] of structSchema.fields) {
        result[fieldName] = deserializeField(schema, fieldType, reader)
      }
      return result
    } else if (structSchema.kind === 'function') {
      return structSchema.deser(reader, schema)
    } else {
      throw new Error(
        `Unexpected schema kind: ${structSchema.kind} for ${fieldType}`
      )
    }
  }
}

/// Serialize given object using schema of the form:
/// { class_name -> [ [field_name, field_type], .. ], .. }
function serialize (schema, fieldType, obj) {
  if (fieldType === null) {
    return new Uint8Array()
  }
  const writer = new nearAPI.utils.serialize.BinaryWriter()
  serializeField(schema, obj, fieldType, writer)
  return writer.toArray()
}

class BinaryReader {
  constructor (buf) {
    this.buf = buf
    this.offset = 0
  }

  readU8 () {
    const value = this.buf.readUInt8(this.offset)
    this.offset += 1
    return value
  }

  readU32 () {
    const value = this.buf.readUInt32LE(this.offset)
    this.offset += 4
    return value
  }

  readU64 () {
    const buf = this.readBuffer(8)
    return new BN(buf, 'le')
  }

  readU128 () {
    const buf = this.readBuffer(16)
    return new BN(buf, 'le')
  }

  readBuffer (len) {
    if (this.offset + len > this.buf.length) {
      throw new BorshError(`Expected buffer length ${len} isn't within bounds`)
    }
    const result = this.buf.slice(this.offset, this.offset + len)
    this.offset += len
    return result
  }

  readString () {
    const len = this.readU32()
    const buf = this.readBuffer(len)
    const textDecoder = TextDecoder()
    try {
      // NOTE: Using TextDecoder to fail on invalid UTF-8
      return textDecoder.decode(buf)
    } catch (e) {
      throw new BorshError(`Error decoding UTF-8 string: ${e}`)
    }
  }

  readFixedArray (len) {
    return new Uint8Array(this.readBuffer(len))
  }

  readArray (fn) {
    const len = this.readU32()
    const result = []
    for (let i = 0; i < len; ++i) {
      result.push(fn())
    }
    return result
  }
}

function deserialize (schema, fieldType, buffer) {
  if (fieldType === null) {
    return null
  }
  const reader = new BinaryReader(buffer)
  const result = deserializeField(schema, fieldType, reader)
  if (reader.offset < buffer.length) {
    throw new Error(
      `Unexpected ${
        buffer.length - reader.offset
      } bytes after deserialized data`
    )
  }
  return result
}

const DEFAULT_FUNC_CALL_AMOUNT = new BN('300000000000000')

const RETRY_TX_STATUS = 10

const signAndSendTransactionAsync = async (
  accessKey,
  account,
  receiverId,
  actions
) => {
  const status = await account.connection.provider.status()
  const [txHash, signedTx] = await nearAPI.transactions.signTransaction(
    receiverId,
    ++accessKey.accessKey.nonce,
    actions,
    nearAPI.utils.serialize.base_decode(status.sync_info.latest_block_hash),
    account.connection.signer,
    account.accountId,
    account.connection.networkId
  )
  const bytes = signedTx.encode()
  await account.connection.provider.sendJsonRpc('broadcast_tx_async', [
    Buffer.from(bytes).toString('base64')
  ])
  console.log('TxHash', nearAPI.utils.serialize.base_encode(txHash))
  return txHash
}

const txnStatus = async (
  account,
  txHash,
  retries = RETRY_TX_STATUS,
  wait = 1000
) => {
  let result
  for (let i = 0; i < retries; i++) {
    try {
      result = await account.connection.provider.txStatus(
        txHash,
        account.accountId
      )
      if (
        result.status.SuccessValue !== undefined ||
        result.status.Failure !== undefined
      ) {
        break
      }
    } catch (e) {
      await sleep(wait)
    }
  }
  if (!result) {
    throw new Error(
      `Transaction ${txHash} didn't finish after ${
        (retries * wait) / 1000
      } seconds`
    )
  }

  const flatLogs = [
    result.transaction_outcome,
    ...result.receipts_outcome
  ].reduce((acc, it) => acc.concat(it.outcome.logs), [])
  if (flatLogs && flatLogs !== []) {
    console.log(flatLogs)
  }

  if (result.status.SuccessValue !== undefined) {
    return result
  }

  throw new Error(JSON.stringify(result.status.Failure))
}

function getBorshTransactionLastResult (txResult) {
  return txResult && Buffer.from(txResult.status.SuccessValue, 'base64')
}

class BorshContract {
  constructor (borshSchema, account, contractId, options) {
    this.account = account
    this.contractId = contractId
    options.viewMethods.forEach((d) => {
      Object.defineProperty(this, d.methodName, {
        writable: false,
        enumerable: true,
        value: async (args) => {
          args = serialize(borshSchema, d.inputFieldType, args)
          const result = await backoff(10, () =>
            this.account.connection.provider.query({
              request_type: 'call_function',
              finality: 'final',
              account_id: this.contractId,
              method_name: d.methodName,
              args_base64: args.toString('base64')
            })
          )
          if (result.logs) {
            this.account.printLogs(this.contractId, result.logs)
          }
          return (
            result.result &&
            result.result.length > 0 &&
            deserialize(
              borshSchema,
              d.outputFieldType,
              Buffer.from(result.result)
            )
          )
        }
      })
    })
    options.changeMethods.forEach((d) => {
      Object.defineProperty(this, d.methodName, {
        writable: false,
        enumerable: true,
        value: async (args, gas, amount) => {
          args = serialize(borshSchema, d.inputFieldType, args)

          const rawResult = await signAndSendTransaction(
            this.accessKey,
            this.account,
            this.contractId,
            [
              nearAPI.transactions.functionCall(
                d.methodName,
                Buffer.from(args),
                gas || DEFAULT_FUNC_CALL_AMOUNT,
                amount
              )
            ]
          )

          const result = getBorshTransactionLastResult(rawResult)
          return result && deserialize(borshSchema, d.outputFieldType, result)
        }
      })
    })

    options.changeMethods.forEach((d) => {
      Object.defineProperty(this, d.methodName + '_async', {
        writable: false,
        enumerable: true,
        value: async (args, gas, amount) => {
          args = serialize(borshSchema, d.inputFieldType, args)
          return await signAndSendTransactionAsync(
            this.accessKey,
            this.account,
            this.contractId,
            [
              nearAPI.transactions.functionCall(
                d.methodName,
                Buffer.from(args),
                gas || DEFAULT_FUNC_CALL_AMOUNT,
                amount
              )
            ]
          )
        }
      })
    })
  }

  async accessKeyInit () {
    await this.account.ready

    this.accessKey = await this.account.findAccessKey()
    if (!this.accessKey) {
      throw new Error(
        `Can not sign transactions for account ${this.account.accountId}, no matching key pair found in Signer.`,
        'KeyNotFound'
      )
    }
  }
}

function borshify (block) {
  return Buffer.concat([
    bs58.decode(block.prev_block_hash),
    bs58.decode(block.next_block_inner_hash),
    Buffer.concat([
      Web3.utils.toBN(block.inner_lite.height).toBuffer('le', 8),
      bs58.decode(block.inner_lite.epoch_id),
      bs58.decode(block.inner_lite.next_epoch_id),
      bs58.decode(block.inner_lite.prev_state_root),
      bs58.decode(block.inner_lite.outcome_root),
      Web3.utils
        .toBN(block.inner_lite.timestamp_nanosec || block.inner_lite.timestamp)
        .toBuffer('le', 8),
      bs58.decode(block.inner_lite.next_bp_hash),
      bs58.decode(block.inner_lite.block_merkle_root)
    ]),
    bs58.decode(block.inner_rest_hash),

    Buffer.from([1]),
    Web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
    Buffer.concat(
      block.next_bps.map((nextBp) =>
        Buffer.concat([
          Object.prototype.hasOwnProperty.call(nextBp, 'validator_stake_struct_version') && nextBp.validator_stake_struct_version === 'V2'
            ? Buffer.from([VALIDATOR_STAKE_V2])
            : Buffer.from([VALIDATOR_STAKE_V1]),
          Web3.utils.toBN(nextBp.account_id.length).toBuffer('le', 4),
          Buffer.from(nextBp.account_id),
          nextBp.public_key.substr(0, 8) === 'ed25519:'
            ? Buffer.from([0])
            : Buffer.from([1]),
          bs58.decode(nextBp.public_key.substr(8)),
          Web3.utils.toBN(nextBp.stake).toBuffer('le', 16)
        ])
      )
    ),

    Web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
    Buffer.concat(
      block.approvals_after_next.map((signature) =>
        signature === null
          ? Buffer.from([0])
          : Buffer.concat([
            Buffer.from([1]),
            signature.substr(0, 8) === 'ed25519:'
              ? Buffer.from([0])
              : Buffer.from([1]),
            bs58.decode(signature.substr(8))
          ])
      )
    )
  ])
}

function borshifyInitialValidators (initialValidators) {
  return Buffer.concat([
    Web3.utils.toBN(initialValidators.length).toBuffer('le', 4),
    Buffer.concat(
      initialValidators.map((nextBp) =>
        Buffer.concat([
          Object.prototype.hasOwnProperty.call(nextBp, 'validator_stake_struct_version') && nextBp.validator_stake_struct_version === 'V2'
            ? Buffer.from([VALIDATOR_STAKE_V2])
            : Buffer.from([VALIDATOR_STAKE_V1]),
          Web3.utils.toBN(nextBp.account_id.length).toBuffer('le', 4),
          Buffer.from(nextBp.account_id),
          nextBp.public_key.substr(0, 8) === 'ed25519:'
            ? Buffer.from([0])
            : Buffer.from([1]),
          bs58.decode(nextBp.public_key.substr(8)),
          Web3.utils.toBN(nextBp.stake).toBuffer('le', 16)
        ])
      )
    )
  ])
}

const hexToBuffer = (hex) => Buffer.from(Web3.utils.hexToBytes(hex))
const readerToHex = (len) => (reader) =>
  Web3.utils.bytesToHex(reader.readFixedArray(len))

exports.BorshContract = BorshContract
exports.hexToBuffer = hexToBuffer
exports.readerToHex = readerToHex
exports.borshifyOutcomeProof = borshifyOutcomeProof
exports.txnStatus = txnStatus
exports.borshify = borshify
exports.borshifyInitialValidators = borshifyInitialValidators
exports.signAndSendTransaction = signAndSendTransaction

// For debugging only.
exports.serialize = serialize

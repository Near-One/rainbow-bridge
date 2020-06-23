const Web3 = require('web3');
const nearlib = require('near-api-js');

const BN = require('bn.js');
const { TextDecoder } = require('util');

class BorshError extends Error {
    constructor (message) {
        super(message);

        this.name = this.constructor.name;
    }
}

function serializeField (schema, value, fieldType, writer) {
    if (fieldType === 'u8') {
        writer.write_u8(value);
    } else if (fieldType === 'u64') {
        writer.write_u64(value);
    } else if (fieldType === 'u128') {
        writer.write_u128(value);
    } else if (fieldType === 'bool') {
        return writer.write_u8(value ? 1 : 0);
    } else if (fieldType === 'string') {
        return writer.write_string(value);
    } else if (fieldType instanceof Array) {
        if (typeof fieldType[0] === 'number') {
            if (value.length !== fieldType[0]) {
                throw new Error(`Expecting byte array of length ${fieldType[0]}, but got ${value.length} bytes`);
            }
            writer.write_fixed_array(value);
        } else {
            writer.write_array(value, (item) => {
                serializeField(schema, item, fieldType[0], writer);
            });
        }
    } else {
        const structSchema = schema[fieldType];
        if (!structSchema) {
            throw new Error(`Schema type ${fieldType} is missing in schema`);
        }
        if (structSchema.kind === 'option') {
            if (value === null) {
                writer.write_u8(0);
            } else {
                writer.write_u8(1);
                serializeField(schema, value, structSchema.type, writer);
            }
        } else if (structSchema.kind === 'struct') {
            structSchema.fields.map(([fieldName, fieldType]) => {
                serializeField(schema, value[fieldName], fieldType, writer);
            });
        } else if (structSchema.kind === 'function') {
            writer.write_buffer(structSchema.ser(value));
        } else {
            throw new Error(`Unexpected schema kind: ${structSchema.kind} for ${fieldType}`);
        }
    }
}

function deserializeField (schema, fieldType, reader) {
    if (fieldType === 'u8') {
        return reader.read_u8();
    } else if (fieldType === 'u64') {
        return reader.read_u64();
    } else if (fieldType === 'u128') {
        return reader.read_u128();
    } else if (fieldType === 'bool') {
        return !!reader.read_u8();
    } else if (fieldType === 'string') {
        return reader.read_string();
    } else if (fieldType instanceof Array) {
        if (typeof fieldType[0] === 'number') {
            return reader.read_fixed_array(fieldType[0]);
        } else {
            return reader.read_array(() => deserializeField(schema, fieldType[0], reader));
        }
    } else {
        const structSchema = schema[fieldType];
        if (!structSchema) {
            throw new Error(`Schema type ${fieldType} is missing in schema`);
        }
        if (structSchema.kind === 'option') {
            const optionRes = reader.read_u8();
            if (optionRes === 0) {
                return null;
            } else if (optionRes === 1) {
                return deserializeField(schema, structSchema.type, reader);
            } else {
                throw new Error(`Unexpected option flag: ${optionRes}`);
            }
        } else if (structSchema.kind === 'struct') {
            const result = {};
            for (const [fieldName, fieldType] of structSchema.fields) {
                result[fieldName] = deserializeField(schema, fieldType, reader);
            }
            return result;
        } else if (structSchema.kind === 'function') {
            return structSchema.deser(reader, schema);
        } else {
            throw new Error(`Unexpected schema kind: ${structSchema.kind} for ${fieldType}`);
        }
    }
}

/// Serialize given object using schema of the form:
/// { class_name -> [ [field_name, field_type], .. ], .. }
function serialize (schema, fieldType, obj) {
    if (fieldType === null) {
        return new Uint8Array();
    }
    const writer = new nearlib.utils.serialize.BinaryWriter();
    serializeField(schema, obj, fieldType, writer);
    return writer.toArray();
}

class BinaryReader {
    constructor (buf) {
        this.buf = buf;
        this.offset = 0;
    }

    read_u8 () {
        const value = this.buf.readUInt8(this.offset);
        this.offset += 1;
        return value;
    }

    read_u32 () {
        const value = this.buf.readUInt32LE(this.offset);
        this.offset += 4;
        return value;
    }

    read_u64 () {
        const buf = this.read_buffer(8);
        return new BN(buf, 'le');
    }

    read_u128 () {
        const buf = this.read_buffer(16);
        return new BN(buf, 'le');
    }

    read_buffer (len) {
        if ((this.offset + len) > this.buf.length) {
            throw new BorshError(`Expected buffer length ${len} isn't within bounds`);
        }
        const result = this.buf.slice(this.offset, this.offset + len);
        this.offset += len;
        return result;
    }

    read_string () {
        const len = this.read_u32();
        const buf = this.read_buffer(len);
        // @ts-ignore
        const textDecoder = TextDecoder();
        try {
        // NOTE: Using TextDecoder to fail on invalid UTF-8
            return textDecoder.decode(buf);
        } catch (e) {
            throw new BorshError(`Error decoding UTF-8 string: ${e}`);
        }
    }

    read_fixed_array (len) {
        return new Uint8Array(this.read_buffer(len));
    }

    read_array (fn) {
        const len = this.read_u32();
        const result = [];
        for (let i = 0; i < len; ++i) {
            result.push(fn());
        }
        return result;
    }
}

function deserialize (schema, fieldType, buffer) {
    if (fieldType === null) {
        return null;
    }
    const reader = new BinaryReader(buffer);
    const result = deserializeField(schema, fieldType, reader);
    if (reader.offset < buffer.length) {
        throw new Error(`Unexpected ${buffer.length - reader.offset} bytes after deserialized data`);
    }
    return result;
}

const DEFAULT_FUNC_CALL_AMOUNT = new BN('10000000000000000');

const signAndSendTransaction = async (accessKey, account, receiverId, actions) => {
    // TODO: Find matching access key based on transaction
    const status = await account.connection.provider.status();

    const [txHash, signedTx] = await nearlib.transactions.signTransaction(
        receiverId, ++accessKey.nonce, actions, nearlib.utils.serialize.base_decode(status.sync_info.latest_block_hash), account.connection.signer, account.accountId, account.connection.networkId,
    );
    console.log('TxHash', nearlib.utils.serialize.base_encode(txHash));

    const result = await account.connection.provider.sendTransaction(signedTx);

    const flatLogs = [result.transaction_outcome, ...result.receipts_outcome].reduce((acc, it) => acc.concat(it.outcome.logs), []);
    if (flatLogs) {
        console.log(flatLogs);
    }

    if (result.status.Failure) {
        throw new Error(JSON.stringify(result.status.Failure));
    }

    return result;
};

function getBorshTransactionLastResult (txResult) {
    return txResult && Buffer.from(txResult.status.SuccessValue, 'base64');
}

class BorshContract {
    constructor (borshSchema, account, contractId, options) {
        this.account = account;
        this.contractId = contractId;
        options.viewMethods.forEach((d) => {
            Object.defineProperty(this, d.methodName, {
                writable: false,
                enumerable: true,
                value: async (args) => {
                    args = serialize(borshSchema, d.inputFieldType, args);
                    const result = await this.account.connection.provider.query(`call/${this.contractId}/${d.methodName}`, nearlib.utils.serialize.base_encode(args));
                    if (result.logs) {
                        this.account.printLogs(this.contractId, result.logs);
                    }
                    return result.result && result.result.length > 0 && deserialize(borshSchema, d.outputFieldType, Buffer.from(result.result));
                },
            });
        });
        options.changeMethods.forEach((d) => {
            Object.defineProperty(this, d.methodName, {
                writable: false,
                enumerable: true,
                value: async (args, gas, amount) => {
                    args = serialize(borshSchema, d.inputFieldType, args);
                    try {
                        const rawResult = await signAndSendTransaction(this.accessKey, this.account, this.contractId, [nearlib.transactions.functionCall(
                            d.methodName,
                            Buffer.from(args),
                            gas || DEFAULT_FUNC_CALL_AMOUNT,
                            amount,
                        )]);

                        const result = getBorshTransactionLastResult(rawResult);
                        return result && deserialize(borshSchema, d.outputFieldType, result);
                    } catch (e) {
                        console.log('Failed: ', e);
                        throw e;
                    }
                },
            });
        });
    }

    async accessKeyInit () {
        await this.account.ready;

        this.accessKey = await this.account.findAccessKey();
        if (!this.accessKey) {
            // @ts-ignore
            throw new Error(`Can not sign transactions for account ${this.account.accountId}, no matching key pair found in Signer.`, 'KeyNotFound');
        }
    }
}

// @ts-ignore
const hexToBuffer = (hex) => Buffer.from(Web3.utils.hexToBytes(hex));
// @ts-ignore
const readerToHex = (len) => (reader) => Web3.utils.bytesToHex(reader.read_fixed_array(len));

exports.BorshContract = BorshContract;
exports.hexToBuffer = hexToBuffer;
exports.readerToHex = readerToHex;
// For debugging only.
exports.serialize = serialize;

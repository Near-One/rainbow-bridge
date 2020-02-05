const Web3 = require('web3');
const nearlib = require('nearlib');
const BN = require('bn.js');
const exec = require('child_process').exec;

const roots = require('./dag_merkle_roots.json');

function execute(command, callback){
    return new Promise(resolve => exec(command, (error, stdout, stderr) => {
        if (error) {
            console.log(error);
        }
        resolve(stdout);
    }));
};

function subscribeOnBlocksRangesFrom(web3, block_number, handler) {
    let inBlocksCallbacks = false;
    let last_block_number = block_number;

    web3.eth.subscribe("newBlockHeaders", async (error, event) => {
        if (error) {
            console.log(error);
            return;
        }

        if (!inBlocksCallbacks) {
            inBlocksCallbacks = true;

            let start = last_block_number;
            let stop = event.number;
            last_block_number = event.number;
            await handler(start, stop); //TODO

            inBlocksCallbacks = false;
        }
    });
}

function arrayPrefixU32Length(array) {
    return [
        Math.trunc(array.length) % 256,
        Math.trunc(array.length / 256) % 256,
        Math.trunc(array.length / 256 / 256) % 256,
        Math.trunc(array.length / 256 / 256 / 256) % 256,
    ].concat(...array);
}

const hexToBuffer = (hex) => Buffer.from(Web3.utils.hexToBytes(hex));
const readerToHex = (len) => (reader) => Web3.utils.bytesToHex(reader.read_fixed_array(len));

const borshSchema = {
    'initInput': {kind: 'struct', fields: [
            ['dags_start_epoch', 'u64'],
            ['dags_merkle_roots', ['H128']]
        ]},
    'H128': {kind: 'function', ser: hexToBuffer, deser: readerToHex(16) },
    'dagMerkleRootInput': { kind: 'struct', fields: [
            ['epoch', 'u64'],
        ]},
    'addBlockHeadersInput': { kind: 'struct', fields: [
            ['block_headers', [['u8']]],
            ['dag_nodes', [['DoubleNodeWithMerkleProof']]],
        ]},
    'DoubleNodeWithMerkleProof': { kind: 'struct', fields: [
            ['dag_nodes', ['H512']],
            ['proof', ['H128']],
        ]},
    'H512': {kind: 'function', ser: hexToBuffer, deser: readerToHex(64) },
};

function getBorshTransactionLastResult(txResult) {
    return txResult && Buffer.from(txResult.status.SuccessValue, 'base64');
}

function serializeField(schema, value, fieldType, writer) {
    if (fieldType === 'u8') {
        writer.write_u8(value);
    } else if (fieldType === 'u64') {
        writer.write_u64(value);
    } else if (fieldType instanceof Array) {
        if (typeof fieldType[0] === 'number') {
            if (value.length !== fieldType[0]) {
                throw new Error(`Expecting byte array of length ${fieldType[0]}, but got ${value.length} bytes`);
            }
            writer.write_fixed_array(value);
        } else {
            writer.write_array(value, (item) => { serializeField(schema, item, fieldType[0], writer); });
        }
    } else {
        const structSchema = schema[fieldType];
        if (!structSchema) {
            throw new Error(`Schema type ${fieldType} is missing in schema`);
        }
        if (structSchema.kind === 'struct') {
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


function deserializeField(schema, fieldType, reader) {
    if (fieldType === 'u8') {
        return reader.read_u8();
    } else if (fieldType === 'u64') {
        return reader.read_u64();
    } else if (fieldType === 'bool') {
        return !!reader.read_u8();
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
        if (structSchema.kind === 'struct') {
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
function serialize(schema, fieldType, obj) {
    if (fieldType === null) {
        return new Uint8Array();
    }
    const writer = new nearlib.utils.serialize.BinaryWriter();
    serializeField(schema, obj, fieldType, writer);
    return writer.toArray();
}


class BinaryReader {
    constructor(buf) {
        this.buf = buf;
        this.offset = 0;
    }

    read_u8() {
        const value = this.buf.readUInt8(this.offset);
        this.offset += 1;
        return value;
    }

    read_u32() {
        const value = this.buf.readUInt32LE(this.offset);
        this.offset += 4;
        return value;
    }

    read_u64() {
        const buf = this.read_buffer(8);
        return new BN(buf, 'le');
    }

    read_u128() {
        const buf = this.read_buffer(16);
        return new BN(buf, 'le');
    }

    read_buffer(len) {
        if ((this.offset + len) > this.buf.length) {
            throw new BorshError(`Expected buffer length ${len} isn't within bounds`);
        }
        const result = this.buf.slice(this.offset, this.offset + len);
        this.offset += len;
        return result;
    }

    read_string() {
        const len = this.read_u32();
        const buf = this.read_buffer(len);
        try {
            // NOTE: Using TextDecoder to fail on invalid UTF-8
            return textDecoder.decode(buf);
        } catch (e) {
            throw new BorshError(`Error decoding UTF-8 string: ${e}`);
        }
    }

    read_fixed_array(len) {
        return new Uint8Array(this.read_buffer(len));
    }

    read_array(fn) {
        const len = this.read_u32();
        const result = [];
        for (let i = 0; i < len; ++i) {
            result.push(fn());
        }
        return result;
    }
}

function deserialize(schema, fieldType, buffer) {
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

const signAndSendTransaction = async (account, receiverId, actions) => {
    await account.ready;

    // TODO: Find matching access key based on transaction
    const accessKey = await account.findAccessKey();
    if (!accessKey) {
        throw new Error(`Can not sign transactions for account ${account.accountId}, no matching key pair found in Signer.`, 'KeyNotFound');
    }

    const status = await account.connection.provider.status();

    const [txHash, signedTx] = await nearlib.transactions.signTransaction(
      receiverId, ++accessKey.nonce, actions, nearlib.utils.serialize.base_decode(status.sync_info.latest_block_hash), account.connection.signer, account.accountId, account.connection.networkId
    );
    console.log("TxHash", nearlib.utils.serialize.base_encode(txHash));

    let result;
    try {
        result = await account.connection.provider.sendTransaction(signedTx);
    } catch (error) {
        throw error;
    }

    if (result.status.Failure) {
        throw new Error(JSON.stringify(result.status.Failure))
    }

    return result;
}

class Contract {
    constructor(account, contractId, options) {
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
                        this.account.printLogs(contractId, result.logs);
                    }
                    return result.result && result.result.length > 0 && deserialize(borshSchema, d.outputFieldType, Buffer.from(result.result));
                }
            });
        });
        options.changeMethods.forEach((d) => {
            Object.defineProperty(this, d.methodName, {
                writable: false,
                enumerable: true,
                value: async (args, gas, amount) => {
                    args = serialize(borshSchema, d.inputFieldType, args);
                    try {
                        const rawResult = await signAndSendTransaction(account, contractId, [nearlib.transactions.functionCall(
                          d.methodName,
                          Buffer.from(args),
                          gas || DEFAULT_FUNC_CALL_AMOUNT,
                          amount
                        )]);

                        const result = getBorshTransactionLastResult(rawResult);
                        return result && deserialize(borshSchema, d.outputFieldType, result);
                    } catch (e) {
                        console.log("Failed: ", e);
                        throw e;
                    }
                }
            });
        });
    }
}

class EthBridgeContract extends Contract {
    constructor(account, contractId) {
        super(account, contractId, {
            viewMethods: [{
                methodName: "initialized",
                inputFieldType: null,
                outputFieldType: 'bool',
            }, {
                methodName: "dag_merkle_root",
                inputFieldType: "dagMerkleRootInput",
                outputFieldType: 'H128',
            }, {
                methodName: "last_block_number",
                inputFieldType: null,
                outputFieldType: 'u64',
            }],

            changeMethods: [{
                methodName: "init",
                inputFieldType: "initInput",
                outputFieldType: null,
            }, {
                methodName: "add_block_headers",
                inputFieldType: "addBlockHeadersInput",
                outputFieldType: null,
            }],
        })
    }
}

(async function () {

    const web3 = new Web3("wss://mainnet.infura.io/ws/v3/b5f870422ee5454fb11937e947154cd2");
    const near = await nearlib.connect({
        nodeUrl: 'http://localhost:3030', //'https://rpc.nearprotocol.com',
        networkId: 'local', // TODO: detect automatically
        deps: {
            keyStore: new nearlib.keyStores.UnencryptedFileSystemKeyStore(__dirname + '/neardev')
        }
    });


    const account = new nearlib.Account(near.connection, 'ethbridge');

    const ethBridgeContract = new EthBridgeContract(account, 'ethbridge');

    let initialized = await ethBridgeContract.initialized();
    if (!initialized) {
        console.log('EthBridge is not initialized, initializing...');
        await ethBridgeContract.init({
            dags_start_epoch: 0,
            dags_merkle_roots: roots.dag_merkle_roots
        }, new BN('1000000000000000'));
        console.log('EthBridge initialization finished');
    }

    console.log('EthBridge check initialization...');
    const first_root = await ethBridgeContract.dag_merkle_root({ epoch: 0 });
    const last_root = await ethBridgeContract.dag_merkle_root({ epoch: 511 });
    if (first_root === '0x55b891e842e58f58956a847cbbf67821' &&
        last_root === '0x4aa6ca6ebef942d8766065b2e590fd32')
    {
        console.log('EthBridge initialized properly');
    } else {
        console.log('EthBridge initialization ERROR!');
        return;
    }

    let last_block_number = (await ethBridgeContract.last_block_number()).toNumber();
    console.log("Contract block number is " + last_block_number);
    if (last_block_number == 0) {
        // Let's start bridge from current block since it is not initialized
        last_block_number = await web3.eth.getBlockNumber();
        console.log("Web3 block number is " + last_block_number);
    }

    const submitBlocks = async (blocks, start, stop) => {
        // Check bridge state, may be changed since computation could be long
        const last_block_number_onchain = await ethBridgeContract.last_block_number();
        console.log('ethBridgeContract.last_block_number =', last_block_number_onchain.toNumber());
        if (last_block_number_onchain >= stop) {
            console.log('Skipping submission due all were already submitted by someone');
            return;
        }
        if (last_block_number_onchain > start) {
            console.log('Trim first ${last_block_number_onchain - start} headers were due already submitted by someone');
            blocks = blocks.slice(last_block_number_onchain - start);
            start = last_block_number_onchain;
        }

        let timeBeforeSubmission = Date.now();
        console.log(`Submitting ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
        const args = {
            block_headers: blocks.map(block => web3.utils.hexToBytes(block.header_rlp)),
            dag_nodes: blocks.map(block => {
                const h512s = block.elements
                  .filter((_, index) => index % 2 === 0)
                  .map((element, index) => {
                      return web3.utils.padLeft(element, 64) + web3.utils.padLeft(block.elements[index*2 + 1], 64).substr(2)
                  });
                return h512s
                  .filter((_, index) => index % 2 === 0)
                  .map((element, index) => {
                      return {
                          dag_nodes: [element, h512s[index*2 + 1]],
                          proof: block.merkle_proofs.slice(
                            index * block.proof_length,
                            (index + 1) * block.proof_length,
                          ).map(leaf => web3.utils.padLeft(leaf, 32))
                      };
                  });
            })
        };
        await ethBridgeContract.add_block_headers(args, new BN('1000000000000000'));
        console.log(
          "Blocks submission took " + Math.trunc((Date.now() - timeBeforeSubmission)/10)/100 + "s " +
          "(" + Math.trunc((Date.now() - timeBeforeSubmission)/(blocks.length - 1)/10)/100 + "s per header)"
        );
        console.log(`Successfully submitted ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
    };

    subscribeOnBlocksRangesFrom(web3, last_block_number, async (start, stop) => {
        let blocks = [];
        let timeBeforeProofsComputed = Date.now();
        console.log(`Need to collect ${stop - start + 1} proofs from #${start} to #${stop}`);
        let shouldStop = false;
        for (let i = start; !shouldStop && i <= stop; ) {
            const N = blocks ? 2 : 3;
            console.log(`Computing for blocks #${i} to #${i + N - 1}`)
            let submitAmount = 0;
            const promises = [];
            for (; submitAmount < N; submitAmount++) {
                const ind = i + submitAmount;
                if (Math.trunc(i/30000) == Math.trunc(ind/30000)) {
                    // TODO: remove await and figureout why ethashproof fails even for same epoch
                    promises.push(await execute(`./ethashproof/cmd/relayer/relayer ${ind} | sed -e '1,/Json output/d'`));
                } else {
                    break;
                }
            }
            blocks = blocks.slice(blocks.length - 1).concat((await Promise.all(promises)).map(JSON.parse));
            
            submitBlocks(blocks, i, i + submitAmount - 1).catch(() => {
                shouldStop = true;
            })
            i += submitAmount;
        }
        console.log(
            "Proofs computation took " + Math.trunc((Date.now() - timeBeforeProofsComputed)/10)/100 + "s " +
            "(" + Math.trunc((Date.now() - timeBeforeProofsComputed)/(stop - start + 1)/10)/100 + "s per header)"
        );
    });
})()

const Web3 = require('web3');
const nearlib = require('nearlib');
const BN = require('bn.js');
const exec = require('child_process').exec;
const blockFromRpc = require('ethereumjs-block/from-rpc')
const utils = require('ethereumjs-util');

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

        if (!inBlocksCallbacks && event.number - last_block_number > 4) {
            inBlocksCallbacks = true;

            let start = last_block_number;
            let stop = event.number - 2;
            last_block_number = stop;
            await handler(start, stop);

            inBlocksCallbacks = false;
        }
    });
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

const signAndSendTransaction = async (accessKey, account, receiverId, actions) => {
    // TODO: Find matching access key based on transaction
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

    const flatLogs = [result.transaction_outcome, ...result.receipts_outcome].reduce((acc, it) => acc.concat(it.outcome.logs), []);
    if (flatLogs) {
        console.log(flatLogs);
    }

    if (result.status.Failure) {
        throw new Error(JSON.stringify(result.status.Failure))
    }

    return result;
}

function getBorshTransactionLastResult(txResult) {
    return txResult && Buffer.from(txResult.status.SuccessValue, 'base64');
}

class Contract {
    constructor(borshSchema, account, contractId, options) {
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
                        const rawResult = await signAndSendTransaction(this.accessKey, this.account, this.contractId, [nearlib.transactions.functionCall(
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

    async accessKeyInit() {
        await this.account.ready;

        this.accessKey = await this.account.findAccessKey();
        if (!this.accessKey) {
            throw new Error(`Can not sign transactions for account ${this.account.accountId}, no matching key pair found in Signer.`, 'KeyNotFound');
        }

    }
}

const hexToBuffer = (hex) => Buffer.from(Web3.utils.hexToBytes(hex));
const readerToHex = (len) => (reader) => Web3.utils.bytesToHex(reader.read_fixed_array(len));

const borshSchema = {
    'bool': {
        kind: 'function',
        ser: (b) => Buffer.from(Web3.utils.hexToBytes(b ? '0x01' : '0x00')),
        deser: (z) => readerToHex(1)(z) === '0x01'
    },
    'initInput': {kind: 'struct', fields: [
            ['validate_ethash', 'bool'],
            ['dags_start_epoch', 'u64'],
            ['dags_merkle_roots', ['H128']]
        ]},
    'dagMerkleRootInput': { kind: 'struct', fields: [
            ['epoch', 'u64'],
        ]},
    'addBlockHeaderInput': { kind: 'struct', fields: [
            ['block_header', ['u8']],
            ['dag_nodes', ['DoubleNodeWithMerkleProof']],
        ]},
    'DoubleNodeWithMerkleProof': { kind: 'struct', fields: [
            ['dag_nodes', ['H512']],
            ['proof', ['H128']],
        ]},
    'H128': {kind: 'function', ser: hexToBuffer, deser: readerToHex(16) },
    'H256': {kind: 'function', ser: hexToBuffer, deser: readerToHex(32) },
    'H512': {kind: 'function', ser: hexToBuffer, deser: readerToHex(64) },
    '?H256': {kind: 'option', type: 'H256'}
};

class EthBridgeContract extends Contract {
    constructor(account, contractId) {
        super(borshSchema, account, contractId, {
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
            }, {
                methodName: "block_hash",
                inputFieldType: "u64",
                outputFieldType: '?H256',
            }, {
                methodName: "block_hash_safe",
                inputFieldType: "u64",
                outputFieldType: '?H256',
            }],

            changeMethods: [{
                methodName: "init",
                inputFieldType: "initInput",
                outputFieldType: null,
            }, {
                methodName: "add_block_header",
                inputFieldType: "addBlockHeaderInput",
                outputFieldType: null,
            }],
        })
    }
}

function web3BlockToRlp(blockData) {
    blockData.difficulty = parseInt(blockData.difficulty, 10);
    blockData.totalDifficulty = parseInt(blockData.totalDifficulty, 10);
    blockData.uncleHash = blockData.sha3Uncles;
    blockData.coinbase = blockData.miner;
    blockData.transactionTrie = blockData.transactionsRoot;
    blockData.receiptTrie = blockData.receiptsRoot;
    blockData.bloom = blockData.logsBloom;
    const blockHeader = blockFromRpc(blockData);
    return utils.rlp.encode(blockHeader.header.raw);
}

(async function () {
    const web3 = new Web3(process.env.ETHEREUM_NODE_URL);
    const near = await nearlib.connect({
        nodeUrl: process.env.NEAR_NODE_URL, // 'https://rpc.nearprotocol.com',
        networkId: process.env.NEAR_NODE_NETWORK_ID,
        deps: {
            keyStore: new nearlib.keyStores.UnencryptedFileSystemKeyStore(__dirname + '/neardev')
        }
    });

    const account = new nearlib.Account(near.connection, process.env.NEAR_RELAYER_ACCOUNT_ID);

    const ethBridgeContract = new EthBridgeContract(account, process.env.NEAR_ETHBRIDGE_ACCOUNT_ID);
    await ethBridgeContract.accessKeyInit();

    let initialized = false;
    try {
        initialized = await ethBridgeContract.initialized();
    } catch (e) {
        // I guess not
    }
    if (!initialized) {
        console.log('EthBridge is not initialized, initializing...');
        await ethBridgeContract.init({
            validate_ethash: process.env.BRIDGE_VALIDATE_ETHASH === 'true',
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

    const submitBlock = async (block, blockNumber) => {
        let sleepTimer = 1;
        const maxSleepTime = 10;
        const sleep = async () => {
            await new Promise((resolve, reject) => {
                setTimeout(resolve, sleepTimer * 1000);
            });
            if (sleepTimer < maxSleepTime) {
                sleepTimer += 1;
            };
        }
        let ok = false;
        for (let iters = 0; iters < 20; ++iters) {
            try {
                let last_block_number_onchain = (await ethBridgeContract.last_block_number()).toNumber();
                if (last_block_number_onchain > 0 && last_block_number_onchain < blockNumber - 1) {
                    console.log(`Sleeping ${sleepTimer} sec. The latest block on chain is ${last_block_number_onchain}, but need to submit block #${blockNumber}`);
                    await sleep();
                } else {
                    ok = true;
                    break;
                }
            } catch (e) {
                console.log("Block awaiting failed :(", e);
                await sleep();
            }
        }
        if (!ok) {
            process.exit(1);
        }

        // Check bridge state, may be changed since computation could be long
        let timeBeforeSubmission = Date.now();
        console.log(`Submitting block ${blockNumber} to EthBridge`);
        const h512s = block.elements
            .filter((_, index) => index % 2 === 0)
            .map((element, index) => {
                return web3.utils.padLeft(element, 64) + web3.utils.padLeft(block.elements[index*2 + 1], 64).substr(2)
            });

        const args = {
            block_header: web3.utils.hexToBytes(block.header_rlp),
            dag_nodes: h512s
                .filter((_, index) => index % 2 === 0)
                .map((element, index) => {
                    return {
                        dag_nodes: [element, h512s[index*2 + 1]],
                        proof: block.merkle_proofs.slice(
                            index * block.proof_length,
                            (index + 1) * block.proof_length,
                        ).map(leaf => web3.utils.padLeft(leaf, 32))
                    };
                }),
        };

        for (let i = 0; i < 10; ++i) {
            try {
                await ethBridgeContract.add_block_header(args, new BN('1000000000000000'));
                console.log(
                    "Blocks submission took " + Math.trunc((Date.now() - timeBeforeSubmission)/10)/100 + "s " +
                    "(" + Math.trunc((Date.now() - timeBeforeSubmission)/10)/100 + "s per header)"
                );
                console.log(`Successfully submitted block ${blockNumber} to EthBridge`);
                return;
            } catch (e) {
                // failed
                console.log(`Sleeping 0.5sec. Failed at iteration #${i}:`, e);
                await new Promise((resolve, reject) => {
                    setTimeout(resolve, 500);
                });
            }
        }

        throw new Error("Failed to submit a block");
    };

    subscribeOnBlocksRangesFrom(web3, last_block_number, async (start, stop) => {
        let timeBeforeProofsComputed = Date.now();
        console.log(`Processing ${stop - start + 1} blocks from #${start} to #${stop}`);
        for (let i = start; i <= stop; ++i) {
            let ok = false;
            for (let retryIter = 0; retryIter < 10; retryIter++) {
                try {
                    const blockRlp = web3.utils.bytesToHex(web3BlockToRlp(await web3.eth.getBlock(i)));
                    const block = JSON.parse(await execute(`./ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`));
                    submitBlock(block, i).catch((e) => {
                        console.error(e);
                        process.exit(2);
                    })
                    ok = true;
                    break;
                } catch (e) {
                    console.log(`Sleeping 0.5sec. Failed at iteration #${retryIter}:`, e);
                    await new Promise((resolve, reject) => {
                        setTimeout(resolve, 500);
                    });
                }
            }
            if (!ok) {
                console.error(`Failed to create a proof for a block #${i}`);
                process.exit(3);
            }
        }
        console.log(
            "Proofs computation took " + Math.trunc((Date.now() - timeBeforeProofsComputed)/10)/100 + "s " +
            "(" + Math.trunc((Date.now() - timeBeforeProofsComputed)/(stop - start + 1)/10)/100 + "s per header)"
        );
    });
})()

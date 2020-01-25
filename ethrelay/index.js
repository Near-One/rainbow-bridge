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
            await handler(start, stop);

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

    const ethBridgeContract = new nearlib.Contract(account, 'ethbridge', {
        viewMethods: ["initialized", "dag_merkle_root", "last_block_number"],
        changeMethods: ["init", "add_block_headers"],
    });

    let initialized = await ethBridgeContract.initialized();
    if (!initialized) {
        console.log('EthBridge is not initialized, initializing...');
        await ethBridgeContract.init({
            dags_start_epoch: 0,
            dags_merkle_roots: roots.dag_merkle_roots
        }, new BN('1000000000000000000'));
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

    let last_block_number = await ethBridgeContract.last_block_number();
    if (last_block_number === 0) {
        // Let's start bridge from current block since it is not initialized
        last_block_number = await web3.eth.getBlockNumber();
    }

    subscribeOnBlocksRangesFrom(web3, last_block_number, async (start, stop) => {
        let blocks = [];
        for (let i = start; i <= stop; i++) {
            console.log(`Computing for block #${i}`)
            const res = await execute(`./ethashproof/cmd/relayer/relayer ${i} | sed -e '1,/Json output/d'`);
            blocks.push(JSON.parse(res));
        }

        // Check bridge state, may be changed since computation could be long
        const last_block_number_onchain = await ethBridgeContract.last_block_number();
        console.log('ethBridgeContract.last_block_number =', last_block_number_onchain);
        if (last_block_number_onchain >= stop) {
            console.log('Skipping submission due all were already sumbitted by someone');
            return;
        }
        if (last_block_number_onchain > start) {
            console.log('Trim first ${last_block_number_onchain - start} headers were due already submitted by someone');
            blocks = blocks.slice(last_block_number_onchain - start);
            start = last_block_number_onchain;
        }

        console.log(`Submitting ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
        await ethBridgeContract.add_block_headers({
            block_headers: blocks.map(block => arrayPrefixU32Length(web3.utils.hexToBytes(block.header_rlp))),
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
        }, new BN('1000000000000000000'));
        console.log(`Successfully submitted ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
    });

    //console.log(await web3.eth.getBlockNumber());
    //console.log(nearlib);
})()

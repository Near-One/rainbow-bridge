const Web3 = require('web3');
const nearlib = require('nearlib');
const BN = require('bn.js');
const exec = require('child_process').exec;
const fs = require('fs');
const path = require('path');

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
            await handler(start, start + 1);

            inBlocksCallbacks = false;
        }
    });
}

(async function () {

    const web3 = new Web3("http://localhost:9545");
    const near = await nearlib.connect({
        nodeUrl: 'http://localhost:3030', //'https://rpc.nearprotocol.com',
        networkId: 'local', // TODO: detect automatically
        deps: {
            keyStore: new nearlib.keyStores.UnencryptedFileSystemKeyStore(__dirname + '/neardev')
        }
    });

    // Import private key from ENV
    if ((await web3.eth.getAccounts()).length == 0 &&
        process.env.NEAR_BRIDGE_OWNER_PRIVATE_KEY)
    {
        const acc = web3.eth.accounts.privateKeyToAccount(process.env.NEAR_BRIDGE_OWNER_PRIVATE_KEY);
        web3.eth.accounts.wallet.add(acc);
        web3.eth.defaultAccount = acc.address;
    }
    const account = (await web3.eth.getAccounts())[0];
    let nonce = await web3.eth.getTransactionCount(account);
    
    console.log('yyy');

    let nearBridgeContract = new web3.eth.Contract(
        JSON.parse(fs.readFileSync(path.join(__dirname, '../nearbridge/NearBridge.full.abi'))),
        process.env.NEAR_BRIDGE_SMART_CONTRACT_ADDRESS,
        {
            from: account,
        }
    );

    if (!process.env.NEAR_BRIDGE_SMART_CONTRACT_ADDRESS) {
        console.log('Deploying NearBridge smart contract');
        nearBridgeContract = await nearBridgeContract.deploy({
            data: '0x' + fs.readFileSync(path.join(__dirname, '../nearbridge/NearBridge.full.bin'))
        }).send({
            from: account,
            gas: 1000000
        });
    }

    const status = await near.connection.provider.status();
    console.log('near.status', status);
    let lastBlockNumber = status.sync_info.latest_block_height;

    return;

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

    let last_block_number = 2;//await ethBridgeContract.last_block_number();
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
        console.log('block_headers', JSON.stringify(
            blocks.map(block => arrayPrefixU32Length(web3.utils.hexToBytes(block.header_rlp)))
        ));
        console.log('dag_nodes:', JSON.stringify(
            arrayPrefixU32Length(blocks.map(block => {
                const h512s = block.elements
                    .filter((_, index) => index % 2 === 0)
                    .map((element, index) => {
                        return web3.utils.padLeft(element, 64) + web3.utils.padLeft(block.elements[index*2 + 1], 64).substr(2)
                    });
                return arrayPrefixU32Length(
                    h512s
                    .filter((_, index) => index % 2 === 0)
                    .map((element, index) => {
                        return {
                            dag_nodes: arrayPrefixU32Length([web3.utils.hexToBytes(element), web3.utils.hexToBytes(h512s[index*2 + 1])]),
                            proof: arrayPrefixU32Length(block.merkle_proofs.slice(
                                index * block.proof_length,
                                (index + 1) * block.proof_length,
                            ).map(leaf => web3.utils.padLeft(leaf, 32)))
                        };
                    })
                );
            }))
        ));
        
        const h512s = blocks[0].elements
                    .filter((_, index) => index % 2 === 0)
                    .map((element, index) => {
                        return web3.utils.padLeft(element, 64) + web3.utils.padLeft(blocks[0].elements[index*2 + 1], 64).substr(2)
                    });

        console.log('xxx', JSON.stringify({
            dag_nodes: arrayPrefixU32Length([web3.utils.hexToBytes(h512s[0]), web3.utils.hexToBytes(h512s[1])]),
            proof: arrayPrefixU32Length(blocks[0].merkle_proofs.slice(
                0 * blocks[0].proof_length,
                (0 + 1) * blocks[0].proof_length,
            ).map(leaf => web3.utils.padLeft(leaf, 32)))
        }));

        console.log('!!!!', web3.utils.hexToBytes(web3.utils.padLeft(blocks[0].merkle_proofs[0], 32)));

        await ethBridgeContract.add_block_headers({
            //block_headers: blocks.map(block => arrayPrefixU32Length(web3.utils.hexToBytes(block.header_rlp))),

            // dag_nodes: arrayPrefixU32Length(blocks.map(block => {
            //     const h512s = block.elements
            //         .filter((_, index) => index % 2 === 0)
            //         .map((element, index) => {
            //             return web3.utils.padLeft(element, 64) + web3.utils.padLeft(block.elements[index*2 + 1], 64).substr(2)
            //         });
            //     return arrayPrefixU32Length(
            //         h512s
            //         .filter((_, index) => index % 2 === 0)
            //         .map((element, index) => {
            //             return {
            //                 dag_nodes: arrayPrefixU32Length([web3.utils.hexToBytes(element), web3.utils.hexToBytes(h512s[index*2 + 1])]),
            //                 proof: arrayPrefixU32Length(block.merkle_proofs.slice(
            //                     index * block.proof_length,
            //                     (index + 1) * block.proof_length,
            //                 ).map(leaf => web3.utils.padLeft(leaf, 32)))
            //             };
            //         })
            //     );
            // }))

            // dag_nodes: {
            //     dag_nodes: arrayPrefixU32Length([web3.utils.hexToBytes(h512s[0]), web3.utils.hexToBytes(h512s[1])]),
            //     proof: arrayPrefixU32Length(blocks[0].merkle_proofs.slice(
            //         0 * blocks[0].proof_length,
            //         (0 + 1) * blocks[0].proof_length,
            //     ).map(leaf => web3.utils.padLeft(leaf, 32)))
            // }

            dag_nodes: web3.utils.hexToBytes(web3.utils.padLeft(blocks[0].merkle_proofs[0], 32))
        }, new BN('1000000000000000000'));
        console.log(`Successfully submitted ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
    });

    //console.log(await web3.eth.getBlockNumber());
    //console.log(nearlib);
})()

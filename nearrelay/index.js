const Web3 = require('web3');
const nearlib = require('nearlib');
const BN = require('bn.js');
const exec = require('child_process').exec;
const fs = require('fs');
const path = require('path');
const bs58 = require('bs58')

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
        nodeUrl: process.env.NEAR_NODE_URL, //'https://rpc.nearprotocol.com',
        networkId: process.env.NEAR_NODE_NETWORK_ID, // TODO: detect automatically
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

    let nearBridgeContract = new web3.eth.Contract(
        JSON.parse(fs.readFileSync(path.join(__dirname, '../nearbridge/NearBridge.full.abi'))),
        process.env.NEAR_BRIDGE_SMART_CONTRACT_ADDRESS,
        {
            from: account,
            handleRevert: true,
        }
    );

    if (!process.env.NEAR_BRIDGE_SMART_CONTRACT_ADDRESS) {
        console.log('Deploying NearBridge smart contract');
        nearBridgeContract = await nearBridgeContract.deploy({
            data: '0x' + fs.readFileSync(path.join(__dirname, '../nearbridge/NearBridge.full.bin'))
        }).send({
            from: account,
            gas: 3000000,
            handleRevert: true,
        });
        console.log('Deployed to address:', nearBridgeContract.address);
    }

    const checkNearStatus = async function () {
        let latest_submitted_block = Number(await nearBridgeContract.methods.lastBlockNumber().call());
        console.log('latest_submitted_block', typeof latest_submitted_block, latest_submitted_block);

        const status = await near.connection.provider.status();
        let lastNearBlock = status.sync_info.latest_block_height;
        console.log('lastNearBlock', typeof lastNearBlock, lastNearBlock);

        const promises = [];
        for (let i = latest_submitted_block; i < lastNearBlock; i += 10) {
            promises.push(near.connection.provider.block(i));
        }

        const blocks = (await Promise.all(promises)).map(block => {
            return [
                '0x',
                web3.utils.padLeft(block.header.height.toString(16), 16).match(/../g).reverse().join(''),
                web3.utils.padLeft(web3.utils.toHex(bs58.decode(block.header.epoch_id)).substr(2), 64),
                web3.utils.padLeft(web3.utils.toHex(bs58.decode(block.header.next_epoch_id)).substr(2), 64),
                web3.utils.padLeft(web3.utils.toHex(bs58.decode(block.header.prev_state_root)).substr(2), 64),
                web3.utils.padLeft(web3.utils.toHex(bs58.decode(block.header.outcome_root)).substr(2), 64),
                web3.utils.padLeft(block.header.timestamp.toString(16), 16).match(/../g).reverse().join(''),
                web3.utils.padLeft(web3.utils.toHex(bs58.decode(block.header.next_bp_hash)).substr(2), 64),
            ].join('');
        });

        // TODO: Investigate how to use new feature web3.eth.handleRevert
        try {
            console.log(`Submitting ${blocks.length} blocks`);
            const tx = await nearBridgeContract.methods.addBlockHeaders(blocks).send({ gas: 5000000 });
            console.log('Sumbitted!');
        } catch (txRevertMessage) {
            const err = txRevertMessage.toString();
            const receipt = JSON.parse(err.substr(err.indexOf('{')));
            const tx = await web3.eth.getTransaction(receipt.transactionHash);
            try {
                await web3.eth.call(tx, tx.blockNumber);
            } catch (callRevertReason) {
                const err = callRevertReason.toString();
                console.log('Reverted! Reason:', err.substr(err.lastIndexOf(':') + 2));
            }
        }

        console.log('Sleep for 10 seconds');
        setTimeout(checkNearStatus, 10000);
    };

    checkNearStatus();
})()

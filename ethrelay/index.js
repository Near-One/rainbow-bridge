const Web3 = require('web3');
const nearlib = require('nearlib');
const exec = require('child_process').exec;

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

(async function () {

    const web3 = new Web3("wss://mainnet.infura.io/ws");
    const near = await nearlib.connect({
        nodeUrl: 'https://rpc.nearprotocol.com',
        deps: {
            keyStore: new nearlib.keyStores.UnencryptedFileSystemKeyStore()
        }
    });

    // TODO: Get las sumbitted block from EthBridge
    let last_block_number = 9186015;

    subscribeOnBlocksRangesFrom(web3, last_block_number, async (start, stop) => {
        console.log(start, stop);

        const blocks = [];
        for (let i = start; i <= stop; i++) {
            console.log(`Computing for block #${i}`)
            const res = await execute(`./ethashproof/cmd/relayer/relayer ${i} | sed -e '1,/Json output/d'`);
            blocks.push(JSON.parse(res));
        }

        // TODO: Submit blocks with proofs
        console.log(`Submited ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
    });

    //console.log(await web3.eth.getBlockNumber());
    //console.log(nearlib);
})()

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
        nodeUrl: 'http://localhost:3030', //'https://rpc.nearprotocol.com',
        deps: {
            keyStore: new nearlib.keyStores.UnencryptedFileSystemKeyStore()
        }
    });

    const account = new nearlib.Account(near.connection, 'ethrelay');

    const ethBridgeContract = new nearlib.Contract(account, 'ethbridge', {
        viewMethods: ["last_block_number", "add_block_headers"],
        changeMethods: [],
    });

    let last_block_number = await ethBridgeContract.last_block_number();
    if (last_block_number === 0) {
        // Let's start bridge from current block since it is not initialized
        last_block_number = await web3.eth.getBlockNumber();
    }

    subscribeOnBlocksRangesFrom(web3, last_block_number, async (start, stop) => {
        console.log(start, stop);

        const blocks = [];
        for (let i = start; i <= stop; i++) {
            console.log(`Computing for block #${i}`)
            const res = await execute(`./ethashproof/cmd/relayer/relayer ${i} | sed -e '1,/Json output/d'`);
            blocks.push(JSON.parse(res));
        }

        console.log(`Submiting ${blocks.length} blocks from ${start} to ${stop} to EthBridge`);
        await ethBridgeContract.add_block_headers({
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
                            dag_nodes: [web3.utils.hexToBytes(element), web3.utils.hexToBytes(h512s[index*2 + 1])],
                            proof: block.merkle_proofs.slice(
                                Math.trunc(index) * block.proof_length,
                                Math.trunc(index + 1) * block.proof_length,
                            ).map(leaf => web3.utils.padLeft(leaf, 64))
                        };
                    });
            })

            // pub fn to_double_node_with_merkle_proof_vec(&self) -> Vec<DoubleNodeWithMerkleProof> {
            //     let h512s = Self::combine_dag_h256_to_h512(self.elements.clone());
            //     h512s.iter().zip(h512s.iter().skip(1)).enumerate().filter(|(i,_)| {
            //         i % 2 == 0
            //     }).map(|(i,(a,b))| {
            //         DoubleNodeWithMerkleProof {
            //             dag_nodes: vec![*a, *b],
            //             proof: self.merkle_proofs[i/2 * self.proof_length as usize .. (i/2 + 1) * self.proof_length as usize].to_vec(),
            //         }
            //     }).collect()
            // }

            // pub struct DoubleNodeWithMerkleProof {
            //     pub dag_nodes: Vec<H512>, // [H512; 2]
            //     pub proof: Vec<H128>,
            // }

            // fn combine_dag_h256_to_h512(elements: Vec<H256>) -> Vec<H512> {
            //     elements.iter().zip(elements.iter().skip(1)).enumerate().filter(|(i,_)| {
            //         i % 2 == 0
            //     }).map(|(_,(a,b))| {
            //         let mut buffer = [0u8; 64];
            //         buffer[..32].copy_from_slice(&(a.0).0);
            //         buffer[32..].copy_from_slice(&(b.0).0);
            //         H512(buffer.into())
            //     }).collect()
            // }
        });
    });

    //console.log(await web3.eth.getBlockNumber());
    //console.log(nearlib);
})()

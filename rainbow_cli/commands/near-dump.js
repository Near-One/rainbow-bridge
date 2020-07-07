const fs = require('fs').promises;
const Path = require('path');
const { RainbowConfig } = require('../lib/config');
const fetch = require('node-fetch');

async function getLatestBlock (nearNodeUrl) {
    const resp = await fetch(nearNodeUrl, {
        method: 'post',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            jsonrpc: '2.0',
            id: 'dontcare',
            method: 'block',
            params: {
                finality: 'final',
            },
        }),
    });
    const data = await resp.json();
    return data.result;
}

async function getNextLightClientBlock (nearNodeUrl, blockHash) {
    const resp = await fetch(nearNodeUrl, {
        method: 'post',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            jsonrpc: '2.0',
            id: 'dontcare',
            method: 'next_light_client_block',
            params: [blockHash],
        }),
    });
    const data = await resp.json();
    return data.result;
}

class NearDump {
    static async execute (kindOfData, { path, numBlocks }) {
        if (kindOfData !== 'headers') {
            console.log('Usage: node index.js near-dump headers');
            process.exit(2);
        }

        if (!numBlocks) {
            numBlocks = 100;
        }
        const nearNodeUrl = RainbowConfig.getParam('near-node-url');

        let latestBlock = await getLatestBlock(nearNodeUrl);
        console.log(`Downloading ${numBlocks} light client blocks start from ${latestBlock.header.height}`);

        let newLatestBlock;
        while (numBlocks > 0) {
            newLatestBlock = await getLatestBlock(nearNodeUrl);
            if (newLatestBlock.header.height > latestBlock.header.height) {
                console.log(`Got new block at height ${newLatestBlock.header.height}`);
                let block;
                do {
                    block = await getNextLightClientBlock(nearNodeUrl, newLatestBlock.header.hash);
                } while (!block);
                console.log(`Got new light client block at height ${block.inner_lite.height}`);
                await NearDump.saveBlock(block.inner_lite.height, block, path);
                latestBlock = newLatestBlock;
                numBlocks--;
            } else {
                continue;
            }
        }
    }

    static async saveBlock (i, block, path) {
        const file = Path.join(path, `${i}.json`);
        await fs.writeFile(file, JSON.stringify(block));
    }
}

exports.NearDump = NearDump;

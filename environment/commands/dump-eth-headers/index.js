const Web3 = require('web3');
const Path = require('path');
const fs = require('fs').promises;
const {web3BlockToRlp, execute} = require('../../lib/eth-relay');

class DumpETHHeaders {
  static async execute({path, startBlock, endBlock, ethNodeUrl}) {
    let web3 = new Web3(ethNodeUrl);
    if (!endBlock) {
      endBlock = await web3.eth.getBlockNumber();
    } else {
      endBlock = Number(endBlock);
    }
    if (!startBlock) {
      startBlock = Math.max(0, Number(endBlock) - 43000 + 1);
    } else if (Number(startBlock)<0) {
      startBlock = Math.max(0, Number(endBlock) + Number(startBlock) + 1)
    } else {
      startBlock = Number(startBlock);
    }
    console.log(`Downloading block ${endBlock} down to ${startBlock} to ${path}. ${endBlock - startBlock + 1} blocks in total.`)

    for (let b = endBlock; b >= startBlock; b--) {
      console.log(`Downloading block ${b}`);
      const blockRlp = web3.utils.bytesToHex(web3BlockToRlp(await web3.eth.getBlock(b)));
      console.log(`Processing block ${b}`);
      const unparsedBlock = await execute(`./vendor/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`);
      const block = JSON.parse(unparsedBlock);
      DumpETHHeaders.saveBlock(b, block, path);
    }

    try {
      // Only WebSocket provider can close.
      web3.currentProvider.connection.close();
    } catch (e) {
    }
  }

  static async saveBlock(i, block, path) {
    let file = Path.join(path, `${i}.json`);
    await fs.writeFile(file, JSON.stringify(block));
  }
}

exports.DumpETHHeaders = DumpETHHeaders;

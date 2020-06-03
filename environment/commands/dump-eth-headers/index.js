const Web3 = require('web3');
const Path = require('path');
const fs = require('fs').promises;

class DumpETHHeaders {
  static async execute({path, startBlock, endBlock, ethNodeUrl}) {
    let web3 = new Web3(ethNodeUrl);
    if (!endBlock) {
      endBlock = await web3.eth.getBlockNumber();
    }
    if (!startBlock) {
      startBlock = Math.max(0, endBlock - 43000);
    }

    for (let b = startBlock; b <= endBlock; b++) {
      console.log(`Downloading block ${b}`);
      let block = await web3.eth.getBlock(b);
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

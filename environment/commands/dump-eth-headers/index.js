const {
    EthClientSetup
} = require('../../lib/eth-client-setup');
const {
    EthRelay
} = require('../../lib/eth-relay');

class DumpETHHeaders {
  static async execute(path, start_block, end_block) { 
    const setup = new EthClientSetup();
    await setup.initialize();
    const ethRelay = new EthRelay();
    ethRelay.initialize(setup.ethClientContract, process.env.ETH_NODE_URL, {mode: 'download_only', path: path});
    await ethRelay.run();
  }
}

exports.DumpETHHeaders = DumpETHHeaders;

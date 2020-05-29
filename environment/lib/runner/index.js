const {EthClientSetup} = require('../eth-client-setup');
const {EthRelay} = require('../eth-relay');
const {EthProofExtractor} = require('../eth-proof-extractor');
const {EthProverTester} = require('../eth-prover-tester');

class RainbowRunner {
  async run(service) {
    switch (service) {
    case 'eth_client_setup': {
      const setup = new EthClientSetup();
      await setup.initialize();
      break;
    }
    case 'start_ethrelay': {
      const setup = new EthClientSetup();
      await setup.initialize();
      const ethRelay = new EthRelay();
      ethRelay.initialize(setup.ethClientContract, process.env.ETH_NODE_URL);
      await ethRelay.run();
      break;
    }
    case 'extract_proof': {
      const ethProofExtractor = new EthProofExtractor();
      ethProofExtractor.initialize(process.env.ETH_NODE_URL);
      await ethProofExtractor.debugPrint(process.env.TX_HASH);
      ethProofExtractor.destroy()
      break;
    }
    case 'test_ethprover': {
      const setup = new EthClientSetup();
      await setup.initialize();
      const tester =
          new EthProverTester(process.env.ETH_NODE_URL, setup.ethClientContract,
                              setup.ethProverContract);
      await tester.run();
      tester.destroy();
      break;
    }
    default: {
      console.log(`Unrecognized command ${service}`);
      process.exit(1);
    }
    }
  }
}

exports.RainbowRunner = RainbowRunner;

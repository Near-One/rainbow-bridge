const { EthClientSetup } = require('./lib/eth-client-setup');
const { EthRelay } = require('./lib/eth-relay');
const { EthProofExtractor } = require('./lib/eth-proof-extractor');
const { EthProverTester } = require('./lib/eth-prover-tester');

(async function () {
    switch (process.argv[2]) {
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
            const tester = new EthProverTester(process.env.ETH_NODE_URL, setup.ethClientContract, setup.ethProverContract);
            console.log("TESTING");
            await tester.run();
            tester.destroy();
            console.log("DONE");
            break;
        }
        default: {
            console.log(`Unrecognized command ${process.argv}`);
            process.exit(1);
        }
    }
})()

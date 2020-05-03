const {EthClientSetup} = require('./lib/eth-client-setup');
const {EthRelay} = require('./lib/eth-relay');

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
        default: {
            console.log(`Unrecognized command ${process.argv}`);
            process.exit(1);
        }
    }
})()

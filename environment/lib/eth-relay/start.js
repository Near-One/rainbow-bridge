const {
    EthClientSetup,
} = require('../eth-client-setup');
const {
    EthRelay,
} = require('../eth-relay');

(async () => {
    const setup = new EthClientSetup();
    await setup.initialize();
    const ethRelay = new EthRelay();
    ethRelay.initialize(setup.ethClientContract, process.env.ETH_NODE_URL);
    await ethRelay.run();
})().catch(e => { console.log(e); });

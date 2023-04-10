/* eslint-disable indent */
/* eslint semi: ["error", "always"] */

const fs = require('fs');

const {
    Eth2NearRelay
} = require('./index');

// Add your web3 endpoint url here
const WEB3_ENDPOINT_URL = 'YOUR_ENDPOINT_HERE';

async function main () {
    const args = process.argv.slice(2);
    const blockNumber = parseInt(args[0], 10);
    const relay = new Eth2NearRelay();
    relay.initialize('', { ethNodeUrl: WEB3_ENDPOINT_URL, gasPerTransaction: '1', totalSubmitBlock: 0, metricsPort: 0, nearNetworkId: 'mainnet' });
    const res = await relay.getParseBlock(blockNumber);
    fs.writeFileSync(`${__dirname}/../../contracts/near/eth-client/src/data/${blockNumber}.json`, JSON.stringify(res)); // eslint-disable-line
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });

/// This module gives a few utils for robust error handling, 
/// and wrap web3 with error handling and retry
const Web3 = require('web3');

const RETRY = 10;
const DELAY = 500;
const BACKOFF = 1.2;

const retry = (retries, fn) => fn().catch(err => retries > 1 ? retry(retries - 1, fn) : Promise.reject(err));
const sleep = (duration) => new Promise(res => setTimeout(res, duration));
const backoff = (retries, fn, delay = DELAY, wait = BACKOFF) =>
    fn().catch(err => retries > 1
        ? sleep(delay).then(() => backoff(retries - 1, fn, delay * wait))
        : Promise.reject(err));

class RobustWeb3 {
    constructor(ethNodeUrl) {
        this.ethNodeUrl = ethNodeUrl;
        this.web3 = new Web3(ethNodeUrl);
    }

    async getBlockNumber() {
        let self = this;
        await backoff(RETRY, async () => {
            try {
                return await self.web3.eth.getBlockNumber();
            } catch (e) {
                if (e && e.toString() === 'Error: connection not open') {
                    self.web3 = new Web3(self.ethNodeUrl);
                }
            }
        });
    }

    async getBlock(b) {
        let self = this;
        await backoff(RETRY, async () => {
            try {
                return await self.web3.eth.getBlock(b);
            } catch (e) {
                if (e && e.toString() === 'Error: connection not open') {
                    self.web3 = new Web3(self.ethNodeUrl);
                }
            }
        });
    }

    async getTransactionReceipt(t) {
        let self = this;
        await backoff(RETRY, async () => {
            try {
                return await self.web3.eth.getTransactionReceipt(t);
            } catch (e) {
                if (e && e.toString() === 'Error: connection not open') {
                    self.web3 = new Web3(self.ethNodeUrl);
                }
            }
        });
    }

    destroy() {
        if (this.web3.currentProvider.connection.close) { // Only WebSocket provider has close, HTTPS don't
            this.web3.currentProvider.connection.close();
        }
    }
}

function normalizeEthKey(key) {
    let result = key.toLowerCase();
    if (!result.startsWith('0x')) {
        result = '0x' + result;
    }
    return result;
}

module.exports = {
    retry,
    sleep,
    backoff,
    RobustWeb3,
    normalizeEthKey
}
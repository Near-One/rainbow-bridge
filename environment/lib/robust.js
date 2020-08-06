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

const SLOW_TX_ERROR_MSG = 'transaction not executed within 5 minutes';

class RobustWeb3 {
    constructor(ethNodeUrl) {
        this.ethNodeUrl = ethNodeUrl;
        this.web3 = new Web3(ethNodeUrl);
    }

    async getBlockNumber() {
        return await backoff(RETRY, async () => {
            try {
                return await this.web3.eth.getBlockNumber();
            } catch (e) {
                if (e && e.toString() === 'Error: connection not open') {
                    this.web3.setProvider(this.ethNodeUrl);
                }
            }
        });
    }

    async getBlock(b) {
        return await backoff(RETRY, async () => {
            try {
                return await this.web3.eth.getBlock(b);
            } catch (e) {
                if (e && e.toString() === 'Error: connection not open') {
                    this.web3.setProvider(this.ethNodeUrl);
                }
            }
        });
    }

    async getTransactionReceipt(t) {
        return await backoff(RETRY, async () => {
            try {
                return await this.web3.eth.getTransactionReceipt(t);
            } catch (e) {
                if (e && e.toString() === 'Error: connection not open') {
                    this.web3.setProvider(this.ethNodeUrl);
                }
            }
        });
    }

    async callContract(contract, method, args, options) {
        let gasPrice = await this.web3.eth.getGasPrice();
        let nonce = await this.web3.eth.getTransactionCount(options.from, 'pending');
        while (gasPrice < 10000 * 1e9) {
            try {
                // Keep sending with same nonce but higher gasPrice to override same txn
                let tx = {
                    from: options.from,
                    to: contract.options.address,
                    gas: Web3.utils.toHex(options.gas),
                    gasPrice: Web3.utils.toHex(gasPrice),
                    nonce: Web3.utils.toHex(nonce),
                    data: contract.methods[method](...args).encodeABI()
                };

                return await promiseWithTimeout(5 * 60 * 1000, this.web3.eth.sendTransaction(tx), SLOW_TX_ERROR_MSG);
            } catch (e) {
                if (e.message === SLOW_TX_ERROR_MSG) {
                    console.log(SLOW_TX_ERROR_MSG);
                    console.log(`current gasPrice: ${gasPrice}. resend tx with double gasPrice`)
                    gasPrice *= 2;
                } else if (e.message.indexOf("the tx doesn't have the correct nonce") >= 0) {
                    console.log('nonce error, retrying with new nonce');
                    nonce = await this.web3.eth.getTransactionCount(options.from, 'pending');
                } else if (e.toString() === 'Error: connection not open') {
                    console.log('web3 disconnected, reconnecting');
                    this.web3.setProvider(this.ethNodeUrl);
                } else {
                    throw e;
                }
            }
        }
        throw new Error('Cannot finish txn within 1e13 gas')
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

const promiseWithTimeout = (timeoutMs, promise, failureMessage) => {
    let timeoutHandle;
    const timeoutPromise = new Promise((resolve, reject) => {
        timeoutHandle = setTimeout(() => reject(new Error(failureMessage)), timeoutMs);
    });

    return Promise.race([
        promise,
        timeoutPromise,
    ]).then((result) => {
        clearTimeout(timeoutHandle);
        return result;
    });
}

module.exports = {
    retry,
    sleep,
    backoff,
    RobustWeb3,
    normalizeEthKey,
    promiseWithTimeout
}
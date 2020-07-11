/// This module gives a few utils for robust error handling, 
/// and wrap web3 with error handling and retry

const RETRY = 10;
const DELAY = 500;
const BACKOFF = 1.2;

const retry = (retries, fn) => fn().catch(err => retries > 1 ? retry(retries - 1, fn) : Promise.reject(err));
const sleep = (duration) => new Promise(res => setTimeout(res, duration));
const backoff = (retries, fn, delay = DELAY, wait = BACKOFF) =>
    fn().catch(err => retries > 1
        ? sleep(delay).then(() => backoff(retries - 1, fn, delay * wait))
        : Promise.reject(err));

const web3GetBlockNumber = async (web3) => await backoff(RETRY, async () => await web3.eth.getBlockNumber());
const web3GetBlock = async (web3, b) => await backoff(RETRY, async () => await web3.eth.getBlock(b));

module.exports = {
    retry,
    sleep,
    backoff,
    web3GetBlockNumber,
    web3GetBlock
}
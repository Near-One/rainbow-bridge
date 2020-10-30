const { nearAPI, Web3, RobustWeb3, normalizeEthKey, sleep } = require('./robust')
const { RainbowConfig } = require('./config')
const {
  txnStatus,
  BorshContract,
  hexToBuffer,
  readerToHex
} = require('./borsh')
const {
  setupEthNear,
  accountExists,
  remove0x,
  createLocalKeyStore,
  getWeb3,
  getEthContract,
  addSecretKey,
  fromWei,
  toWei,
  ethCallContract
} = require('./utils')
const { maybeCreateAccount, verifyAccount } = require('./helpers')
const { borshifyInitialValidators, borshify } = require('./borsh')

function getScript(name) {
  return path.resolve(path.join(__dirname, `scripts/${name}.sh`));
}

module.exports = {
  borshifyInitialValidators,
  borshify,
  getScript,
  nearAPI,
  Web3,
  sleep,
  RobustWeb3,
  setupEthNear,
  normalizeEthKey,
  accountExists,
  remove0x,
  createLocalKeyStore,
  getWeb3,
  getEthContract,
  addSecretKey,
  fromWei,
  toWei,
  txnStatus,
  ethCallContract,
  BorshContract,
  hexToBuffer,
  readerToHex,
  maybeCreateAccount,
  verifyAccount,
  RainbowConfig
}

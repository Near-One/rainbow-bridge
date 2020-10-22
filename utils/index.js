const { nearAPI, Web3, RobustWeb3 } = require('./robust')
const { RainbowConfig } = require('./config')
const {
  txnStatus,
  BorshContract,
  hexToBuffer,
  readerToHex,
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
  ethCallContract,
} = require('./utils')

module.exports = {
  nearAPI,
  Web3,
  RobustWeb3,
  setupEthNear,
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
  RainbowConfig,
}

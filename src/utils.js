const homedir = require('os').homedir()
const path = require('path')
const fs = require('fs')
const assert = require('bsert')

const nearAPI = require('near-api-js')
const Web3 = require('web3')

const CREDENTIALS_DIR = '.near-credentials'
const PROJECT_KEY_DIR = './neardev'

const DEFAULT_GAS = 1000000

async function setupNear(config) {
  const deps = await createLocalKeyStore(config.networkId, config.keyPath)
  if (config.keyPath) {
    delete config.keyPath
  }
  return nearAPI.connect({
    networkId: config.networkId,
    nodeUrl: config.nearNodeUrl,
    deps,
  })
}

async function setupEth(config) {
  const web3 = await getWeb3(config)
  web3.eth.defaultAccount = addSecretKey(web3, config.ethFromSecretKey)
  config.ethFrom = web3.eth.defaultAccount
  return web3
}

/**
 * Setup connection to NEAR and Ethereum from given configuration.
 * @param {Object} config Config object which defines nearNodeUrl/ethNodeUrl, networkId and more.
 */
async function setupEthNear(config) {
  const near = await setupNear(config)
  const web3 = await setupEth(config)
  return { near, web3 }
}

/**
 * Remove 0x if prepended
 * @param {String} input data
 * @return {String} string without 0x
 */
function remove0x(value) {
  assert(typeof value === 'string', 'remove0x: must pass in string')

  if (value.slice(0, 2) === '0x') {
    return value.slice(2)
  } else {
    return value
  }
}

function normalizeHex(value) {
  value = value.toLowerCase()
  if (!value.startsWith('0x')) {
    return `0x${value}`
  }
  return value
}

async function accountExists(connection, accountId) {
  try {
    const account = new nearAPI.Account(connection, accountId)
    await account.state()
    return true
  } catch (error) {
    if (!error.message.includes('does not exist while viewing')) {
      throw error
    }
    return false
  }
}

async function createLocalKeyStore(networkId, keyPath) {
  // TODO: this should live in near-api-js
  const credentialsPath = path.join(homedir, CREDENTIALS_DIR)
  const keyStores = [
    new nearAPI.keyStores.UnencryptedFileSystemKeyStore(credentialsPath),
    new nearAPI.keyStores.UnencryptedFileSystemKeyStore(PROJECT_KEY_DIR),
  ]
  if (keyPath) {
    const account = JSON.parse(fs.readFileSync(keyPath).toString())
    const keyPair = nearAPI.utils.KeyPair.fromString(account.secret_key)
    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    keyStore.setKey(networkId, account.account_id, keyPair).then(() => {})
    keyStores.push(keyStore)
  }
  return { keyStore: new nearAPI.keyStores.MergeKeyStore(keyStores) }
}

function getWeb3(config) {
  // TODO: add RobustWeb3 usage here.
  return new Web3(config.ethNodeUrl)
}

function getEthContract(web3, path, address) {
  const bin = fs.readFileSync(`${path}.full.bin`)
  const abi = fs.readFileSync(`${path}.full.abi`)
  const contract = new web3.eth.Contract(JSON.parse(abi), address, {
    from: web3.eth.defaultAccount,
  })
  contract.bin = bin
  return contract
}

function addSecretKey(web3, secretKey) {
  let account = web3.eth.accounts.privateKeyToAccount(normalizeHex(secretKey))
  web3.eth.accounts.wallet.add(account)
  return account.address
}

/**
 * Wrap pure calls to Web3 contract to handle errors/reverts/gas usage.
 * TODO: should use RobustWeb3 code.
 */
async function ethCallContract(contract, methodName, args) {
  let dryRun
  try {
    dryRun = await contract.methods[methodName](...args).call()
    return contract.methods[methodName](...args).send({
      gas: DEFAULT_GAS,
    })
  } catch (error) {
    if (error.message.includes('reverted by the EVM')) {
      console.warn(dryRun)
    }
    throw error
  }
}

module.exports = {
  setupEthNear,
  accountExists,
  remove0x,
  createLocalKeyStore,
  getWeb3,
  getEthContract,
  addSecretKey,
  fromWei: Web3.utils.fromWei,
  toWei: Web3.utils.toWei,
  ethCallContract,
}

const fs = require('fs')
const {
  RainbowConfig,
  RobustWeb3,
  remove0x,
  normalizeEthKey
} = require('rainbow-bridge-utils')

// TODO use config instead
const BRIDGE_SRC_DIR = __dirname
const path = require('path')
const { exit } = require('process')
const LIBS_TC_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  '../../node_modules/rainbow-token-connector'
)
RainbowConfig.declareOption(
  'eth-erc20-abi-path',
  'Path to the .abi file defining Ethereum ERC20 contract.',
  path.join(LIBS_TC_SRC_DIR, 'res/TToken.full.abi')
)

function getBalance (ethSecretKey, tokenAddressArg = null) {
  const tokenAddress = tokenAddressArg
    ? remove0x(tokenAddressArg)
    : RainbowConfig.getParam('eth-erc20-address')
  const robustWeb3 = new RobustWeb3(RainbowConfig.getParam('eth-node-url'))
  const web3 = robustWeb3.web3

  const ethContract = new web3.eth.Contract(
    JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-erc20-abi-path'))),
    tokenAddress
  )
  const ethAccount = web3.eth.accounts.privateKeyToAccount(
    normalizeEthKey(ethSecretKey)
  )
  web3.eth.accounts.wallet.add(ethAccount)
  web3.eth.defaultAccount = ethAccount.address
  const ethAddress = ethAccount.address
  return ethContract.methods.balanceOf(remove0x(ethAddress)).call().then(result => {
    console.log(result)
    exit(0)
  })
}

exports.getBalance = getBalance

require('make-runnable/custom')({
  printOutputFrame: false
})

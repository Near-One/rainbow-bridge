const fs = require('fs')
const {
  RobustWeb3,
  remove0x,
  normalizeEthKey
} = require('rainbow-bridge-utils')

// TODO use config instead
const { exit } = require('process')

function getEthErc20Balance ({ ethSecretKey, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3

  const ethContract = new web3.eth.Contract(
    JSON.parse(fs.readFileSync(ethErc20AbiPath)),
    ethErc20Address
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

exports.getEthErc20Balance = getEthErc20Balance

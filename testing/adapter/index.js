const fs = require('fs')
const {
  RobustWeb3,
  remove0x,
  normalizeEthKey
} = require('rainbow-bridge-utils')

const { exit } = require('process')

function getEthAddressBySecretKey ({ ethSecretKey, ethNodeUrl }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  const ethAccount = web3.eth.accounts.privateKeyToAccount(
    normalizeEthKey(ethSecretKey)
  )
  console.log(ethAccount.address)
  exit(0)
}

function getEthErc20Balance ({ ethAccountAddress, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  const ethContract = new web3.eth.Contract(
    JSON.parse(fs.readFileSync(ethErc20AbiPath)),
    remove0x(ethErc20Address)
  )
  return ethContract.methods.balanceOf(remove0x(ethAccountAddress)).call().then(result => {
    console.log(result)
    exit(0)
  })
}

exports.getEthErc20Balance = getEthErc20Balance
exports.getEthAddressBySecretKey = getEthAddressBySecretKey

const fs = require('fs')
const {
  RobustWeb3,
  remove0x,
  normalizeEthKey,
  JSONreplacer
} = require('rainbow-bridge-utils')

async function mintErc20 ({ ethAccountAddress, amount, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      remove0x(ethErc20Address)
    )
    await ethContract.methods.mint(ethAccountAddress, Number(amount)).send({ from: ethAccountAddress, gas: 5000000 })
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

function getEthAddressBySecretKey ({ ethSecretKey, ethNodeUrl }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethAccount = web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethSecretKey)
    )
    console.log(ethAccount.address)
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

async function getEthErc20Balance ({ ethAccountAddress, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      remove0x(ethErc20Address)
    )
    const result = await ethContract.methods.balanceOf(remove0x(ethAccountAddress)).call()
    console.log(result)
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

async function ethToNearApprove ({ ethAccountAddress, amount, ethNodeUrl, ethErc20Address, ethErc20AbiPath, ethLockerAddress }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath)),
      remove0x(ethErc20Address)
    )
    await robustWeb3.callContract(
      ethContract,
      'approve',
      [ethLockerAddress, Number(amount)],
      {
        from: ethAccountAddress,
        gas: 5000000
      }
    )
    console.log('OK')
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

async function ethToNearLock ({ ethAccountAddress, amount, nearAccountName, ethNodeUrl, ethErc20Address, ethLockerAbiPath, ethLockerAddress }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethTokenLockerContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethLockerAbiPath)),
      ethLockerAddress
    )
    const transaction = await robustWeb3.callContract(
      ethTokenLockerContract,
      'lockToken',
      [ethErc20Address, Number(amount), nearAccountName],
      {
        from: ethAccountAddress,
        gas: 5000000
      }
    )
    const lockedEvent = transaction.events.Locked
    console.log('OK')
    console.log(JSON.stringify(lockedEvent, JSONreplacer))
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

exports.ethToNearApprove = ethToNearApprove
exports.ethToNearLock = ethToNearLock
exports.mintErc20 = mintErc20
exports.getEthErc20Balance = getEthErc20Balance
exports.getEthAddressBySecretKey = getEthAddressBySecretKey

const fs = require('fs')
const bs58 = require('bs58')
const { toBuffer } = require('eth-util-lite')
const {
  RobustWeb3,
  remove0x,
  normalizeEthKey,
  JSONreplacer,
  borshifyOutcomeProof
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

function getAddressBySecretKey ({ ethSecretKey, ethNodeUrl }) {
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

async function getErc20Balance ({ ethAccountAddress, ethNodeUrl, ethErc20Address, ethErc20AbiPath }) {
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

async function getClientBlockHeightHash ({
  ethNodeUrl,
  ethClientAddress,
  ethClientAbiPath,
  ethMasterAccount
}) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const clientContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethClientAbiPath)),
      ethClientAddress,
      {
        from: ethMasterAccount,
        handleRevert: true
      }
    )

    const clientState = await clientContract.methods.bridgeState().call()
    const clientBlockHash = bs58.encode(
      toBuffer(
        await clientContract.methods.blockHashes(clientState.currentHeight).call()
      )
    )
    console.log(clientState.currentHeight, clientBlockHash)
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

async function nearToEthUnlock ({
  blockHeight,
  proof,
  ethNodeUrl,
  ethMasterSk,
  ethProverAddress,
  ethProverAbiPath,
  ethLockerAddress,
  ethLockerAbiPath,
  ethGasMultiplier
}) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const ethMasterAccount = web3.eth.accounts.privateKeyToAccount(normalizeEthKey(ethMasterSk)).address
    const proverContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethProverAbiPath)),
      ethProverAddress,
      {
        from: ethMasterAccount,
        handleRevert: true
      }
    )
    const ethTokenLockerContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethLockerAbiPath)),
      ethLockerAddress
    )
    const borshProof = borshifyOutcomeProof(JSON.parse(proof))
    blockHeight = Number(blockHeight)
    await proverContract.methods
      .proveOutcome(borshProof, blockHeight)
      .call()
    await robustWeb3.callContract(
      ethTokenLockerContract,
      'unlockToken',
      [borshProof, blockHeight],
      {
        from: ethMasterAccount,
        gas: 5000000,
        handleRevert: true,
        gasPrice: Number(await robustWeb3.web3.eth.getGasPrice() * ethGasMultiplier)
      }
    )
    console.log('OK')
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

exports.ethToNearApprove = ethToNearApprove
exports.ethToNearLock = ethToNearLock
exports.nearToEthUnlock = nearToEthUnlock
exports.mintErc20 = mintErc20
exports.getErc20Balance = getErc20Balance
exports.getAddressBySecretKey = getAddressBySecretKey
exports.getClientBlockHeightHash = getClientBlockHeightHash

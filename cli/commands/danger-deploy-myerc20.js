const { Web3, normalizeEthKey } = require('rainbow-bridge-utils')
const { BN } = require('ethereumjs-util')
const fs = require('fs')

class DangerDeployMyERC20 {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    ethErc20AbiPath,
    ethGasMultiplier
  }) {
    const web3 = new Web3(ethNodeUrl)
    let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethMasterSk)
    )
    web3.eth.accounts.wallet.add(ethMasterAccount)
    web3.eth.defaultAccount = ethMasterAccount.address
    ethMasterAccount = ethMasterAccount.address

    // use default ERC20 ABI
    const binPath = '../testing/ci/MyERC20.full.bin'

    const tokenContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethErc20AbiPath))
    )
    const txContract = await tokenContract
      .deploy({
        data: '0x' + fs.readFileSync(binPath),
        arguments: []
      })
      .send({
        from: ethMasterAccount,
        gas: 3000000,
        gasPrice: new BN(await web3.eth.getGasPrice()).mul(
          new BN(ethGasMultiplier)
        )
      })

    const tokenAddress = normalizeEthKey(txContract.options.address)
    console.log(tokenAddress)
    web3.currentProvider.disconnect()
  }
}

exports.DangerDeployMyERC20 = DangerDeployMyERC20

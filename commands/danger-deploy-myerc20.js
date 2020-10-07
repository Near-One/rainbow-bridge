const Web3 = require('web3')
const { RainbowConfig } = require('rainbow-bridge-lib/config')
const { BN } = require('ethereumjs-util')
const fs = require('fs')
const { normalizeEthKey } = require('rainbow-bridge-lib/rainbow/robust')

class DangerDeployMyERC20 {
  static async execute() {
    const web3 = new Web3(RainbowConfig.getParam('eth-node-url'))
    let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(RainbowConfig.getParam('eth-master-sk'))
    )
    web3.eth.accounts.wallet.add(ethMasterAccount)
    web3.eth.defaultAccount = ethMasterAccount.address
    ethMasterAccount = ethMasterAccount.address

    // use default ERC20 ABI
    const abiPath = RainbowConfig.getParam('eth-erc20-abi-path')
    const binPath = './test/MyERC20.full.bin'

    const tokenContract = new web3.eth.Contract(
      JSON.parse(fs.readFileSync(abiPath))
    )
    const txContract = await tokenContract
      .deploy({
        data: '0x' + fs.readFileSync(binPath),
        arguments: [],
      })
      .send({
        from: ethMasterAccount,
        gas: 3000000,
        gasPrice: new BN(await web3.eth.getGasPrice()).mul(
          new BN(RainbowConfig.getParam('eth-gas-multiplier'))
        ),
      })

    const tokenAddress = normalizeEthKey(txContract.options.address)
    console.log(tokenAddress)
    web3.currentProvider.connection.close()
  }
}

exports.DangerDeployMyERC20 = DangerDeployMyERC20

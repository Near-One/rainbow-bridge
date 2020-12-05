const {
  TransferETHERC20ToNear,
  TransferEthERC20FromNear,
  DeployToken
} = require('./transfer-eth-erc20')
const {
  mintErc20,
  getEthErc20Balance,
  getEthAddressBySecretKey
} = require('./adapter')

exports.TransferETHERC20ToNear = TransferETHERC20ToNear
exports.TransferEthERC20FromNear = TransferEthERC20FromNear
exports.DeployToken = DeployToken
exports.mintErc20 = mintErc20
exports.getEthErc20Balance = getEthErc20Balance
exports.getEthAddressBySecretKey = getEthAddressBySecretKey

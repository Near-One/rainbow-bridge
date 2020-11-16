const {
  TransferETHERC20ToNear,
  TransferEthERC20FromNear,
  DeployToken
} = require('./transfer-eth-erc20')
const {
  getEthErc20Balance
} = require('./adapter')

exports.TransferETHERC20ToNear = TransferETHERC20ToNear
exports.TransferEthERC20FromNear = TransferEthERC20FromNear
exports.DeployToken = DeployToken
exports.getEthErc20Balance = getEthErc20Balance

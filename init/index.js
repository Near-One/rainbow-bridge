const {
  InitEthClient,
  InitEthEd25519,
  InitEthErc20,
  InitEthLocker,
  InitEthProver,
} = require('./eth-contracts')
const { InitNearContracts } = require('./near-contracts')
const { InitNearFunToken } = require('./near-fun-token')

exports.InitEthEd25519 = InitEthEd25519
exports.InitEthErc20 = InitEthErc20
exports.InitEthLocker = InitEthLocker
exports.InitEthClient = InitEthClient
exports.InitEthProver = InitEthProver
exports.InitNearContracts = InitNearContracts
exports.InitNearFunToken = InitNearFunToken

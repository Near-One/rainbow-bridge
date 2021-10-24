const {
  InitEthClient,
  InitEthEd25519,
  InitEthErc20,
  InitEthLocker,
  InitEthNftLocker,
  InitEthProver
} = require('./eth-contracts')
const { InitNearContracts } = require('./near-contracts')
const { InitNearTokenFactory } = require('./near-token-factory')
const { InitNearNftTokenFactory } = require('./near-nft-token-factory')

exports.InitEthEd25519 = InitEthEd25519
exports.InitEthErc20 = InitEthErc20
exports.InitEthLocker = InitEthLocker
exports.InitEthNftLocker = InitEthNftLocker
exports.InitEthClient = InitEthClient
exports.InitEthProver = InitEthProver
exports.InitNearContracts = InitNearContracts
exports.InitNearTokenFactory = InitNearTokenFactory
exports.InitNearNftTokenFactory = InitNearNftTokenFactory

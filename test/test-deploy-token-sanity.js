const Web3 = require('web3')
const { nearlib } = require('rainbow-bridge-lib')
const { RainbowConfig } = require('rainbow-bridge-lib/config')
const { BN } = require('ethereumjs-util')
const fs = require('fs')
const path = require('path')
const { normalizeEthKey } = require('rainbow-bridge-lib/rainbow/robust')
const { DeployToken } = require('rainbow-bridge-lib/transfer-eth-erc20')

const TEST_DIR = __dirname
const BRIDGE_SRC_DIR = path.join(TEST_DIR, '..')
const LIBS_SOL_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  'node_modules/rainbow-bridge-sol'
)
const LIBS_TC_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  'node_modules/rainbow-token-connector'
)

async function init() {
  RainbowConfig.declareOption(
    'near-token-factory-contract-path',
    'The path to the Wasm file containing the fungible contract. Note, this version of fungible contract should support minting.',
    path.join(LIBS_TC_SRC_DIR, 'res/bridge_token_factory.wasm')
  )
  RainbowConfig.declareOption(
    'near-token-factory-init-balance',
    'The initial balance of fungible token contract in femtoNEAR.',
    '1000000000000000000000000000'
  )
  RainbowConfig.declareOption('eth-gas-multiplier', '', '1')
  RainbowConfig.declareOption(
    'eth-erc20-abi-path',
    'Path to the .abi file defining Ethereum ERC20 contract.',
    path.join(LIBS_TC_SRC_DIR, 'res/TToken.full.abi')
  )
  RainbowConfig.declareOption(
    'eth-erc20-bin-path',
    'Path to the .bin file defining Ethereum ERC20 contract.',
    path.join(LIBS_TC_SRC_DIR, 'res/TToken.full.bin')
  )
  RainbowConfig.declareOption(
    'eth-ed25519-abi-path',
    '',
    path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/Ed25519.full.abi')
  )
  RainbowConfig.declareOption(
    'eth-ed25519-bin-path',
    '',
    path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/Ed25519.full.bin')
  )
  RainbowConfig.declareOption('eth-client-lock-eth-amount', '', '1e18')
  RainbowConfig.declareOption('eth-client-lock-duration', '', '30')
  RainbowConfig.declareOption(
    'eth-client-abi-path',
    'Path to the .abi file defining Ethereum Client contract.',
    path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/NearBridge.full.abi')
  )
  RainbowConfig.declareOption(
    'eth-client-bin-path',
    'Path to the .bin file defining Ethereum Client contract.',
    path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/NearBridge.full.bin')
  )
  RainbowConfig.declareOption(
    'eth-prover-abi-path',
    'Path to the .abi file defining Ethereum Prover contract.',
    path.join(LIBS_SOL_SRC_DIR, 'nearprover/dist/NearProver.full.abi')
  )
  RainbowConfig.declareOption(
    'eth-prover-bin-path',
    'Path to the .bin file defining Ethereum Prover contract.',
    path.join(LIBS_SOL_SRC_DIR, 'nearprover/dist/NearProver.full.bin')
  )
  RainbowConfig.declareOption(
    'near-token-factory-account',
    '',
    'neartokenfactory'
  )
  RainbowConfig.declareOption(
    'eth-locker-abi-path',
    'Path to the .abi file defining Ethereum locker contract. This contract works in pair with mintable fungible token on NEAR blockchain.',
    path.join(LIBS_TC_SRC_DIR, 'res/BridgeTokenFactory.full.abi')
  )
  RainbowConfig.declareOption(
    'eth-locker-bin-path',
    'Path to the .bin file defining Ethereum locker contract. This contract works in pair with mintable fungible token on NEAR blockchain.',
    path.join(LIBS_TC_SRC_DIR, 'res/BridgeTokenFactory.full.bin')
  )
  RainbowConfig.declareOption(
    'near-prover-account',
    'The account of the Near Prover contract that can be used to accept ETH headers.',
    'rainbow_bridge_eth_on_near_prover'
  )
  RainbowConfig.saveConfig()
}

async function testDeployToken() {
  await init()

  const web3 = new Web3(RainbowConfig.getParam('eth-node-url'))
  let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(
    normalizeEthKey(RainbowConfig.getParam('eth-master-sk'))
  )
  web3.eth.accounts.wallet.add(ethMasterAccount)
  web3.eth.defaultAccount = ethMasterAccount.address
  ethMasterAccount = ethMasterAccount.address

  // use default ERC20 ABI
  const abiPath = RainbowConfig.getParam('eth-erc20-abi-path')
  const binPath = './MyERC20.full.bin'

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

  tokenAddress = normalizeEthKey(txContract.options.address)
  console.log('token address is', tokenAddress)
  web3.currentProvider.connection.close()

  await DeployToken.execute('myerc20', tokenAddress)

  console.log(
    'near-myerc20-account set to ' +
      RainbowConfig.getParam('near-myerc20-account')
  )
  console.log(
    'eth-myerc20-address set to ' +
      RainbowConfig.getParam('eth-myerc20-address')
  )
}

testDeployToken()

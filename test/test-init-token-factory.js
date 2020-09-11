const nearlib = require('near-api-js')
const {
  InitNearContracts,
  InitNearFunToken,
  InitEthEd25519,
  InitEthErc20,
  InitEthLocker,
  InitEthClient,
  InitEthProver,
} = require('rainbow-bridge-lib/init')
const { RainbowConfig } = require('rainbow-bridge-lib/config')
const {
  maybeCreateAccount,
  verifyAccount,
} = require('rainbow-bridge-lib/rainbow/helpers')
const { BN } = require('ethereumjs-util')
const path = require('path')

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
    'near-fun-token-contract-path',
    'The path to the Wasm file containing the fungible contract. Note, this version of fungible contract should support minting.',
    path.join(LIBS_TC_SRC_DIR, 'res/bridge_token_factory.wasm')
  )
  RainbowConfig.declareOption(
    'near-fun-token-init-balance',
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
  await InitEthErc20.execute()
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
  await InitEthEd25519.execute()
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
  await InitEthClient.execute()
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
  await InitEthProver.execute()
  RainbowConfig.declareOption('near-fun-token-account', '', 'nearfuntoken')
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
  await InitEthLocker.execute()
  RainbowConfig.declareOption(
    'near-prover-account',
    'The account of the Near Prover contract that can be used to accept ETH headers.',
    'rainbow_bridge_eth_on_near_prover'
  )
}

async function testInitTokenFactory() {
  await init()
  const masterAccount = RainbowConfig.getParam('near-master-account')
  const masterSk = RainbowConfig.getParam('near-master-sk')
  const tokenAccount = RainbowConfig.getParam('near-fun-token-account')
  let tokenSk = RainbowConfig.maybeGetParam('near-fun-token-sk')
  if (!tokenSk) {
    console.log(
      'Secret key for fungible token is not specified. Reusing master secret key.'
    )
    tokenSk = masterSk
    RainbowConfig.setParam('near-fun-token-sk', tokenSk)
  }
  const tokenContractPath = RainbowConfig.getParam(
    'near-fun-token-contract-path'
  )
  const tokenInitBalance = RainbowConfig.getParam('near-fun-token-init-balance')
  const proverAccount = RainbowConfig.getParam('near-prover-account')

  const nearNodeUrl = RainbowConfig.getParam('near-node-url')
  const nearNetworkId = RainbowConfig.getParam('near-network-id')

  const tokenPk = nearlib.KeyPair.fromString(tokenSk).getPublicKey()

  const keyStore = new nearlib.keyStores.InMemoryKeyStore()
  await keyStore.setKey(
    nearNetworkId,
    masterAccount,
    nearlib.KeyPair.fromString(masterSk)
  )
  await keyStore.setKey(
    nearNetworkId,
    tokenAccount,
    nearlib.KeyPair.fromString(tokenSk)
  )
  const near = await nearlib.connect({
    nodeUrl: nearNodeUrl,
    networkId: nearNetworkId,
    masterAccount: masterAccount,
    deps: { keyStore: keyStore },
  })

  await verifyAccount(near, masterAccount)
  console.log('Deploying token contract.')
  await maybeCreateAccount(
    near,
    masterAccount,
    tokenAccount,
    tokenPk,
    tokenInitBalance,
    tokenContractPath
  )
  const tokenFactoryContract = new nearlib.Contract(
    new nearlib.Account(near.connection, tokenAccount),
    tokenAccount,
    {
      changeMethods: ['new', 'deploy_bridge_token'],
      viewMethods: ['get_bridge_token_account_id'],
    }
  )
  const lockerAddress = RainbowConfig.getParam('eth-locker-address')
  try {
    // Try initializing the factory.
    await tokenFactoryContract.new(
      {
        prover_account: proverAccount,
        locker_address: lockerAddress.startsWith('0x')
          ? lockerAddress.substr(2)
          : lockerAddress,
      },
      new BN('70000000000000000000000000')
    )
  } catch (err) {
    console.log(`Failed to initialize the token factory ${err}`)
    process.exit(1)
  }
  const erc20Address = RainbowConfig.getParam('eth-erc20-address')
  try {
    // Try initializing the contract.
    await tokenFactoryContract.deploy_bridge_token(
      {
        address: erc20Address.startsWith('0x')
          ? erc20Address.substr(2)
          : erc20Address,
      },
      new BN('150000000000000000000000000')
    )
  } catch (err) {
    console.log(`Failed to initialize the token contract ${err}`)
    process.exit(1)
  }
  console.log('Fungible token deployed')
}

testInitTokenFactory()

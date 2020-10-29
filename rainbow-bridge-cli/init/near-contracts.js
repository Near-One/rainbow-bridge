const {
  RobustWeb3,
  nearAPI,
  RainbowConfig,
  maybeCreateAccount,
  verifyAccount
} = require('rainbow-bridge-utils')
const {
  EthOnNearClientContract,
  EthOnNearProverContract
} = require('rainbow-bridge-eth2near-block-relay')

class InitNearContracts {
  static async execute () {
    const masterAccount = RainbowConfig.getParam('near-master-account')
    const masterSk = RainbowConfig.getParam('near-master-sk')
    const clientAccount = RainbowConfig.getParam('near-client-account')
    let clientSk = RainbowConfig.maybeGetParam('near-client-sk')
    if (!clientSk) {
      console.log(
        'Key to call Near Client contract is not specified. Reusing master key.'
      )
      clientSk = masterSk
      RainbowConfig.setParam('near-client-sk', masterSk)
    }
    const clientContractPath = RainbowConfig.getParam(
      'near-client-contract-path'
    )
    const clientInitBalance = RainbowConfig.getParam('near-client-init-balance')

    const proverAccount = RainbowConfig.getParam('near-prover-account')
    let proverSk = RainbowConfig.maybeGetParam('near-prover-sk')
    if (!proverSk) {
      console.log(
        'Key to call Near Prover contract is not specified. Reusing master key.'
      )
      proverSk = masterSk
      RainbowConfig.setParam('near-prover-sk', masterSk)
    }
    const proverContractPath = RainbowConfig.getParam(
      'near-prover-contract-path'
    )
    const proverInitBalance = RainbowConfig.getParam('near-prover-init-balance')

    const nearNodeUrl = RainbowConfig.getParam('near-node-url')
    const nearNetworkId = RainbowConfig.getParam('near-network-id')
    const validateEthash = RainbowConfig.getParam('near-client-validate-ethash')
    const trustedSigner = RainbowConfig.getParam('near-client-trusted-signer')

    const clientPk = nearAPI.KeyPair.fromString(clientSk).getPublicKey()
    const proverPk = nearAPI.KeyPair.fromString(proverSk).getPublicKey()

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      nearNetworkId,
      masterAccount,
      nearAPI.KeyPair.fromString(masterSk)
    )
    await keyStore.setKey(
      nearNetworkId,
      clientAccount,
      nearAPI.KeyPair.fromString(clientSk)
    )
    await keyStore.setKey(
      nearNetworkId,
      proverAccount,
      nearAPI.KeyPair.fromString(proverSk)
    )
    const near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      masterAccount: masterAccount,
      deps: {
        keyStore: keyStore
      }
    })

    console.log('Creating accounts and deploying the contracts.')
    await verifyAccount(near, masterAccount)
    await maybeCreateAccount(
      near,
      masterAccount,
      clientAccount,
      clientPk,
      clientInitBalance,
      clientContractPath
    )
    await verifyAccount(near, clientAccount)
    await maybeCreateAccount(
      near,
      masterAccount,
      proverAccount,
      proverPk,
      proverInitBalance,
      proverContractPath
    )
    await verifyAccount(near, proverAccount)

    console.log('Initializing client and prover contracts.')
    const clientContract = new EthOnNearClientContract(
      new nearAPI.Account(near.connection, clientAccount),
      clientAccount
    )
    const robustWeb3 = new RobustWeb3(RainbowConfig.getParam('eth-node-url'))
    await clientContract.maybeInitialize(
      validateEthash === 'true',
      trustedSigner || null,
      robustWeb3
    )
    const proverContract = new EthOnNearProverContract(
      new nearAPI.Account(near.connection, proverAccount),
      proverAccount
    )
    await proverContract.maybeInitialize(clientAccount)
    RainbowConfig.saveConfig()
    process.exit(0)
  }
}

exports.InitNearContracts = InitNearContracts

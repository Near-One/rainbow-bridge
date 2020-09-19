const nearlib = require('near-api-js')
const { maybeCreateAccount, verifyAccount } = require('../rainbow/helpers')
const { RainbowConfig } = require('../config')
const { BN } = require('ethereumjs-util')
const { DeployToken } = require('../transfer-eth-erc20/deploy-token')

class InitNearTokenFactory {
  static async execute() {
    const masterAccount = RainbowConfig.getParam('near-master-account')
    const masterSk = RainbowConfig.getParam('near-master-sk')
    const tokenFactoryAccount = RainbowConfig.getParam(
      'near-token-factory-account'
    )
    let tokenSk = RainbowConfig.maybeGetParam('near-token-factory-sk')
    if (!tokenSk) {
      console.log(
        'Secret key for fungible token is not specified. Reusing master secret key.'
      )
      tokenSk = masterSk
      RainbowConfig.setParam('near-token-factory-sk', tokenSk)
    }
    const tokenContractPath = RainbowConfig.getParam(
      'near-token-factory-contract-path'
    )
    const tokenInitBalance = RainbowConfig.getParam(
      'near-token-factory-init-balance'
    )
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
      tokenFactoryAccount,
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
      tokenFactoryAccount,
      tokenPk,
      tokenInitBalance,
      tokenContractPath
    )
    const tokenFactoryContract = new nearlib.Contract(
      new nearlib.Account(near.connection, tokenFactoryAccount),
      tokenFactoryAccount,
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
        new BN('300000000000000')
      )
    } catch (err) {
      console.log(`Failed to initialize the token factory ${err}`)
      process.exit(1)
    }

    DeployToken.execute('erc20', RainbowConfig.getParam('eth-erc20-address'))
  }
}

exports.InitNearTokenFactory = InitNearTokenFactory

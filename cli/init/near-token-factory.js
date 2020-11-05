const {
  nearAPI,
  maybeCreateAccount,
  verifyAccount
} = require('rainbow-bridge-utils')
const { BN } = require('ethereumjs-util')
const { DeployToken } = require('rainbow-bridge-testing')

class InitNearTokenFactory {
  static async execute ({
    nearMasterAccount,
    nearMasterSk,
    nearTokenFactoryAccount,
    nearTokenFactorySk,
    nearTokenFactoryContractPath,
    nearTokenFactoryInitBalance,
    nearProverAccount,
    nearNodeUrl,
    nearNetworkId,
    ethLockerAddress,
    ethErc20Address
  }) {
    if (!nearTokenFactorySk) {
      console.log(
        'Secret key for fungible token is not specified. Reusing master secret key.'
      )
      nearTokenFactorySk = nearMasterSk
      RainbowConfig.setParam('near-token-factory-sk', nearTokenFactorySk)
    }
    const nearTokenFactoryPk = nearAPI.KeyPair.fromString(nearTokenFactorySk).getPublicKey()

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      nearNetworkId,
      nearMasterAccount,
      nearAPI.KeyPair.fromString(nearMasterSk)
    )
    await keyStore.setKey(
      nearNetworkId,
      nearTokenFactoryAccount,
      nearAPI.KeyPair.fromString(nearTokenFactorySk)
    )
    const near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      masterAccount: nearMasterAccount,
      deps: { keyStore: keyStore }
    })

    await verifyAccount(near, nearMasterAccount)
    console.log('Deploying token contract.')
    await maybeCreateAccount(
      near,
      nearMasterAccount,
      nearTokenFactoryAccount,
      nearTokenFactoryPk,
      nearTokenFactoryInitBalance,
      nearTokenFactoryContractPath
    )
    const tokenFactoryContract = new nearAPI.Contract(
      new nearAPI.Account(near.connection, nearTokenFactoryAccount),
      nearTokenFactoryAccount,
      {
        changeMethods: ['new', 'deploy_bridge_token'],
        viewMethods: ['get_bridge_token_account_id']
      }
    )
    try {
      // Try initializing the factory.
      await tokenFactoryContract.new(
        {
          prover_account: nearProverAccount,
          locker_address: ethLockerAddress.startsWith('0x')
            ? ethLockerAddress.substr(2)
            : ethLockerAddress
        },
        new BN('300000000000000')
      )
    } catch (err) {
      console.log(`Failed to initialize the token factory ${err}`)
      process.exit(1)
    }

    DeployToken.execute('erc20', ethErc20Address)
  }
}

exports.InitNearTokenFactory = InitNearTokenFactory

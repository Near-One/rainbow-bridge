const {
  nearAPI,
  maybeCreateAccount,
  verifyAccount
} = require('rainbow-bridge-utils')
const { BN } = require('ethereumjs-util')
const { DeployToken } = require('rainbow-bridge-testing')

class InitNearNftTokenFactory {
  static async execute ({
    nearMasterAccount,
    nearMasterSk,
    nearNftTokenFactoryAccount,
    nearNftTokenFactorySk,
    nearNftTokenFactoryContractPath,
    nearTokenFactoryInitBalance,
    nearProverAccount,
    nearNodeUrl,
    nearNetworkId,
    ethNftLockerAddress,
    ethErc721Address
  }) {
    if (!nearNftTokenFactorySk) {
      console.log(
        'Secret key for fungible token is not specified. Reusing master secret key.'
      )
      nearNftTokenFactorySk = nearMasterSk
    }
    const nearTokenFactoryPk = nearAPI.KeyPair.fromString(nearNftTokenFactorySk).getPublicKey()

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      nearNetworkId,
      nearMasterAccount,
      nearAPI.KeyPair.fromString(nearMasterSk)
    )
    await keyStore.setKey(
      nearNetworkId,
      nearNftTokenFactoryAccount,
      nearAPI.KeyPair.fromString(nearNftTokenFactorySk)
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
      nearNftTokenFactoryAccount,
      nearTokenFactoryPk,
      nearTokenFactoryInitBalance,
      nearNftTokenFactoryContractPath
    )
    const tokenFactoryContract = new nearAPI.Contract(
      new nearAPI.Account(near.connection, nearNftTokenFactoryAccount),
      nearNftTokenFactoryAccount,
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
          locker_address: ethNftLockerAddress.startsWith('0x')
            ? ethNftLockerAddress.substr(2)
            : ethNftLockerAddress
        },
        new BN('300000000000000')
      )
    } catch (err) {
      console.log(`Failed to initialize the token factory ${err}`)
      process.exit(1)
    }

    const deployedTokenInfo = await DeployToken.execute({
      tokenName: 'erc721',
      ethTokenAddress: ethErc721Address,
      nearNodeUrl,
      nearNetworkId,
      nearMasterAccount,
      nearMasterSk,
      nearNftTokenFactoryAccount,
      nearNftTokenFactorySk
    })
    if (!deployedTokenInfo) {
      return null
    }
    const {
      nearTokenAccount,
      ethTokenAddress,
      ...otherDeployedTokenInfo
    } = deployedTokenInfo
    return {
      nearErc721Account: nearTokenAccount,
      ethErc721Address: ethTokenAddress,
      ...otherDeployedTokenInfo
    }
  }
}

exports.InitNearNftTokenFactory = InitNearNftTokenFactory

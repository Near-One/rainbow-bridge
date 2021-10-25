const {
  nearAPI,
  remove0x,
  verifyAccount
} = require('rainbow-bridge-utils')
const { BN } = require('ethereumjs-util')

class DeployNftToken {
  static async execute ({
    tokenName,
    ethTokenAddress,
    nearNodeUrl,
    nearNetworkId,
    nearMasterAccount,
    nearMasterSk,
    nearNftTokenFactoryAccount,
    nearNftTokenFactorySk
  }) {
    // use init near instead
    if (!nearNftTokenFactorySk) {
      console.log(
        'Secret key for fungible token is not specified. Reusing master secret key.'
      )
      nearNftTokenFactorySk = nearMasterSk
    }

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
    await verifyAccount(near, nearNftTokenFactoryAccount)

    console.log('Adding token ' + tokenName + ' at ' + ethTokenAddress)

    const tokenFactoryContract = new nearAPI.Contract(
      new nearAPI.Account(near.connection, nearNftTokenFactoryAccount),
      nearNftTokenFactoryAccount,
      {
        changeMethods: ['deploy_bridge_token'],
        viewMethods: ['get_bridge_token_account_id']
      }
    )

    try {
      // Try initializing the contract.
      await tokenFactoryContract.deploy_bridge_token(
        {
          address: remove0x(ethTokenAddress)
        },
        new BN('300000000000000'),
        new BN('3500000000000000000000000')
      )
    } catch (err) {
      console.log(
        `Failed to initialize the token ${tokenName} contract: ${err}`
      )
      process.exit(1)
    }
    console.log(`${tokenName} deployed`)

    return {
      nearNftTokenFactorySk,
      nearTokenAccount: remove0x(ethTokenAddress) + '.' + nearNftTokenFactoryAccount,
      ethTokenAddress
    }
  }
}

module.exports = { DeployNftToken }

const nearlib = require('near-api-js')
const utils = require('../src/utils')
const { RainbowConfig } = require('../config')
const { verifyAccount } = require('../rainbow/helpers')
const { BN } = require('ethereumjs-util')

function tokenAccountParam(tokenName) {
  return 'near-' + tokenName + '-account'
}

function tokenAddressParam(tokenName) {
  return 'eth-' + tokenName + '-address'
}

class DeployToken {
  static async execute(tokenName, tokenAddress) {
    console.log('execute', tokenName, tokenAddress)
    if (RainbowConfig.maybeGetParam(tokenAccountParam(tokenName))) {
      console.log(
        `Token name ${tokenName} is occupied. Is it already deployed?`
      )
      process.exit(1)
    }

    // use init near instead
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
    const nearNodeUrl = RainbowConfig.getParam('near-node-url')
    const nearNetworkId = RainbowConfig.getParam('near-network-id')

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
    await verifyAccount(near, tokenFactoryAccount)

    console.log('Adding token ' + tokenName + ' at ' + tokenAddress)

    const tokenFactoryContract = new nearlib.Contract(
      new nearlib.Account(near.connection, tokenFactoryAccount),
      tokenFactoryAccount,
      {
        changeMethods: ['deploy_bridge_token'],
        viewMethods: ['get_bridge_token_account_id'],
      }
    )

    try {
      // Try initializing the contract.
      await tokenFactoryContract.deploy_bridge_token(
        {
          address: utils.remove0x(tokenAddress),
        },
        new BN('300000000000000'),
        new BN('150000000000000000000000000')
      )
    } catch (err) {
      console.log(
        `Failed to initialize the token ${tokenName} contract: ${err}`
      )
      process.exit(1)
    }
    console.log(`${tokenName} deployed`)
    RainbowConfig.setParam(
      tokenAccountParam(tokenName),
      utils.remove0x(tokenAddress) + '.' + tokenFactoryAccount
    )
    RainbowConfig.setParam(tokenAddressParam(tokenName), tokenAddress)

    RainbowConfig.saveConfig()
    console.log(
      `Token address of ${tokenName} set to ${RainbowConfig.getParam(
        tokenAccountParam(tokenName)
      )}, param ${tokenAccountParam(tokenName)}`
    )
  }
}

module.exports = { DeployToken, tokenAddressParam, tokenAccountParam }

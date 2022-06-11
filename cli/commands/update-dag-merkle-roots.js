const { nearAPI } = require('rainbow-bridge-utils')
const { EthOnNearClientContract } = require('rainbow-bridge-eth2near-block-relay')

class UpdateDagMerkleRoots {
  static async execute ({
    dagsStartEpoch,
    nearClientAccount,
    nearClientSk,
    nearNodeUrl,
    nearNetworkId
  }) {
    if (!nearClientSk) {
      console.log(
        'Key to call Near Client contract is not specified.'
      )

      process.exit(1)
    }

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    await keyStore.setKey(
      nearNetworkId,
      nearClientAccount,
      nearAPI.KeyPair.fromString(nearClientSk)
    )

    const near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      deps: {
        keyStore: keyStore
      }
    })

    console.log('Update dag merkle roots for client contract.')
    const clientContract = new EthOnNearClientContract(
      new nearAPI.Account(near.connection, nearClientAccount),
      nearClientAccount
    )
    await clientContract.updateDagMerkleRoots(dagsStartEpoch)
  }
}

exports.UpdateDagMerkleRoots = UpdateDagMerkleRoots

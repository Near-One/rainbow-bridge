const { nearAPI } = require('rainbow-bridge-utils')
const { EthOnNearClientContract, dagMerkleRoots } = require('rainbow-bridge-eth2near-block-relay')
const sha256 = require('js-sha256')
const readlineSync = require('readline-sync')
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
      keyStore
    })

    console.log(`DAG Merkle roots SHA256-checksum: ${sha256(JSON.stringify(dagMerkleRoots, null, 2))}`)
    console.log(`Start epoch: ${dagsStartEpoch}`)
    console.log(`Client account: ${nearClientAccount}`)

    const inputResult = readlineSync.question('Please verify the data above. Do you confirm that you want to update the DAG Merkle roots? Enter CONFIRM if yes: ')
    if (inputResult.toUpperCase() !== 'CONFIRM') {
      console.error('The task was aborted')
      return
    }

    console.log('Update DAG Merkle roots for the client contract.')
    const clientContract = new EthOnNearClientContract(
      new nearAPI.Account(near.connection, nearClientAccount),
      nearClientAccount
    )
    await clientContract.updateDagMerkleRoots(dagsStartEpoch)
  }
}

exports.UpdateDagMerkleRoots = UpdateDagMerkleRoots

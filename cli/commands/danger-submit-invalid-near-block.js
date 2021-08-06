const { Near2EthRelay } = require('rainbow-bridge-near2eth-block-relay')

class DangerSubmitInvalidNearBlock {
  static async execute ({
    nearNodeUrl,
    nearNetworkId,
    ethNodeUrl,
    ethMasterSk,
    ethClientArtifactPath,
    ethClientAddress,
    ethGasMultiplier,
    near2ethRelayMinDelay,
    near2ethRelayMaxDelay,
    near2ethRelayErrorDelay
  }) {
    const relay = new Near2EthRelay()
    await relay.initialize({
      nearNodeUrl,
      nearNetworkId,
      ethNodeUrl,
      ethMasterSk,
      ethClientArtifactPath,
      ethClientAddress,
      ethGasMultiplier
    })
    await relay.DANGERsubmitInvalidNearBlock({
      near2ethRelayMinDelay,
      near2ethRelayMaxDelay,
      near2ethRelayErrorDelay,
      ethGasMultiplier
    })
  }
}

exports.DangerSubmitInvalidNearBlock = DangerSubmitInvalidNearBlock

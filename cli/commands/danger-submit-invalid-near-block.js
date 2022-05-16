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
    ethUseEip1559,
    near2ethRelayMinDelay,
    near2ethRelayMaxDelay,
    near2ethRelayErrorDelay,
    near2ethRelayBlockSelectDuration,
    near2ethRelayNextBlockSelectDelayMs,
    near2ethRelayAfterSubmitDelayMs,
    logVerbose
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
      near2ethRelayBlockSelectDuration,
      near2ethRelayNextBlockSelectDelayMs,
      near2ethRelayAfterSubmitDelayMs,
      ethGasMultiplier,
      ethUseEip1559,
      logVerbose
    })
  }
}

exports.DangerSubmitInvalidNearBlock = DangerSubmitInvalidNearBlock

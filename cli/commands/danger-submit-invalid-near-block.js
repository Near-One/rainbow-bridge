const { Near2EthRelay } = require('rainbow-bridge-near2eth-block-relay')

class DangerSubmitInvalidNearBlock {
  static async execute () {
    const relay = new Near2EthRelay()
    await relay.initialize()
    await relay.DANGERsubmitInvalidNearBlock()
  }
}

exports.DangerSubmitInvalidNearBlock = DangerSubmitInvalidNearBlock

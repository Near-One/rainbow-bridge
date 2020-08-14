const { Near2EthRelay } = require('../lib/near2eth-relay')

class DangerSubmitInvalidNearBlock {
  static async execute() {
    const relay = new Near2EthRelay()
    await relay.initialize()
    await relay.DANGER_submitInvalidNearBlock()
  }
}

exports.DangerSubmitInvalidNearBlock = DangerSubmitInvalidNearBlock

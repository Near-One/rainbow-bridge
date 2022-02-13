/* eslint-env jest */
/* eslint-disable indent */
require('dotenv').config()
const { Eth2NearRelay } = require('./index')
const os = require('os')
const fs = require('fs')
const path = require('path')
jest.setTimeout(1800000)

test('Computing Ethasproof epochs in advance ', async () => {
    const config = {
      ethNodeUrl: process.env.WEB3_RPC_ENDPOINT,
      totalSubmitBlock: 0,
      gasPerTransaction: 1,
      nearNetworkId: 'testnet',
      metricsPort: 0
    }

    const eth2NearRelay = new Eth2NearRelay()
    eth2NearRelay.initialize(null, config)
    const proofDir = path.join(os.homedir(), '.ethashproof')
    try {
        fs.rmSync(proofDir, { recursive: true })
    } catch (_e) {
    }

    const epoch = 0
    const numBlocksPerEpoch = 30000
    // Trigger computing epochs 0 and 1
    const result1 = await eth2NearRelay.getParseBlock(epoch * numBlocksPerEpoch + 26000)
    expect(result1).toBeTruthy()
    // Check if a cache for epoch 0 is created
    expect(fs.existsSync(path.join(proofDir, '0.json'))).toBeTruthy()

    const result2 = await eth2NearRelay.ethashproof.nextEpochPromise
    expect(result2).toBeTruthy()
    // Check if a cache for epoch 1 is created
    expect(fs.existsSync(path.join(proofDir, '1.json'))).toBeTruthy()

    const result3 = await eth2NearRelay.getParseBlock((epoch + 1) * numBlocksPerEpoch)
    expect(result3).toBeTruthy()

    // Check if a cache for epoch 1 is still exist
    expect(fs.existsSync(path.join(proofDir, '1.json'))).toBeTruthy()
    // Check if a cache for epoch 2 is not created
    expect(fs.existsSync(path.join(proofDir, '2.json'))).toBeFalsy()

    // Trigger computing epochs 1 and 2
    const result4 = await eth2NearRelay.getParseBlock((epoch + 1) * numBlocksPerEpoch + 26000)
    expect(result4).toBeTruthy()

    const result5 = await eth2NearRelay.getParseBlock((epoch + 2) * numBlocksPerEpoch)
    expect(result5).toBeTruthy()

    // Check if a cache for epoch 2 is created
    expect(fs.existsSync(path.join(proofDir, '2.json'))).toBeTruthy()
    // Check if outdated cache is removed
    expect(fs.existsSync(path.join(proofDir, '0.json'))).toBeFalsy()
})

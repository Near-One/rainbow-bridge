const { Eth2NearRelay } = require('./index')
const config = require('./test-config.json')
const os = require('os')
const fs = require('fs')
const path = require('path')
jest.setTimeout(1800000)

test('Computing Ethasproof epochs in advance ', async () => {
  const eth2NearRelay = new Eth2NearRelay()
  eth2NearRelay.initialize(null, config)
  const proofDir = path.join(os.homedir(), '.ethashproof')
  try {
    fs.rmSync(proofDir, { recursive: true })
  } catch (_e) {
  }

  const epoch = 401
  const numBlocksPerEpoch = 30000
  const result1 = await eth2NearRelay.getParseBlock(epoch * numBlocksPerEpoch + 26000)
  expect(result1).toBeTruthy()
  expect(fs.existsSync(path.join(proofDir, '401.json'))).toBeTruthy()

  const result2 = await eth2NearRelay.ethashproof.nextEpochPromise
  expect(result2).toBeTruthy()
  expect(fs.existsSync(path.join(proofDir, '402.json'))).toBeTruthy()

  const result3 = await eth2NearRelay.getParseBlock((epoch + 1) * numBlocksPerEpoch)
  expect(result3).toBeTruthy()
  expect(fs.existsSync(path.join(proofDir, '402.json'))).toBeTruthy()
  expect(fs.existsSync(path.join(proofDir, '403.json'))).toBeFalsy()

  const result4 = await eth2NearRelay.getParseBlock((epoch + 1) * numBlocksPerEpoch + 26000)
  expect(result4).toBeTruthy()
  expect(fs.existsSync(path.join(proofDir, '401.json'))).toBeFalsy()

  const result5 = await eth2NearRelay.getParseBlock((epoch + 2) * numBlocksPerEpoch)
  expect(result5).toBeTruthy()
  expect(fs.existsSync(path.join(proofDir, '403.json'))).toBeTruthy()
})

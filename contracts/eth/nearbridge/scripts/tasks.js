const os = require('os')
const fs = require('fs')
const path = require('path')

async function deployNearBridgeProxy (hre, args) {
  const {
    ed25519,
    ethClientLockEthAmount,
    admin,
    ethClientLockDuration,
    ethClientReplaceDuration,
    pausedFlags
  } = args

  let eClientLockDuration = Number(ethClientLockDuration)
  let eClientReplaceDuration = Number(ethClientReplaceDuration)

  // replace duration should be at least twice as long as lock duration or 20 minutes longer
  const minAllowedReplaceDuration = Math.min(
    eClientLockDuration + 20 * 60,
    2 * eClientLockDuration
  )

  if (eClientReplaceDuration < minAllowedReplaceDuration) {
    throw new Error(
      `Invalid parameters ${JSON.stringify({
        eClientLockDuration,
        eClientReplaceDuration,
        minAllowedReplaceDuration
      })}`
    )
  }

  const lockEthAmount = hre.ethers.utils.parseEther(ethClientLockEthAmount)
  const lockDuration = hre.ethers.utils.parseEther(ethClientLockDuration)

  const replaceDuration = hre.ethers.utils
    .parseEther(ethClientReplaceDuration)
    .mul(ethers.BigNumber.from("1000000000"))
  const NearBridgeFactory = await ethers.getContractFactory('NearBridge')

  // deploy the proxy contract
  const NearBridge = await upgrades.deployProxy(
    NearBridgeFactory,
    [ed25519, lockEthAmount, lockDuration, replaceDuration, pausedFlags],
    { kind: 'uups' }
  )

  await NearBridge.deployed()

  // export proxy and implementation addresses to config.json
  const p = path.join(os.homedir(), '.rainbow/config.json')
  const cfg = fs.readFileSync(p)
  const rainbowConfig = JSON.parse(cfg)

  rainbowConfig.ethClientAddress = NearBridge.address
  rainbowConfig.ethClientImplementationAddress = await upgrades.erc1967.getImplementationAddress(
    NearBridge.address
  )

  let data = JSON.stringify(rainbowConfig, null, '\t')
  data = JSON.stringify(rainbowConfig, null, 2)
  fs.writeFileSync(p, data, { flags: 'w+' })
}
exports.deployNearBridgeProxy = deployNearBridgeProxy

const os = require('os')
const fs = require('fs')
const path = require('path')

async function deployNearProverProxy (hre, args) {
  const { ethClientAddress, admin, pausedFlags } = args

  if (!ethClientAddress) {
    throw new Error('ethClientAddress not set')
  }

  const NearProverFactory = await hre.ethers.getContractFactory('NearProver')
  const NearProver = await hre.upgrades.deployProxy(
    NearProverFactory,
    [ethClientAddress, pausedFlags],
    { kind: 'uups' }
  )
  await NearProver.deployed()

  // export proxy and implementation addresses to config.json
  const p = path.join(os.homedir(), '.rainbow/config.json')
  const cfg = fs.readFileSync(p)
  const rainbowConfig = JSON.parse(cfg)

  rainbowConfig.ethProverAddress = NearProver.address
  rainbowConfig.ethProverImplementationAddress = await upgrades.erc1967.getImplementationAddress(
    NearProver.address
  )

  let data = JSON.stringify(rainbowConfig, null, '\t')
  data = JSON.stringify(rainbowConfig, null, 2)
  fs.writeFileSync(p, data, { flags: 'w+' })
}

exports.deployNearProverProxy = deployNearProverProxy

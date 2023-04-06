const os = require('os')
const fs = require('fs')
const path = require('path')
const { upgrades } = require('hardhat')

async function deployNearBridgeProxy (hre, args) {
  const {
    ed25519,
    ethclientlockethamount,
    admin,
    ethclientlockduration,
    ethclientreplaceduration,
    pausedflags
  } = args

  let eClientLockDuration = Number(ethclientlockduration)
  let eClientReplaceDuration = Number(ethclientreplaceduration)

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

  const lockEthAmount = hre.ethers.utils.parseEther(ethclientlockethamount)
  const lockDuration = hre.ethers.utils.parseEther(ethclientlockduration)

  const replaceDuration = hre.ethers.utils
    .parseEther(ethclientreplaceduration)
    .mul(ethers.BigNumber.from("1000000000"))
  const NearBridgeFactory = await ethers.getContractFactory('NearBridge')

  // deploy the proxy contract
  const NearBridge = await upgrades.deployProxy(
    NearBridgeFactory,
    [ed25519, lockEthAmount, lockDuration, replaceDuration, pausedflags],
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

async function transferOwnership(currentadmin, newadmin, bridgeaddress) {
  const p = path.join(os.homedir(), ".rainbow/config.json");
  const cfg = fs.readFileSync(p);
  const rainbowConfig = JSON.parse(cfg);
  const wallet = new ethers.Wallet(rainbowConfig.ethMasterSk);

  if (currentadmin == wallet.address) {
    const NearBridgeFactory = await ethers.getContractFactory("NearBridge");
    const NearBridgeContract = NearBridgeFactory.attach(bridgeaddress);
    const tx = await NearBridgeContract.transferOwnership(newadmin);
    const receipt = await tx.wait();
    if (receipt.status == 1) {
      console.log(
        `Transaction ${receipt.transactionHash} successfull: Ownership transferred from ${currentadmin} to ${newadmin}`
      );
    } else {
      console.log(`Transaction ${receipt.transactionHash} failed`);
    }
  } else {
    console.log("Present owner is invalid");
  }
}

module.exports = {deployNearBridgeProxy, transferOwnership};
// exports.deployNearBridgeProxy = deployNearBridgeProxy

async function deployNearProverProxy (hre, args) {
  const { abi, bytecode, privateKey, ethClientAddress, pausedFlags } = args

  const provider = new ethers.providers.Web3Provider(hre.network.provider)
  let wallet = new ethers.Wallet(privateKey, provider)

  const NearProverFactory = new ethers.ContractFactory(abi, bytecode, wallet)

  const NearProver = await upgrades.deployProxy(
    NearProverFactory,
    [ethClientAddress, pausedFlags],
    { kind: 'uups' }
  )
  await NearProver.deployed()

  console.log(
    JSON.stringify({
      proxy: NearProver.address,
      implementation: await upgrades.erc1967.getImplementationAddress(
        NearProver.address
      )
    })
  )
}

exports.deployNearProverProxy = deployNearProverProxy

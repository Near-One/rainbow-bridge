const path = require('path');

async function deployNearBridgeProxy (hre, args) {
    const {
        abi,
        bytecode,
        ed25519,
        privateKey,
        lockEthAmount,
        lockDuration,
        replaceDuration,
        admin,
        pausedFlags,
    } = args;

    const provider = new ethers.providers.Web3Provider(hre.network.provider)
    let wallet = new ethers.Wallet(privateKey, provider);

    const NearBridgeFactory = new ethers.ContractFactory(
        abi, bytecode, wallet);
    
    const NearBridge = await upgrades.deployProxy(
        NearBridgeFactory,
        [
            ed25519,
            lockEthAmount,
            lockDuration,
            replaceDuration,
            admin,
            pausedFlags,
        ],
        { kind: 'uups' },
    );
    await NearBridge.deployed();

    console.log(JSON.stringify({
        proxy: NearBridge.address,
        implementation: await upgrades.erc1967.getImplementationAddress(
            NearBridge.address,
        ),
    }));
}

exports.deployNearBridgeProxy = deployNearBridgeProxy

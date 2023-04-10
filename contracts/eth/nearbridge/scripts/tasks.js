const os = require('os');
const fs = require('fs');
const path = require('path');
const { upgrades, ethers } = require('hardhat');
const { defender } = require('hardhat');

function sleep (ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function verify (address) {
    if (hre.network.name !== 'hardhat' && hre.network.name !== 'localhost' && hre.network.name !== 'localnet') {
        let retry = 20;
        console.log('Sleeping before verification...');
        while ((await ethers.provider.getCode(address).catch(() => '')).length <= 3 && retry >= 0) {
            await sleep(5000);
            --retry;
        }
        await sleep(30000);

        await hre
            .run('verify:verify', {
                address,
            })
            .catch((error) => console.log(error));
    }
}

async function deployNearBridgeProxy (hre, args) {
    const {
        ed25519,
        ethclientlockethamount,
        admin,
        ethclientlockduration,
        ethclientreplaceduration,
        pausedflags,
        upgrader,
    } = args;

    const eClientLockDuration = Number(ethclientlockduration);
    const eClientReplaceDuration = Number(ethclientreplaceduration);

    // replace duration should be at least twice as long as lock duration or 20 minutes longer
    const minAllowedReplaceDuration = Math.min(
        eClientLockDuration + 20 * 60,
        2 * eClientLockDuration,
    );

    if (eClientReplaceDuration < minAllowedReplaceDuration) {
        throw new Error(
            `Invalid parameters ${JSON.stringify({
                eClientLockDuration,
                eClientReplaceDuration,
                minAllowedReplaceDuration,
            })}`,
        );
    }

    const lockEthAmount = hre.ethers.utils.parseEther(ethclientlockethamount);
    const lockDuration = hre.ethers.utils.parseEther(ethclientlockduration);

    const replaceDuration = hre.ethers.utils
        .parseEther(ethclientreplaceduration)
        .mul(ethers.BigNumber.from('1000000000'));
    let [signer] = await ethers.getSigners();
    const NearBridgeFactory = (await ethers.getContractFactory('NearBridge')).connect(signer);
    console.log(await ethers.getSigners());
    // deploy the proxy contract
    const NearBridge = await upgrades.deployProxy(
        NearBridgeFactory,
        [ed25519, lockEthAmount, lockDuration, replaceDuration, pausedflags, upgrader],
        { kind: 'uups' },
    );

    await NearBridge.deployed();
    console.log('Bridge Deployed at: ', NearBridge.address);
    // export proxy and implementation addresses to config.json
    const p = path.join(os.homedir(), '.rainbow/config.json');
    const cfg = fs.readFileSync(p);
    const rainbowConfig = JSON.parse(cfg);

    rainbowConfig.ethClientAddress = NearBridge.address;
    rainbowConfig.ethClientImplementationAddress = await upgrades.erc1967.getImplementationAddress(
        NearBridge.address,
    );

    let data = JSON.stringify(rainbowConfig, null, '\t');
    data = JSON.stringify(rainbowConfig, null, 2);
    fs.writeFileSync(p, data, { flags: 'w+' });
    await verify(rainbowConfig.ethClientAddress);
}

async function transferOwnership (currentadmin, newadmin, bridgeaddress) {
    const p = path.join(os.homedir(), '.rainbow/config.json');
    const cfg = fs.readFileSync(p);
    const rainbowConfig = JSON.parse(cfg);
    const wallet = new ethers.Wallet(rainbowConfig.ethMasterSk);

    if (currentadmin == wallet.address) {
        const NearBridgeFactory = await ethers.getContractFactory('NearBridge');
        const NearBridgeContract = NearBridgeFactory.attach(bridgeaddress);
        const tx = await NearBridgeContract.transferOwnership(newadmin);
        const receipt = await tx.wait();
        if (receipt.status == 1) {
            console.log(
                `Transaction ${receipt.transactionHash} successfull: Ownership transferred from ${currentadmin} to ${newadmin}`,
            );
        } else {
            console.log(`Transaction ${receipt.transactionHash} failed`);
        }
    } else {
        console.log('Present owner is invalid');
    }
}

async function proposeUpgrade (proxyaddress, newcontractname, upgrader) {
    const newContract = await ethers.getContractFactory(newcontractname);
    console.log('Preparing proposal.....');
    const proposal = await defender.proposeUpgrade(proxyaddress, newContract, {multisig: upgrader, multisigType: 'Gnosis Safe'});
    console.log('Upgrade proposal created at: ', proposal.url);
}

module.exports = { deployNearBridgeProxy, transferOwnership, proposeUpgrade };

require('dotenv').config();

const { ethers } = require('hardhat');
const { assert, expect } = require('chai');

const BRIDGE_ADDRESS_SLOT = 2;

async function upgradeProversBridgeAddressTo(provider, proverAddress, newBridgeAddress) {
    const nearProverFactory = await ethers.getContractFactory('NearProver');
    const nearProver = nearProverFactory.attach(proverAddress);

    console.log(`Got prover at address: ${proverAddress}`);

    const initialBridgeAddress = await nearProver.bridge();
    console.log(`Initial bridge address: ${initialBridgeAddress}`);
    console.log(`Trying to upgrade bridge address to: ${newBridgeAddress}`);

    const adminWallet = new ethers.Wallet(process.env.ETH_PRIVATE_KEY, provider);

    assert.equal(
        adminWallet.address,
        await nearProver.admin(),
        "The used account is not an admin of NearProver"
    );

    // Mask matches only on the latest 20 bytes (to store the address)
    const mask = ethers.BigNumber.from("0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff");
    const response = await nearProver
        .connect(adminWallet)
        .adminSstoreWithMask(BRIDGE_ADDRESS_SLOT, newBridgeAddress, mask);
    await response.wait(10).then(function(receipt) {
        console.log(`Transaction mined: `);
        console.log(receipt);
    });

    const bridgeAddressAfterUpgrade = await nearProver.bridge();
    console.log(`Bridge address after upgrade: ${bridgeAddressAfterUpgrade}`);
    expect(bridgeAddressAfterUpgrade.toLowerCase())
        .to
        .equal(newBridgeAddress.toLowerCase());
}

async function getProversBridgeAddress(proverAddress) {
    const nearProverFactory = await ethers.getContractFactory('NearProver');
    const nearProver = nearProverFactory.attach(proverAddress);
    const bridgeAddress = await nearProver.bridge();

    return bridgeAddress;
}

exports.upgradeProversBridgeAddressTo = upgradeProversBridgeAddressTo;
exports.getProversBridgeAddress = getProversBridgeAddress;


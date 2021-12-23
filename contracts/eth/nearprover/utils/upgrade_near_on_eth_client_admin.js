require('dotenv').config();

const { ethers } = require('hardhat');
const { assert, expect } = require('chai');

const ADMIN_ADDRESS_SLOT = 0;

async function upgradeNearOnEthClientAdminTo (provider, bridgeAddress, newAdminAddress, ledgerKeyPath) {
    const nearBridgeFactory = await ethers.getContractFactory('NearBridge');
    const nearBridge = nearBridgeFactory.attach(bridgeAddress);

    console.log(`Current NearOnEth client admin: ${await nearBridge.admin()}`);
    console.log(`Trying to upgrade admin address to: ${newAdminAddress}`);

    let signer;
    // We use non-strict unequality as it also includes undefined, 0, etc
    if (ledgerKeyPath != null) {
        signer = new EthLedgerSigner(provider, ledgerKeyPath);
    } else {
        signer = new ethers.Wallet(process.env.ETH_PRIVATE_KEY, provider);
    }

    assert.equal(
        await signer.getAddress(),
        await nearBridge.admin(),
        'The used account is not an admin of NearBridge',
    );

    //// Mask matches only on the latest 20 bytes (to store the address)
    const mask = ethers.BigNumber.from('0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff');
    const response = await nearBridge
        .connect(signer)
        .adminSstoreWithMask(ADMIN_ADDRESS_SLOT, newAdminAddress, mask);
    console.log(response);
    console.log('Waiting for tx confirmation...');
    await response.wait(10).then(function (receipt) {
        console.log('Transaction mined: ');
        console.log(receipt);
    });

    const adminAddressAfterUpgrade = await nearBridge.admin();
    console.log(`Bridge address after upgrade: ${adminAddressAfterUpgrade}`);
    expect(adminAddressAfterUpgrade.toLowerCase())
        .to
        .equal(newAdminAddress.toLowerCase());
}

exports.upgradeNearOnEthClientAdminTo = upgradeNearOnEthClientAdminTo;

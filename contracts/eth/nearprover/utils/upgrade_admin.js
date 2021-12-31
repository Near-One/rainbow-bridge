const { ethers } = require('hardhat');
const { assert, expect } = require('chai');
const { LedgerSigner } = require('@ethersproject/hardware-wallets');
const readlineSync = require('readline-sync');
const { upgradeAddressAtSlotLegacy } = require('./utils');

async function getSlotsData (provider, contractAddress, numOfSlotsToDisplay) {
    console.log(`Contract's ${contractAddress} slots:`);
    for (let i = 0; i < numOfSlotsToDisplay; ++i) {
        const slotData = await provider.getStorageAt(contractAddress, i);
        console.log(`Slot ${i}: ${slotData}`);
    }
}

async function upgradeAdminAddressTo ({
    provider,
    contractAddress,
    currentAdminAddress,
    newAdminAddress,
    adminAddressSlot,
    ledgerKeyPath,
}) {
    const slotContent = ethers.BigNumber.from(await provider.getStorageAt(contractAddress, Number(adminAddressSlot))).toHexString();
    assert.equal(
        slotContent.toUpperCase(),
        currentAdminAddress.toUpperCase(),
        `The current admin doesn't match at the slot ${adminAddressSlot} contract ${contractAddress}`,
    );

    const adminControlledFactory = await ethers.getContractFactory('AdminControlled');
    const adminControlled = adminControlledFactory.attach(contractAddress);

    await getSlotsData(provider, contractAddress, 5);
    console.log(`Selected slot: ${adminAddressSlot}`);
    console.log(`Selected slot content: ${slotContent}`);
    console.log(`Current admin: ${await adminControlled.admin()}`);
    console.log(`Trying to upgrade admin address to: ${newAdminAddress}`);

    let signer;
    // We use non-strict unequality as it also includes undefined, etc
    if (ledgerKeyPath != null) {
        signer = new LedgerSigner(provider, 'hid', ledgerKeyPath || null);
    } else {
        signer = new ethers.Wallet(process.env.ETH_PRIVATE_KEY, provider);
    }

    assert.equal(
        await signer.getAddress(),
        await adminControlled.admin(),
        `The used account is not an admin of contract ${contractAddress}`,
    );

    const inputResult = readlineSync.question('WARRING! Please verify all data. This change can not be reverted,' +
        ' do you confirm that you are aware of this and want to change the admin address?\n Enter CONFIRM if yes: ');

    if (inputResult.toUpperCase() !== 'CONFIRM') {
        console.log('The task was aborted');
        return;
    }

    const response = await upgradeAddressAtSlotLegacy(provider, signer, adminControlled, newAdminAddress, adminAddressSlot);
    console.log(response);
    console.log('Waiting for tx confirmation...');
    await response.wait(5).then(function (receipt) {
        console.log('Transaction mined: ');
        console.log(receipt);
    });

    const adminAddressAfterUpgrade = await adminControlled.admin();
    console.log(`Admin address after upgrade: ${adminAddressAfterUpgrade}`);
    expect(adminAddressAfterUpgrade.toLowerCase())
        .to
        .equal(newAdminAddress.toLowerCase());
}

exports.getSlotsData = getSlotsData;
exports.upgradeAdminAddressTo = upgradeAdminAddressTo;

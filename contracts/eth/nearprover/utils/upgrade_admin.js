const { ethers } = require('hardhat');
const { assert, expect } = require('chai');
const { EthLedgerSigner } = require('./eth-ledger-signer');

async function getSlotsData (provider, contractAddress, numOfSlotsToDisplay) {
    for (let i = 0; i < numOfSlotsToDisplay; ++i) {
        const slotData = await provider.getStorageAt(contractAddress, i);
        console.log(slotData);
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
    const currentAdmin = ethers.BigNumber.from(await provider.getStorageAt(contractAddress, Number(adminAddressSlot))).toHexString();
    assert.equal(
        currentAdmin,
        currentAdminAddress,
        `The current admin doesn't match at the slot ${adminAddressSlot} contract ${contractAddress}`,
    );

    const adminControlledFactory = await ethers.getContractFactory('AdminControlled');
    const adminControlled = adminControlledFactory.attach(contractAddress);

    console.log(`Current admin: ${await adminControlled.admin()}`);
    console.log(`Trying to upgrade admin address to: ${newAdminAddress}`);

    let signer;
    // We use non-strict unequality as it also includes undefined, etc
    if (ledgerKeyPath != null) {
        signer = new EthLedgerSigner(provider, ledgerKeyPath);
    } else {
        signer = new ethers.Wallet(process.env.ETH_PRIVATE_KEY, provider);
    }

    assert.equal(
        await signer.getAddress(),
        await adminControlled.admin(),
        `The used account is not an admin of contract ${contractAddress}`,
    );

    // Mask matches only on the latest 20 bytes (to store the address)
    const mask = ethers.BigNumber.from('0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff');
    const response = await adminControlled
        .connect(signer)
        .adminSstoreWithMask(adminAddressSlot, newAdminAddress, mask);
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

require('dotenv').config();

const { ethers } = require('hardhat');
const { assert, expect } = require('chai');
const { LedgerSigner } = require('@ethersproject/hardware-wallets');

const BRIDGE_ADDRESS_SLOT = 2;

async function upgradeProversBridgeAddressTo (provider, proverAddress, newBridgeAddress, ledgerKeyPath) {
    const nearProverFactory = await ethers.getContractFactory('NearProver');
    const nearProver = nearProverFactory.attach(proverAddress);

    console.log(`Got prover at address: ${proverAddress}`);

    const initialBridgeAddress = await nearProver.bridge();
    console.log(`Initial bridge address: ${initialBridgeAddress}`);
    console.log(`Trying to upgrade bridge address to: ${newBridgeAddress}`);

    let signer;
    // We use non-strict unequality as it also includes undefined, 0, etc
    if (ledgerKeyPath != null) {
        signer = new LedgerSigner(provider, undefined, ledgerKeyPath);
    } else {
        signer = new ethers.Wallet(process.env.ETH_PRIVATE_KEY, provider);
    }

    assert.equal(
        await signer.getAddress(),
        await nearProver.admin(),
        'The used account is not an admin of NearProver',
    );

    // Mask matches only on the latest 20 bytes (to store the address)
    const mask = ethers.BigNumber.from('0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff');
    const options = {
        gasLimit: 50000,
        gasPrice: 150000000000, // 150 Gwei
    };
    const tx = await nearProver
        .connect(signer)
        .populateTransaction
        .adminSstoreWithMask(BRIDGE_ADDRESS_SLOT, newBridgeAddress, mask, options);
    tx.nonce = await provider.getTransactionCount(tx.from);
    console.log(tx);
    const signedTx = await signer.signTransaction(tx);
    console.log(signedTx);
    const response = await provider.sendTransaction(signedTx);
    console.log(response);
    console.log('Waiting for tx confirmation...');
    await response.wait(5).then(function (receipt) {
        console.log('Transaction mined: ');
        console.log(receipt);
    });

    const bridgeAddressAfterUpgrade = await nearProver.bridge();
    console.log(`Bridge address after upgrade: ${bridgeAddressAfterUpgrade}`);
    expect(bridgeAddressAfterUpgrade.toLowerCase())
        .to
        .equal(newBridgeAddress.toLowerCase());
}

async function getProversBridgeAddress (proverAddress) {
    const nearProverFactory = await ethers.getContractFactory('NearProver');
    const nearProver = nearProverFactory.attach(proverAddress);
    const bridgeAddress = await nearProver.bridge();

    return bridgeAddress;
}

exports.upgradeProversBridgeAddressTo = upgradeProversBridgeAddressTo;
exports.getProversBridgeAddress = getProversBridgeAddress;

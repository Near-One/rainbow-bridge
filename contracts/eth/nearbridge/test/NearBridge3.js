const { ethers } = require('hardhat');
const { expect } = require('chai');

const { borshify, borshifyInitialValidators } = require('rainbow-bridge-lib/rainbow/borsh')

let Ed25519, NearBridge;

beforeEach(async function () {
    Ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
    NearBridge = await (await ethers.getContractFactory('NearBridge')).deploy(
        Ed25519.address,
        ethers.BigNumber.from("1000000000000000000"), // 1e18
        ethers.BigNumber.from("10"), // lock duration
        ethers.BigNumber.from("20000000000"), // replace duration
        await (await ethers.getSigners())[0].getAddress(),
        0
    );
    await NearBridge.deposit({ value: ethers.utils.parseEther('1') });
});

it('Verify init with validators V1', async function () {
    const initialValidators = borshifyInitialValidators(require('./block_validators_v1_testnet.json').next_bps);
    console.log(`Initial validators: ${initialValidators.toString('hex')}`);

    expect(await NearBridge.initWithValidators(borshifyInitialValidators(require('./block_validators_v1_testnet.json').next_bps)));
});


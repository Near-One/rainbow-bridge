const { ethers } = require('hardhat');
const { expect } = require('chai');

const { borshify, borshifyInitialValidators } = require('rainbow-bridge-utils')


describe('New tests', () => {
    let Ed25519;
    let NearBridge;

    beforeEach(async function () {
        Ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
        const NearBridgeFactory = await ethers.getContractFactory('NearBridge');
        NearBridge = await upgrades.deployProxy(NearBridgeFactory, [
            Ed25519.address,
            ethers.BigNumber.from("1000000000000000000"), // 1e18
            ethers.BigNumber.from("10"), // lock duration
            ethers.BigNumber.from("20000000000"), // replace duration
            0
        ], { kind: 'uups' });
        await NearBridge.deposit({ value: ethers.utils.parseEther('1') });
    });

    it('Verify init with validators V1', async function () {
        const initialValidators = borshifyInitialValidators(require('./block_validators_v1_testnet.json').next_bps);

        expect(await NearBridge.initWithValidators(borshifyInitialValidators(require('./block_validators_v1_testnet.json').next_bps)));
    });
});


describe('NearBridge with admin access', () => {
    const ADMIN_ADDRESS_SLOT = 0;
    let nearBridge;
    let adminAccount;

    beforeEach(async () => {
        ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
        [deployerAccount] = await ethers.getSigners();

        // Make the deployer admin
        adminAccount = deployerAccount;

        nearBridge = await (await ethers.getContractFactory('NearBridge')).deploy(
            ed25519.address,
            ethers.BigNumber.from("1000000000000000000"), // 1e18
            ethers.BigNumber.from("10"), // lock duration
            ethers.BigNumber.from("20000000000"), // replace duration
            await adminAccount.getAddress(),
            0
        );
    });

    describe('Upgradability', async () => {
        it('should upgrade the admin address from the provided hex string', async () => {
            const initialAdminAddress = await nearBridge.admin();
            expect(initialAdminAddress)
                .to
                .equal(await adminAccount.address);

            const newAdminAddress = '0x0123456789abcdefcafedeadbeefbea77a1de456';
            expect(newAdminAddress)
                .not
                .equal(initialAdminAddress);

            // Mask matches only on the latest 20 bytes (to store the address)
            const mask = ethers.BigNumber.from("0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff");
            nearBridge.adminSstoreWithMask(ADMIN_ADDRESS_SLOT, newAdminAddress, mask);

            expect((await nearBridge.admin()).toLowerCase())
                .to
                .equal(newAdminAddress);
        });
    });
});

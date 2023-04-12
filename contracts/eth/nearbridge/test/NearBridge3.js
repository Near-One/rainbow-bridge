const { ethers } = require('hardhat');
const { expect } = require('chai');

const { borshify, borshifyInitialValidators } = require('rainbow-bridge-utils')


describe('New tests', () => {
    let Ed25519;
    let NearBridge;

    beforeEach(async function () {
        Ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
        const NearBridgeFactory = await ethers.getContractFactory('NearBridge');
        [AdminWallet] = await ethers.getSigners();
        NearBridge = await upgrades.deployProxy(NearBridgeFactory, [
            Ed25519.address,
            ethers.BigNumber.from("1000000000000000000"), // 1e18
            ethers.BigNumber.from("10"), // lock duration
            ethers.BigNumber.from("20000000000"), // replace duration
            0,
            AdminWallet.address
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
    const UNPAUSED_ALL = 0;
    const PAUSED_DEPOSIT = 1 << 0;
    const PAUSED_WITHDRAW = 1 << 1;
    const PAUSED_ADD_BLOCK = 1 << 2;
    const PAUSED_CHALLENGE = 1 << 3;
    const PAUSED_VERIFY = 1 << 4;

    let nearBridge;
    let adminAccount;
    let userAccount1;
    let userAccount2;
    let newAdminAccount;

    beforeEach(async () => {
        ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
        [deployerAccount, userAccount1, userAccount2, newAdminAccount] = await ethers.getSigners();

        // Make the deployer admin
        adminAccount = deployerAccount;

        const NearBridgeFactory = await ethers.getContractFactory('NearBridge');
        nearBridge = await upgrades.deployProxy(NearBridgeFactory, [
            ed25519.address,
            ethers.BigNumber.from("1000000000000000000"), // 1e18
            ethers.BigNumber.from("10"), // lock duration
            ethers.BigNumber.from("20000000000"), // replace duration
            0,
            adminAccount.address
        ], { kind: 'uups' });
    });

    describe('AdminControlled', async () => {
        it('Admin account matches', async() => {
            expect(
                await nearBridge.admin()
            )
                .to
                .be
                .equal(adminAccount.address);
        });

        it('regular user can not perform admin functions', async() => {
            const recipientBalanceBefore = ethers.BigNumber.from(await ethers.provider.getBalance(userAccount2.address));
            const contractBalanceBefore = ethers.BigNumber.from(await ethers.provider.getBalance(nearBridge.address));

            const amountToTransfer = 4000; // wei
            // user1 tries to perform `adminSendEth()` to replenish user2 balance
            await expect(
                nearBridge
                    .connect(userAccount1)
                    .adminSendEth(userAccount2.address, amountToTransfer)
            )
                .to
                .be
                .reverted;

            const recipientBalanceAfter = ethers.BigNumber.from(await ethers.provider.getBalance(userAccount2.address));
            const contractBalanceAfter = ethers.BigNumber.from(await ethers.provider.getBalance(nearBridge.address));

            expect(recipientBalanceAfter)
                .to
                .be
                .equal(recipientBalanceBefore);
            expect(contractBalanceAfter)
                .to
                .be
                .equal(contractBalanceBefore);

            // Try to pause and unpause
            await expect(nearBridge.connect(userAccount1).adminPause(PAUSED_DEPOSIT)).to.be.reverted;
            await expect(nearBridge.connect(userAccount1).adminPause(PAUSED_WITHDRAW)).to.be.reverted;
            await expect(nearBridge.connect(userAccount1).adminPause(PAUSED_ADD_BLOCK)).to.be.reverted;
            await expect(nearBridge.connect(userAccount1).adminPause(PAUSED_CHALLENGE)).to.be.reverted;
            await expect(nearBridge.connect(userAccount1).adminPause(PAUSED_VERIFY)).to.be.reverted;
            await expect(nearBridge.connect(userAccount1).adminPause(UNPAUSED_ALL)).to.be.reverted;
            // ------------------------------------------

            // Try to use adminSstore
            await expect(
                nearBridge
                    .connect(userAccount1)
                    .adminSstore(0, 1)
            )
                .to
                .be
                .reverted;

            // Try to use adminSstoreWithMask
            await expect(
                nearBridge
                    .connect(userAccount1)
                    .adminSstoreWithMask(0, 1, ethers.BigNumber.from('0x0000ffff'))
            )
                .to
                .be
                .reverted;
        });

        it('admin receive eth and transfer eth', async () => {
            const replenishBalanceValue = 1_500_000;

            const options = { value: replenishBalanceValue };
            await nearBridge
                .connect(adminAccount)
                .adminReceiveEth(options);

            const recipientBalanceBefore = ethers.BigNumber.from(await ethers.provider.getBalance(userAccount2.address));
            const contractBalanceBefore = ethers.BigNumber.from(await ethers.provider.getBalance(nearBridge.address));

            // Check the contract has the specified balance available
            expect(contractBalanceBefore)
                .to
                .be
                .equal(replenishBalanceValue);

            // Send eth using admin access
            const amountToTransfer = 4000; // wei
            await nearBridge
                .connect(adminAccount)
                .adminSendEth(userAccount2.address, amountToTransfer);

            const recipientBalanceAfter = ethers.BigNumber.from(await ethers.provider.getBalance(userAccount2.address));
            const contractBalanceAfter = ethers.BigNumber.from(await ethers.provider.getBalance(nearBridge.address));

            expect(recipientBalanceAfter)
                .to
                .be
                .equal(recipientBalanceBefore.add(amountToTransfer));
            expect(contractBalanceAfter)
                .to
                .be
                .equal(contractBalanceBefore.sub(amountToTransfer));
        });

        it('should upgrade the admin address from the provided hex string using (using `adminSstoreWithMask()`)', async function () {
            //skip this till admin slot with new upgradable contract is found
            this.skip();

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
            await nearBridge.adminSstoreWithMask(ADMIN_ADDRESS_SLOT, newAdminAddress, mask);

            expect((await nearBridge.admin()).toLowerCase())
                .to
                .equal(newAdminAddress);
        });
    });
});

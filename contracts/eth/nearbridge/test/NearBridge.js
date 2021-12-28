const { expect } = require('chai');
const { ethers, upgrades, network } = require('hardhat');

const fs = require('fs').promises;
const { borshify, borshifyInitialValidators } = require('rainbow-bridge-utils');

async function increaseTime (time) {
    await network.provider.send('evm_increaseTime', [time]);
    await network.provider.send('evm_mine', []);
}

let Ed25519, NearBridge, accounts, AdminWallet;
beforeEach(async function () {
    accounts = await ethers.getSigners();
    Ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
    [AdminWallet] = await ethers.getSigners();
    const NearBridgeFactory = await ethers.getContractFactory('NearBridge');
    NearBridge = await upgrades.deployProxy(NearBridgeFactory, [
        Ed25519.address,
        ethers.BigNumber.from('1000000000000000000'), // 1e18
        ethers.BigNumber.from('360'), // lock duration
        ethers.BigNumber.from('362627730000'), // replace duration
        0,
    ], { kind: 'uups' });
    await NearBridge.deployed();
    await NearBridge.deposit({ value: ethers.utils.parseEther('1') });
});

it('upgrade the proxy', async function () {
    const NearBridgeV2Factory = await ethers.getContractFactory('NearBridgeV2');
    NearBridge = await upgrades.upgradeProxy(NearBridge.address, NearBridgeV2Factory);
    expect(await NearBridge.version()).eq('2.0.0');
});

it('transfer contract ownership', async function () {
    expect(await NearBridge.transferOwnership(accounts[1].address))
        .emit(NearBridge, 'OwnershipTransferred')
        .withArgs(accounts[0].address, accounts[1].address);

    const pauseRole = await NearBridge.PAUSE_ROLE();
    const adminRole = await NearBridge.DEFAULT_ADMIN_ROLE();
    expect(await NearBridge.hasRole(pauseRole, accounts[1].address)).ok;
    expect(await NearBridge.hasRole(adminRole, accounts[1].address)).ok;

    expect(await NearBridge.hasRole(pauseRole, accounts[0].address)).not.ok;
    expect(await NearBridge.hasRole(adminRole, accounts[0].address)).not.ok;
});

it('Fail to upgrade contract, Unauthorized', async function () {
    expect(await NearBridge.transferOwnership(accounts[1].address))
        .emit(NearBridge, 'OwnershipTransferred')
        .withArgs(accounts[0].address, accounts[1].address);
    
    const NearBridgeV2Factory = await ethers.getContractFactory('NearBridgeV2');
    await expect(upgrades.upgradeProxy(NearBridge.address, NearBridgeV2Factory))
        .reverted;
});

it('Set setLockEthAmount', async function () {
    expect(await NearBridge.setLockEthAmount(100))
    await expect(NearBridge.setLockEthAmount(101))
    .revertedWith('The lockEthAmount have to be an even and positive number')
});

it.only('should be ok', async function () {
    const block120998 = borshify(require('./block_120998.json'));
    const block121498 = borshify(require('./block_121498.json'));
    const block121998 = borshify(require('./block_121998.json'));

    // We should use previous epoch's next_bps to initWithBlock with block_120998, but they happens to be same
    await NearBridge.initWithValidators(borshifyInitialValidators(require('./block_120998.json').next_bps));
    await NearBridge.initWithBlock(block120998);
    expect(await NearBridge.blockHashes(120998)).to.be.equal(
        '0x1a7a07b5eee1f4d8d7e47864d533143972f858464bacdc698774d167fb1b40e6',
    );

    await NearBridge.addLightClientBlock(block121498);
    expect(await NearBridge.checkBlockProducerSignatureInHead(0)).to.be.true;

    await expect(NearBridge.addLightClientBlock(block121998)).to.be.revertedWith('Epoch id of the block is not valid');
    await increaseTime(3600);
    expect(await NearBridge.blockHashes(121498)).to.be.equal(
        '0x508307e7af9bdbb297afa7af0541130eb32f0f028151319f5a4f7ae68b0ecc56',
    );

    await NearBridge.addLightClientBlock(block121998);
    expect(await NearBridge.checkBlockProducerSignatureInHead(0)).to.be.true;

    await increaseTime(3600);
    expect(await NearBridge.blockHashes(121998)).to.be.equal(
        '0x2358c4881bbd111d2e4352b6a7e6c7595fb39d3c9897d3c624006be1ef809abf',
    );
});

if (process.env.NEAR_HEADERS_DIR) {
    it('ok with many block headers', async function () {
        this.timeout(0);
        const blockFiles = await fs.readdir(process.env.NEAR_HEADERS_DIR);
        blockFiles.sort((a, b) => a.split('.')[0] - b.split('.')[0]);
        const firstBlock = require(process.env.NEAR_HEADERS_DIR + '/' + blockFiles[0]);
        const firstBlockBorsh = borshify(firstBlock);
        // current bps happens to equal to next_bps
        await NearBridge.initWithValidators(borshifyInitialValidators(firstBlock.next_bps));
        await NearBridge.initWithBlock(firstBlockBorsh);
        await NearBridge.blockHashes(firstBlock.inner_lite.height);
        expect(await NearBridge.blockHashes(firstBlock.inner_lite.height)).to.be.a('string');

        for (let i = 1; i < blockFiles.length; i++) {
            const block = require(process.env.NEAR_HEADERS_DIR + '/' + blockFiles[i]);
            const blockBorsh = borshify(block);
            console.log('adding block ' + block.inner_lite.height);
            await NearBridge.addLightClientBlock(blockBorsh);
            await NearBridge.blockHashes(block.inner_lite.height);
            await increaseTime(3600);

            if (i >= 600) {
                console.log('checking block ' + block.inner_lite.height);
                for (let j = 0; j < block.approvals_after_next.length; j++) {
                    console.log('checking approval ' + j);
                    if (block.approvals_after_next[j]) {
                        console.log('approval ' + j + ' is not null');
                        expect(await NearBridge.checkBlockProducerSignatureInHead(j)).to.be.true;
                    }
                }
            }
        }
    });
}

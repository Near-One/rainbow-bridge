const { expect } = require('chai');
const { ethers, upgrades, network } = require('hardhat');

const { borshify, borshifyInitialValidators } = require('rainbow-bridge-utils');

async function increaseTime (time) {
    await network.provider.send('evm_increaseTime', [time]);
    await network.provider.send('evm_mine', []);
}

const gasUsage = {};
describe('Test gas required by calls through proxy', () => {
    let Ed25519, NearBridgeFactory2, defaultAdmin;
    let NearBridgeAsProxy;
    before(async () => {
        [defaultAdmin] = await ethers.getSigners();
        Ed25519 = await (await ethers.getContractFactory('Ed25519'))
            .connect(defaultAdmin)
            .deploy();
        NearBridgeFactory2 = await ethers.getContractFactory('NearBridge');
        NearBridgeAsProxy = await upgrades.deployProxy(
            NearBridgeFactory2,
            [
                Ed25519.address,
                ethers.BigNumber.from('1000000000000000000'), // 1e18
                ethers.BigNumber.from('10'), // lock duration
                ethers.BigNumber.from('20000000000'), // replace duration
                0,
            ],
            { kind: 'uups' },
        );
        await NearBridgeAsProxy.deployed();
        await NearBridgeAsProxy.deposit({ value: ethers.utils.parseEther('1') });
    });

    it('should add block to the Bridge deployed as proxy', async function () {
        const block9605 = borshify(require('./block_9605.json'));
        const block9610 = borshify(require('./block_9610.json'));

        // We don't know block producers that produce block_9605, assume it's same as block_9605.next_bps
        await NearBridgeAsProxy.initWithValidators(
            borshifyInitialValidators(require('./block_9605.json').next_bps),
        );
        await NearBridgeAsProxy.initWithBlock(block9605);
        await NearBridgeAsProxy.blockHashes(9605);
        expect(await NearBridgeAsProxy.blockHashes(9605)).to.be.equal(
            '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
        );

        const tx = await NearBridgeAsProxy.addLightClientBlock(block9610);
        const reciept = await tx.wait();
        console.log(
            'Gas used by \'addLightClientBlock\' method with proxy is : ',
            reciept.gasUsed.toNumber(),
        );
        gasUsage.withProxy = reciept.gasUsed.toNumber();
        await increaseTime(10);
        expect(await NearBridgeAsProxy.blockHashes(9610)).to.be.equal(
            '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
        );
    });
});

describe('Test gas required by calls without proxy', () => {
    let Ed25519, NearBridgeFactory, defaultAdmin;
    let NearBridge;
    before(async () => {
        [defaultAdmin] = await ethers.getSigners();
        Ed25519 = await (await ethers.getContractFactory('Ed25519'))
            .connect(defaultAdmin)
            .deploy();

        NearBridgeFactory = await ethers.getContractFactory('NearBridge');
        NearBridge = await NearBridgeFactory.connect(defaultAdmin).deploy();
        await NearBridge.deployed();

        await NearBridge.initialize(
            Ed25519.address,
            ethers.BigNumber.from('1000000000000000000'), // 1e18
            ethers.BigNumber.from('10'), // lock duration
            ethers.BigNumber.from('20000000000'), // replace duration
            0,
        );

        await NearBridge.deposit({ value: ethers.utils.parseEther('1') });
    });

    it('should add block to the Bridge deployed without proxy', async function () {
        const block9605 = borshify(require('./block_9605.json'));
        const block9610 = borshify(require('./block_9610.json'));

        // We don't know block producers that produce block_9605, assume it's same as block_9605.next_bps
        await NearBridge.initWithValidators(
            borshifyInitialValidators(require('./block_9605.json').next_bps),
        );
        await NearBridge.initWithBlock(block9605);
        await NearBridge.blockHashes(9605);
        expect(await NearBridge.blockHashes(9605)).to.be.equal(
            '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
        );

        const tx = await NearBridge.addLightClientBlock(block9610);
        const rec = await tx.wait();
        console.log(
            'Gas used by \'addLightClientBlock\' method without proxy is : ',
            rec.gasUsed.toNumber(),
        );
        gasUsage.withoutProxy = Number(rec.gasUsed);
        await increaseTime(10);
        expect(await NearBridge.blockHashes(9610)).to.be.equal(
            '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
        );
        if (gasUsage.withProxy >= gasUsage.withProxy) {
            console.log(
                `Call 'addLightClientBlock' through proxy contract consumes ${gasUsage.withProxy} gas which is >  normal call ${gasUsage.withoutProxy} gas`,
            );
            console.log(
                `Gas difference: ${gasUsage.withProxy - gasUsage.withoutProxy}`,
            );
        } else if (gasUsage.withProxy < gasUsage.withProxy) {
            console.log(
                `Call 'addLightClientBlock' through without proxy contract consumes ${gasUsage.withoutProxy} gas which is > call through proxy contract ${gasUsage.withProxy} gas`,
            );
            console.log(
                `Gas difference: ${gasUsage.withoutProxy - gasUsage.withProxy} gas`,
            );
        } else {
            console.log(
                'Both call \'addLightClientBlock\' from proxy and without proxy consumes same gas usage',
            );
        }
    });
});

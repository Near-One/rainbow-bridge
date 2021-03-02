const { ethers } = require('hardhat');
const { expect } = require('chai');

const { time } = require('@openzeppelin/test-helpers');
const { borshify, borshifyInitialValidators } = require('rainbow-bridge-lib/rainbow/borsh')

describe('NearBridge2', () => {
    let ed;
    let bridge;
    let decoder;

    beforeEach(async function () {
        const Ed25519Factory = await ethers.getContractFactory('Ed25519');
        ed = await Ed25519Factory.deploy();
        await ed.deployed();

        const NearBridgeFactory = await ethers.getContractFactory('NearBridge');
        bridge = await NearBridgeFactory.deploy(
            ed.address,
            ethers.BigNumber.from("1000000000000000000"), // 1e18
            ethers.BigNumber.from("10"), // lock duration
            ethers.BigNumber.from("20000000000"), // replace duration
            ethers.constants.AddressZero
        );
        await bridge.deployed();
        await bridge.deposit({ value: ethers.utils.parseEther('1') });

        const NearDecoderFactory = await ethers.getContractFactory('NearDecoder');
        decoder = await NearDecoderFactory.deploy();
        await decoder.deployed();
    });

    it('should be ok', async function () {
        const block9605 = borshify(require('./block_9605.json'));
        const block9610 = borshify(require('./block_9610.json'));

        // We don't know block producers that produce block_9605, assume it's same as block_9605.next_bps
        await bridge.initWithValidators(borshifyInitialValidators(require('./block_9605.json').next_bps));
        await bridge.initWithBlock(block9605);
        await bridge.blockHashes(9605);
        expect(await bridge.blockHashes(9605)).to.be.equal(
            '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
        );

        await bridge.addLightClientBlock(block9610);
        await time.increase(10);
        expect(await bridge.blockHashes(9610)).to.be.equal(
            '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
        );

    });

    it('2020-09-09 Example', async function () {
       const block_15178713 = borshify(require('./block_15178713.json'));
       const block_15178760 = borshify(require('./block_15178760.json'));
       const block_15204402 = borshify(require('./block_15204402.json'));
       const block_15248583 = borshify(require('./block_15248583.json'));

       await bridge.initWithValidators(borshifyInitialValidators(require('./init_validators_15178713.json')));
       await bridge.initWithBlock(block_15178713);
       await time.increase(3600);
       await bridge.addLightClientBlock(block_15178760);
       await time.increase(3600);
       await bridge.addLightClientBlock(block_15204402);
       await time.increase(3600);
       await bridge.addLightClientBlock(block_15248583);

    });

    it('Add second block in first epoch should be verifiable', async function () {
        // Get "initial validators" that will produce block 304
        const block244 = require('./244.json');
        const initialValidators = block244.next_bps;

        const block304 = require('./304.json');
        const block308 = require('./308.json');

        await bridge.initWithValidators(borshifyInitialValidators(initialValidators));
        await bridge.initWithBlock(borshify(block304));
        await bridge.blockHashes(304);

        await time.increase(3600);

        await bridge.addLightClientBlock(borshify(block308));
        await bridge.blockHashes(308);

        for (let i = 0; i < block308.approvals_after_next.length; i++) {
            if (block308.approvals_after_next[i]) {
                expect(await bridge.checkBlockProducerSignatureInHead(i)).to.be.true;
            }
        }
    });

    it('Test adding blocks in new epoch when bps change', async function () {
        const block181 = require('./181.json');
        const block244 = require('./244.json');
        const block304 = require('./304.json');
        const block308 = require('./308.json');
        const block368 = require('./368.json');
        const block369 = require('./369.json');

        await bridge.initWithValidators(borshifyInitialValidators(block181.next_bps));
        await bridge.initWithBlock(borshify(block244));
        await bridge.blockHashes(244);

        await time.increase(3600);

        await bridge.addLightClientBlock(borshify(block304));
        await bridge.blockHashes(304);

        await time.increase(3600);

        await bridge.addLightClientBlock(borshify(block308));
        await bridge.blockHashes(308);

        await time.increase(3600);

        await bridge.addLightClientBlock(borshify(block368));
        await bridge.blockHashes(368);

        await time.increase(3600);

        await bridge.addLightClientBlock(borshify(block369));
        await bridge.blockHashes(369);

    });

    it('After challenge prev should be revert to prev epoch of latest valid block', async function () {
        const block181 = require('./181.json');
        const block244 = require('./244.json');
        const block304 = require('./304.json');
        const block308 = require('./308.json');
        const block368 = require('./368.json');

        await bridge.initWithValidators(borshifyInitialValidators(block181.next_bps));
        await bridge.initWithBlock(borshify(block244));
        await bridge.blockHashes(244);

        await time.increase(3600);
        await bridge.addLightClientBlock(borshify(block304));
        await bridge.blockHashes(304);

        await time.increase(3600);
        await bridge.addLightClientBlock(borshify(block308));
        await bridge.blockHashes(308);

        await time.increase(3600);

        block368.approvals_after_next[0] = block368.approvals_after_next[1];
        await bridge.addLightClientBlock(borshify(block368));
        await bridge.blockHashes(368);
        expect((await bridge.lastValidAt())).to.not.be.equal(0);

        await bridge.challenge(ethers.constants.AddressZero, 0)
        expect((await bridge.lastValidAt())).to.be.equal(0);

    });
});
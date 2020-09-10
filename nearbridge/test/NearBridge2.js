
const { time } = require('@openzeppelin/test-helpers');
const { borshify, borshifyInitialValidators } = require('rainbow-bridge-lib/rainbow/borsh')

const Ed25519 = artifacts.require('Ed25519');
const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

async function timeIncreaseTo(seconds) {
    const delay = 1000 - new Date().getMilliseconds();
    await new Promise(resolve => setTimeout(resolve, delay));
    await time.increaseTo(seconds);
}

contract('NearBridge2', function ([_, addr1]) {
    beforeEach(async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(10));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });
    });

    it('should be ok', async function () {
        const block9605 = borshify(require('./block_9605.json'));
        const block9610 = borshify(require('./block_9610.json'));

        // We don't know block producers that produce block_9605, assume it's same as block_9605.next_bps
        await this.bridge.initWithValidators(borshifyInitialValidators(require('./block_9605.json').next_bps));
        await this.bridge.initWithBlock(block9605);
        await this.bridge.blockHashes(9605);
        expect(await this.bridge.blockHashes(9605)).to.be.equal(
            '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
        );

        await this.bridge.addLightClientBlock(block9610);
        expect(await this.bridge.blockHashes(9610)).to.be.equal(
            '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
        );
    });
});

contract('2020-09-09 Example', function ([_, addr1]) {
   beforeEach(async function () {
       this.decoder = await NearDecoder.new();
       this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(10));
       await this.bridge.deposit({ value: web3.utils.toWei('1') });
   });

   it('should be ok', async function () {
       const block_15178713 = borshify(require('./block_15178713.json'));
       const block_15178760 = borshify(require('./block_15178760.json'));
       const block_15204402 = borshify(require('./block_15204402.json'));
       const block_15205393 = borshify(require('./block_15205393.json'));

       await this.bridge.initWithValidators(borshifyInitialValidators(require('./init_validators_15178713.json').next_bps));
       await this.bridge.initWithBlock(block_15178713);
       await this.bridge.addLightClientBlock(block_15178760);
       await this.bridge.addLightClientBlock(block_15204402);
       await this.bridge.addLightClientBlock(block_15205393);
   });
});

contract('Add second block in first epoch should be verifiable', function ([_, addr1]) {
    beforeEach(async function () {

    });

    it('should be ok', async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });

        // Get "initial validators" that will produce block 304
        const block244 = require('./244.json');
        const initialValidators = block244.next_bps;

        const block304 = require('./304.json');
        const block308 = require('./308.json');

        await this.bridge.initWithValidators(borshifyInitialValidators(initialValidators));
        await this.bridge.initWithBlock(borshify(block304));
        await this.bridge.blockHashes(304);

        let now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block308));
        await this.bridge.blockHashes(308);

        for (let i = 0; i < block308.approvals_after_next.length; i++) {
            if (block308.approvals_after_next[i]) {
                assert(this.bridge.checkBlockProducerSignatureInHead(i));
            }
        }
    });
});

contract('Test adding blocks in new epoch when bps change', function ([_, addr1]) {
    beforeEach(async function () {

    });

    it('should be ok', async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });

        const block181 = require('./181.json');
        const block244 = require('./244.json');
        const block304 = require('./304.json');
        const block308 = require('./308.json');
        const block368 = require('./368.json');
        const block369 = require('./369.json');

        await this.bridge.initWithValidators(borshifyInitialValidators(block181.next_bps));
        await this.bridge.initWithBlock(borshify(block244));
        await this.bridge.blockHashes(244);

        let now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block304));
        await this.bridge.blockHashes(304);

        now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block308));
        await this.bridge.blockHashes(308);

        now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block368));
        await this.bridge.blockHashes(368);

        now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block369));
        await this.bridge.blockHashes(369);
    });
});

contract('After challenge prev should be revert to prev epoch of latest valid block', function ([_, addr1]) {
    beforeEach(async function () {

    });

    it('should be ok', async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });

        const block181 = require('./181.json');
        const block244 = require('./244.json');
        const block304 = require('./304.json');
        const block308 = require('./308.json');
        const block368 = require('./368.json');
        const block369 = require('./369.json');

        await this.bridge.initWithValidators(borshifyInitialValidators(block181.next_bps));
        await this.bridge.initWithBlock(borshify(block244));
        await this.bridge.blockHashes(244);

        let now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block304));
        await this.bridge.blockHashes(304);

        let oldEpochId = (await this.bridge.head()).epochId;

        now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block308));
        await this.bridge.blockHashes(308);

        now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        block368.approvals_after_next[0] = block368.approvals_after_next[1];
        await this.bridge.addLightClientBlock(borshify(block368));
        await this.bridge.blockHashes(368);
        assert((await this.bridge.head()).epochId != oldEpochId)
        await this.bridge.challenge(addr1, 0);
        assert((await this.bridge.head()).epochId == oldEpochId)
    });
});

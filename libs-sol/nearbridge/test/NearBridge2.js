
const { time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58');

const Ed25519 = artifacts.require('Ed25519');
const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

async function timeIncreaseTo(seconds) {
    const delay = 1000 - new Date().getMilliseconds();
    await new Promise(resolve => setTimeout(resolve, delay));
    await time.increaseTo(seconds);
}

function borshifyInitialValidators(initialValidators) {
    return Buffer.concat([
        web3.utils.toBN(initialValidators.length).toBuffer('le', 4),
        Buffer.concat(
            initialValidators.map(nextBp => Buffer.concat([
                web3.utils.toBN(nextBp.account_id.length).toBuffer('le', 4),
                Buffer.from(nextBp.account_id),
                nextBp.public_key.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                bs58.decode(nextBp.public_key.substr(8)),
                web3.utils.toBN(nextBp.stake).toBuffer('le', 16),
            ])),
        ),
    ]);
}

function borshify(block) {
    return Buffer.concat([
        bs58.decode(block.prev_block_hash),
        bs58.decode(block.next_block_inner_hash),
        Buffer.concat([
            web3.utils.toBN(block.inner_lite.height).toBuffer('le', 8),
            bs58.decode(block.inner_lite.epoch_id),
            bs58.decode(block.inner_lite.next_epoch_id),
            bs58.decode(block.inner_lite.prev_state_root),
            bs58.decode(block.inner_lite.outcome_root),
            web3.utils.toBN(block.inner_lite.timestamp).toBuffer('le', 8),
            bs58.decode(block.inner_lite.next_bp_hash),
            bs58.decode(block.inner_lite.block_merkle_root),
        ]),
        bs58.decode(block.inner_rest_hash),

        Buffer.from([1]),
        web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
        Buffer.concat(
            block.next_bps.map(nextBp => Buffer.concat([
                web3.utils.toBN(nextBp.account_id.length).toBuffer('le', 4),
                Buffer.from(nextBp.account_id),
                nextBp.public_key.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                bs58.decode(nextBp.public_key.substr(8)),
                web3.utils.toBN(nextBp.stake).toBuffer('le', 16),
            ])),
        ),

        web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
        Buffer.concat(
            block.approvals_after_next.map(
                signature => signature === null
                    ? Buffer.from([0])
                    : Buffer.concat([
                        Buffer.from([1]),
                        signature.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                        bs58.decode(signature.substr(8)),
                    ]),
            ),
        ),
    ]);
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
        await this.bridge.initWithBlock(block9605, borshifyInitialValidators(require('./block_9605.json').next_bps));
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

        await this.bridge.initWithBlock(borshify(block304), borshifyInitialValidators(initialValidators));
        await this.bridge.blockHashes(304);

        let now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block308));
        await this.bridge.blockHashes(308);

        for (let i = 0; i < block308.approvals_after_next.length; i++) {
            if (block308.approvals_after_next[i]) {
                assert(this.bridge.checkBlockProducerSignatureInLastBlock(i));
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

        await this.bridge.initWithBlock(borshify(block244), borshifyInitialValidators(block181.next_bps));
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
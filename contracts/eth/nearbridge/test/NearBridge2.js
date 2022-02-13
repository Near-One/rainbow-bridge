const { ethers } = require('hardhat');
const { expect } = require('chai');

const { borshify, borshifyInitialValidators } = require('rainbow-bridge-utils')

async function increaseTime(time) {
    await network.provider.send('evm_increaseTime', [time]);
    await network.provider.send('evm_mine', []);
}

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

it('should be ok', async function () {
    const block9605 = borshify(require('./block_9605.json'));
    const block9610 = borshify(require('./block_9610.json'));

    // We don't know block producers that produce block_9605, assume it's same as block_9605.next_bps
    await NearBridge.initWithValidators(borshifyInitialValidators(require('./block_9605.json').next_bps));
    await NearBridge.initWithBlock(block9605);
    await NearBridge.blockHashes(9605);
    expect(await NearBridge.blockHashes(9605)).to.be.equal(
        '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
    );

    await NearBridge.addLightClientBlock(block9610);
    await increaseTime(10);
    expect(await NearBridge.blockHashes(9610)).to.be.equal(
        '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
    );

});

it('2020-09-09 Example', async function () {
   // Skip until tests are upgraded having blocks after nearcore 1.23.0
   this.skip();

   const block_15178713 = borshify(require('./block_15178713.json'));
   const block_15178760 = borshify(require('./block_15178760.json'));
   const block_15204402 = borshify(require('./block_15204402.json'));
   const block_15248583 = borshify(require('./block_15248583.json'));

   await NearBridge.initWithValidators(borshifyInitialValidators(require('./init_validators_15178713.json')));
   await NearBridge.initWithBlock(block_15178713);
   await increaseTime(3600);
   await NearBridge.addLightClientBlock(block_15178760);
   await increaseTime(3600);
   await NearBridge.addLightClientBlock(block_15204402);
   await increaseTime(3600);
   await NearBridge.addLightClientBlock(block_15248583);
});

it('Add second block in first epoch should be verifiable', async function () {
    // Get "initial validators" that will produce block 304
    const block244 = require('./244.json');
    const initialValidators = block244.next_bps;

    const block304 = require('./304.json');
    const block308 = require('./308.json');

    await NearBridge.initWithValidators(borshifyInitialValidators(initialValidators));
    await NearBridge.initWithBlock(borshify(block304));
    await NearBridge.blockHashes(304);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block308));
    await NearBridge.blockHashes(308);

    for (let i = 0; i < block308.approvals_after_next.length; i++) {
        if (block308.approvals_after_next[i]) {
            expect(await NearBridge.checkBlockProducerSignatureInHead(i)).to.be.true;
        }
    }
});

it('Test adding blocks in new epoch when bps change', async function () {
    // Skip until tests are upgraded having blocks after nearcore 1.23.0
    this.skip();

    const block181 = require('./181.json');
    const block244 = require('./244.json');
    const block304 = require('./304.json');
    const block308 = require('./308.json');
    const block368 = require('./368.json');
    const block369 = require('./369.json');

    await NearBridge.initWithValidators(borshifyInitialValidators(block181.next_bps));
    await NearBridge.initWithBlock(borshify(block244));
    await NearBridge.blockHashes(244);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block304));
    await NearBridge.blockHashes(304);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block308));
    await NearBridge.blockHashes(308);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block368));
    await NearBridge.blockHashes(368);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block369));
    await NearBridge.blockHashes(369);
});

it('After challenge prev should be revert to prev epoch of latest valid block', async function () {
    // Skip until tests are upgraded having blocks after nearcore 1.23.0
    this.skip();

    const block181 = require('./181.json');
    const block244 = require('./244.json');
    const block304 = require('./304.json');
    const block308 = require('./308.json');
    const block368 = require('./368.json');

    await NearBridge.initWithValidators(borshifyInitialValidators(block181.next_bps));
    await NearBridge.initWithBlock(borshify(block244));
    await NearBridge.blockHashes(244);

    await increaseTime(3600);
    await NearBridge.addLightClientBlock(borshify(block304));
    await NearBridge.blockHashes(304);

    await increaseTime(3600);
    await NearBridge.addLightClientBlock(borshify(block308));
    await NearBridge.blockHashes(308);

    await increaseTime(3600);

    block368.approvals_after_next[0] = block368.approvals_after_next[1];
    await NearBridge.addLightClientBlock(borshify(block368));
    await NearBridge.blockHashes(368);
    expect((await NearBridge.lastValidAt())).to.not.be.equal(0);

    await NearBridge.challenge(ethers.constants.AddressZero, 0)
    expect((await NearBridge.lastValidAt())).to.be.equal(0);
});

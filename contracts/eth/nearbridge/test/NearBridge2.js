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

it('2023-05-09 Example', async function () {
   const block_91425093 = borshify(require('./block_91425093.json'));
   const block_91468293 = borshify(require('./block_91468293.json'));
   const block_91511493 = borshify(require('./block_91511493.json'));
   const block_91522568 = borshify(require('./block_91522568.json'));

   await NearBridge.initWithValidators(borshifyInitialValidators(require('./init_validators_91425093.json')));
   await NearBridge.initWithBlock(block_91425093);
   await increaseTime(3600);
   await NearBridge.addLightClientBlock(block_91468293);
   await increaseTime(3600);
   await NearBridge.addLightClientBlock(block_91511493);
   await increaseTime(3600);
   await NearBridge.addLightClientBlock(block_91522568);
});

it('Add second block in first epoch should be verifiable', async function () {
    // Get "initial validators" that will produce block 304
    const block244 = require('./block_244.json');
    const initialValidators = block244.next_bps;

    const block304 = require('./block_304.json');
    const block308 = require('./block_308.json');

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
    const block126313429 = require('./block_126313429.json');
    const block126315547 = require('./block_126315547.json');
    const block126315744 = require('./block_126315744.json');
    const block126315811 = require('./block_126315811.json');
    const block126315892 = require('./block_126315892.json');
    const block126315927 = require('./block_126315927.json');

    await NearBridge.initWithValidators(borshifyInitialValidators(block126313429.next_bps));
    await NearBridge.initWithBlock(borshify(block126315547));
    await NearBridge.blockHashes(126315547);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block126315744));
    await NearBridge.blockHashes(126315744);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block126315811));
    await NearBridge.blockHashes(126315811);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block126315892));
    await NearBridge.blockHashes(126315892);

    await increaseTime(3600);

    await NearBridge.addLightClientBlock(borshify(block126315927));
    await NearBridge.blockHashes(126315927);
});

it('After challenge prev should be revert to prev epoch of latest valid block', async function () {

    const block126313429 = require('./block_126313429.json');
    const block126315547 = require('./block_126315547.json');
    const block126315744 = require('./block_126315744.json');
    const block126315811 = require('./block_126315811.json');
    const block126315892 = require('./block_126315892.json');

    await NearBridge.initWithValidators(borshifyInitialValidators(block126313429.next_bps));
    await NearBridge.initWithBlock(borshify(block126315547));
    await NearBridge.blockHashes(126315547);

    await increaseTime(3600);
    await NearBridge.addLightClientBlock(borshify(block126315744));
    await NearBridge.blockHashes(126315744);

    await increaseTime(3600);
    await NearBridge.addLightClientBlock(borshify(block126315811));
    await NearBridge.blockHashes(126315811);

    await increaseTime(3600);

    block126315892.approvals_after_next[0] = block126315892.approvals_after_next[1];
    await NearBridge.addLightClientBlock(borshify(block126315892));
    await NearBridge.blockHashes(126315892);
    expect((await NearBridge.lastValidAt())).to.not.be.equal(0);

    await NearBridge.challenge(ethers.constants.AddressZero, 0)
    expect((await NearBridge.lastValidAt())).to.be.equal(0);
});

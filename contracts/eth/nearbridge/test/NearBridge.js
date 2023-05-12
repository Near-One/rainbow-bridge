const { expect } = require('chai');

const fs = require('fs').promises;
const { borshify, borshifyInitialValidators } = require('rainbow-bridge-utils');

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
        ethers.BigNumber.from("360"), // lock duration
        ethers.BigNumber.from("362627730000"), // replace duration
        await (await ethers.getSigners())[0].getAddress(),
        0
    );
    await NearBridge.deposit({ value: ethers.utils.parseEther('1') });
});

it('should be ok', async function () {
    const block91425093 = borshify(require('./block_91425093.json'));
    const block91468293 = borshify(require('./block_91468293.json'));
    const block91511493 = borshify(require('./block_91511493.json'));

    // We should use previous epoch's next_bps to initWithBlock with block_91425093, but they happens to be same
    await NearBridge.initWithValidators(borshifyInitialValidators(require('./block_91425093.json').next_bps));
    await NearBridge.initWithBlock(block91425093);
    expect(await NearBridge.blockHashes(91425093)).to.be.equal(
        '0x25d3099add104e19078cc02f5e13eff79386cdde07a841b59d9a078e1f8e6384',
    );

    await NearBridge.addLightClientBlock(block91468293);
    expect(await NearBridge.checkBlockProducerSignatureInHead(0)).to.be.true;

    await expect(NearBridge.addLightClientBlock(block91511493)).to.be.revertedWith('Epoch id of the block is not valid');
    await increaseTime(3600);
    expect(await NearBridge.blockHashes(91468293)).to.be.equal(
        '0xee41bd80c7f770caefb4788876e1c027871e42a333d6aabf894a6ce38ade8d33',
    );

    await NearBridge.addLightClientBlock(block91511493);
    expect(await NearBridge.checkBlockProducerSignatureInHead(1)).to.be.true;

    await increaseTime(3600);
    expect(await NearBridge.blockHashes(91511493)).to.be.equal(
        '0x0552d0022ce2a9c83d96f11126090c9fe6b866198be715228f95ad700d8fb19c',
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

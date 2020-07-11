
// const { time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58');

const Ed25519 = artifacts.require('Ed25519');
const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

const { borshify } = require('../../../environment/lib/near2eth-relay');

// async function timeIncreaseTo (seconds) {
//     const delay = 1000 - new Date().getMilliseconds();
//     await new Promise(resolve => setTimeout(resolve, delay));
//     await time.increaseTo(seconds);
// }

// contract('NearBridge2', function ([_, addr1]) {
//     beforeEach(async function () {
//         this.decoder = await NearDecoder.new();
//         this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(10));
//         await this.bridge.deposit({ value: web3.utils.toWei('1') });
//     });
//
//     it('should be ok', async function () {
//         const block9605 = borshify(require('./block_9605.json'));
//         const block9610 = borshify(require('./block_9610.json'));
//
//         await this.bridge.initWithBlock(block9605);
//         await this.bridge.blockHashes(9605);
//         expect(await this.bridge.blockHashes(9605)).to.be.equal(
//             '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
//         );
//
//         await this.bridge.addLightClientBlock(block9610);
//         expect(await this.bridge.blockHashes(9610)).to.be.equal(
//             '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
//         );
//     });
// });

contract('NearBridge3', function ([_, addr1]) {
    beforeEach(async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(10));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });
    });

    it('should be ok', async function () {
        const block9274566 = borshify(require('./block_9274566.json'));

        await this.bridge.initWithBlock(block9274566);
        await this.bridge.blockHashes(9274566);
        expect(await this.bridge.blockHashes(9274566)).to.be.equal(
            '0x' + bs58.decode('HjUuxHjvSLzsjPDoLa1WzPusgtZyeoRA8i4SyHJuWGWL').toString('hex'),
        );
    });
});

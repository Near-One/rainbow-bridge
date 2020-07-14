
const { time } = require('@openzeppelin/test-helpers');
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

    });

    it('should be ok', async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
        await this.bridge.deposit({value: web3.utils.toWei('1')});

        const block9580503 = require('./block_9580503.json');
        const block9580534 = require('./block_9580534.json');
        const block9580624 = require('./block_9580624.json');

        await this.bridge.initWithBlock(borshify(block9580503));
        await this.bridge.blockHashes(9580503);

        await this.bridge.addLightClientBlock(borshify(block9580534));
        await this.bridge.blockHashes(9580534);

        console.log("Verifying block 9580534");
        for (let i = 0; i < block9580534.approvals_after_next.length; i++) {
            if (block9580534.approvals_after_next[i]) {
                if (await this.bridge.checkBlockProducerSignatureInLastBlock(i)) {
                    console.log(`Signature ${i} is OK`);
                } else {
                    console.log(`Signature ${i} is NOT OK`);
                }
            }
        }

        const now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block9580624));
        await this.bridge.blockHashes(9580624);

        for (let i = 0; i < block9580624.approvals_after_next.length; i++) {
            if (block9580624.approvals_after_next[i]) {
                if (await this.bridge.checkBlockProducerSignatureInLastBlock(i)) {
                    console.log(`Signature ${i} is OK`);
                } else {
                    console.log(`Signature ${i} is NOT OK`);
                }
            }
        }
    });
});


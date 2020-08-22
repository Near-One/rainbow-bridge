
const { expectRevert, time } = require('@openzeppelin/test-helpers');
const fs = require('fs').promises;
const { borshify, borshifyInitialValidators } = require('../../../environment/lib/borsh')

const Ed25519 = artifacts.require('Ed25519');
const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

async function timeIncreaseTo(seconds) {
    const delay = 1000 - new Date().getMilliseconds();
    await new Promise(resolve => setTimeout(resolve, delay));
    await time.increaseTo(seconds);
}

contract('NearBridge', function ([_, addr1]) {
    beforeEach(async function () {

    });

    it('should be ok', async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });

        const block120998 = borshify(require('./block_120998.json'));
        const block121498 = borshify(require('./block_121498.json'));
        const block121998 = borshify(require('./block_121998.json'));

        // http post http://127.0.0.1:3030/ jsonrpc=2.0 method=next_light_client_block params:='["E8K1A51oMR5PkotxNGNkDnVQ9knD4WLn8oDcettqiZbn"]' id="dontcare"
        // We should use previous epoch's next_bps to initWithBlock with block_120998, but they happens to be same
        await this.bridge.initWithValidators(borshifyInitialValidators(require('./block_120998.json').next_bps));
        await this.bridge.initWithBlock(block120998);
        await this.bridge.blockHashes(120998);
        expect(await this.bridge.blockHashes(120998)).to.be.equal(
            '0x1a7a07b5eee1f4d8d7e47864d533143972f858464bacdc698774d167fb1b40e6',
        );

        // http post http://127.0.0.1:3030/ jsonrpc=2.0 method=next_light_client_block params:='["2nMXQQPwni4nAatuH9i1kSiC2i8ivUmCx1QhTnu2TNEZ"]' id="dontcare"
        await this.bridge.addLightClientBlock(block121498);
        expect(await this.bridge.blockHashes(121498)).to.be.equal(
            '0x508307e7af9bdbb297afa7af0541130eb32f0f028151319f5a4f7ae68b0ecc56',
        );

        expect(await this.bridge.checkBlockProducerSignatureInHead(0)).to.be.true;

        await expectRevert(
            this.bridge.addLightClientBlock(block121998),
            'Wait until last block become valid',
        );

        const now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600))); // must use BN.add otherwise it's string concat

        // http post http://127.0.0.1:3030/ jsonrpc=2.0 method=next_light_client_block params:='["6RHW1exQNSSdCrjpKXBb8g1uQdmrmSvuiakZeKN58an9"]' id="dontcare"
        await this.bridge.addLightClientBlock(block121998);
        expect(await this.bridge.blockHashes(121998)).to.be.equal(
            '0x2358c4881bbd111d2e4352b6a7e6c7595fb39d3c9897d3c624006be1ef809abf',
        );

        expect(await this.bridge.checkBlockProducerSignatureInHead(0)).to.be.true;
    });

    if (process.env.NEAR_HEADERS_DIR) {
        it('ok with many block headers', async function () {
            this.decoder = await NearDecoder.new();
            this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(10));
            await this.bridge.deposit({ value: web3.utils.toWei('1') });
            this.timeout(0);
            const blockFiles = await fs.readdir(process.env.NEAR_HEADERS_DIR);
            blockFiles.sort((a, b) => Number(a.split('.')[0]) < Number(b.split('.')[0]));
            const firstBlock = require(process.env.NEAR_HEADERS_DIR + '/' + blockFiles[0]);
            const firstBlockBorsh = borshify(firstBlock);
            // current bps happens to equal to next_bps 
            await this.bridge.initWithValidators(borshifyInitialValidators(firstBlock.next_bps));
            await this.bridge.initWithBlock(firstBlockBorsh);
            await this.bridge.blockHashes(firstBlock.inner_lite.height);
            expect(await this.bridge.blockHashes(firstBlock.inner_lite.height)).to.be.a('string');

            for (let i = 1; i < blockFiles.length; i++) {
                const block = require(process.env.NEAR_HEADERS_DIR + '/' + blockFiles[i]);
                const blockBorsh = borshify(block);
                console.log('adding block ' + block.inner_lite.height);
                await this.bridge.addLightClientBlock(blockBorsh);
                await this.bridge.blockHashes(block.inner_lite.height);
                expect(await this.bridge.blockHashes(block.inner_lite.height)).to.be.a('string');
                const now = await time.latest();
                await timeIncreaseTo(now.add(time.duration.seconds(10)));

                if (i >= 600) {
                    console.log('checking block ' + block.inner_lite.height);
                    for (let j = 0; j < block.approvals_after_next.length; j++) {
                        console.log('checking approval ' + j);
                        if (block.approvals_after_next[j]) {
                            console.log('approval ' + j + ' is not null');
                            expect(await this.bridge.checkBlockProducerSignatureInHead(j)).to.be.true;
                        }
                    }
                }
            }
        });
    }
});

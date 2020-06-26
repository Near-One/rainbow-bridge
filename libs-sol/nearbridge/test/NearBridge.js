
const { expectRevert, time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58');
const fs = require('fs').promises;

const Ed25519 = artifacts.require('Ed25519');
const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

async function timeIncreaseTo (seconds) {
    const delay = 1000 - new Date().getMilliseconds();
    await new Promise(resolve => setTimeout(resolve, delay));
    await time.increaseTo(seconds);
}

function borshify (block) {
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
                signature => signature === null ? 
                Buffer.from([0]) :
                Buffer.concat([
                    Buffer.from([signature ? 1 : 0]),
                    signature.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                    signature ? bs58.decode(signature.substr(8)) : Buffer.from([]),
                ]),
            ),
        ),
    ]);
}

contract('NearBridge', function ([_, addr1]) {
    beforeEach(async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address);
        await this.bridge.deposit({ value: web3.utils.toWei('1') });
    });

    it('should be ok', async function () {
        const block120998 = borshify(require('./block_120998.json'));
        const block121498 = borshify(require('./block_121498.json'));
        const block121998 = borshify(require('./block_121998.json'));

        // http post http://127.0.0.1:3030/ jsonrpc=2.0 method=next_light_client_block params:='["E8K1A51oMR5PkotxNGNkDnVQ9knD4WLn8oDcettqiZbn"]' id="dontcare"
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

        expect(await this.bridge.checkBlockProducerSignatureInLastBlock(0, block121498)).to.be.true;

        await expectRevert(
            this.bridge.addLightClientBlock(block121998),
            'Wait until last block become valid',
        );

        const now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(10))); // must use BN.add otherwise it's string concat

        // http post http://127.0.0.1:3030/ jsonrpc=2.0 method=next_light_client_block params:='["6RHW1exQNSSdCrjpKXBb8g1uQdmrmSvuiakZeKN58an9"]' id="dontcare"
        await this.bridge.addLightClientBlock(block121998);
        expect(await this.bridge.blockHashes(121998)).to.be.equal(
            '0x2358c4881bbd111d2e4352b6a7e6c7595fb39d3c9897d3c624006be1ef809abf',
        );

        expect(await this.bridge.checkBlockProducerSignatureInLastBlock(0, block121998)).to.be.true;
    });

    if(process.env['NEAR_HEADERS_DIR']) {
        it('ok with many block headers', async function() {
            let blockFiles = await fs.readdir(process.env['NEAR_HEADERS_DIR']);
            blockFiles.sort((a, b) => Number(a.split('.')[0]) < Number(b.split('.')[0]));
            const firstBlock = require(process.env['NEAR_HEADERS_DIR'] +'/' + blockFiles[0]);
            const firstBlockBorsh = borshify(firstBlock);
            await this.bridge.initWithBlock(firstBlockBorsh);
            await this.bridge.blockHashes(firstBlock.inner_lite.height);
            expect(await this.bridge.blockHashes(firstBlock.inner_lite.height)).to.be.a('string');
            // for(let i = 0; i < firstBlock.approvals_after_next.length; i++) {
            //     console.log("---==="+i)
            //     if(firstBlock.approvals_after_next[i]) {
            //         console.log("---==="+i)
            //         expect(await this.bridge.checkBlockProducerSignatureInLastBlock(i, firstBlockBorsh)).to.be.true;
            //     }
            // }

            for (let i = 1; i < blockFiles.length; i++) {
                let block = require(process.env['NEAR_HEADERS_DIR'] +'/' + blockFiles[i]);
                const blockBorsh = borshify(block);
                await this.bridge.addLightClientBlock(blockBorsh);
                await this.bridge.blockHashes(block.inner_lite.height);
                expect(await this.bridge.blockHashes(block.inner_lite.height)).to.be.a('string');
                console.log(i);
                const now = await time.latest();
                await timeIncreaseTo(now.add(time.duration.seconds(10)));
                for(let j = 0; j < block.approvals_after_next.length; j++) {
                    console.log("checking approval "+j)
                    if(block.approvals_after_next[j]) {
                        console.log("approval j is not null"+j)
                        expect(await this.bridge.checkBlockProducerSignatureInLastBlock(j, blockBorsh)).to.be.true;
                    }
                }
                
            }
        })
    }
});

const { borshifyOutcomeProof } = require(`rainbow-bridge-lib/rainbow/borsh`);
const bs58 = require('bs58');
const fs = require('fs').promises;

const NearProver = artifacts.require('NearProver');
const NearBridgeMock = artifacts.require('NearBridgeMock');

contract('NearProver', function ([_, addr1]) {
    beforeEach(async function () {
        this.bridge = await NearBridgeMock.new();
        this.prover = await NearProver.new(this.bridge.address);
    });

    it('should be ok', async function () {
        await this.bridge.setBlockMerkleRoot(498, '0x22f00dd154366d758cd3e4fe81c1caed8e0db6227fe4b2b52a8e5a468aa0a723');
        const proof2 = borshifyOutcomeProof(require('./proof2.json'));
        expect(await this.prover.proveOutcome(proof2, 498)).to.be.true;

        await this.bridge.setBlockMerkleRoot(1705, '0x0d0776820a9a81481a559c36fd5d69c33718fb7d7fd3be7564a446e043e2cb35');
        const proof3 = borshifyOutcomeProof(require('./proof3.json'));
        expect(await this.prover.proveOutcome(proof3, 1705)).to.be.true;

        /*await this.bridge.setBlockMerkleRoot(5563, '0x82415d76338be5e5a45524042595f1d9e95f1836c59921bc3fab3201a1519581');
        const proof4 = borshifyOutcomeProof(require('./proof4.json'));
        expect(await this.prover.proveOutcome(proof4, 5563)).to.be.true;*/
    });

    it('should be ok 2', async function () {
        const proof5 = require('./proof5.json');
        await this.bridge.setBlockMerkleRoot(384, '0xa9cd8eb4dd92ba5f2fef47d68e1d73ac8c57047959f6f8a2dcc664419e74e4b8');
        expect(await this.prover.proveOutcome(borshifyOutcomeProof(proof5), 384)).to.be.true;
    });

    it('should be ok 3', async function () {
        const proof6 = require('./proof6.json');
        await this.bridge.setBlockMerkleRoot(377, '0xcc3954a51b7c1a86861df8809f79c2bf839741e3e380e28360b8b3970a5d90bd');
        expect(await this.prover.proveOutcome(borshifyOutcomeProof(proof6), 377)).to.be.true;
    });

    if (process.env['NEAR_PROOFS_DIR']) {
        it('should able to verify proofs from dump', async function () {
            this.timeout(0);
            let proofFiles = await fs.readdir(process.env['NEAR_PROOFS_DIR']);

            for (let i = 1; i < proofFiles.length; i++) {
                let proof = require(process.env['NEAR_PROOFS_DIR'] + '/' + proofFiles[i]);
                let height = proof.block_header_lite.inner_lite.height;
                await this.bridge.setBlockMerkleRoot(height, '0x' + bs58.decode(proof.block_header_lite.inner_lite.block_merkle_root).toString('hex'));
                proof = borshifyOutcomeProof(proof);
                expect(await this.prover.proveOutcome(proof, height)).to.be.true;
                console.log('proved proof ' + proofFiles[i]);
            }
        })
    }

});

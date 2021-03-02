const { borshifyOutcomeProof } = require(`rainbow-bridge-lib/rainbow/borsh`);
const bs58 = require('bs58');
const fs = require('fs').promises;
const { computeMerkleRoot } = require('../utils/utils');

const NearProver = artifacts.require('NearProver');
const NearBridgeMock = artifacts.require('NearBridgeMock');

async function testProof(env, merkleRoot, height, proofPath) {
    let proof = require(proofPath)
    console.log(computeMerkleRoot(proof).toString('hex'))
    expect(merkleRoot === '0x' + computeMerkleRoot(proof).toString('hex')).to.be.true;
    proof = borshifyOutcomeProof(proof);
    await env.bridge.setBlockMerkleRoot(height, merkleRoot);
    expect(await env.prover.proveOutcome(proof, height)).to.be.true;
}

contract('NearProver', function ([_, addr1]) {
    beforeEach(async function () {
        this.bridge = await NearBridgeMock.new();
        this.prover = await NearProver.new(this.bridge.address, '0x0000000000000000000000000000000000000000');
    });

    it('should be ok', async function () {
        await testProof(this, '0x22f00dd154366d758cd3e4fe81c1caed8e0db6227fe4b2b52a8e5a468aa0a723', 498, './proof2.json')
        await testProof(this, '0x0d0776820a9a81481a559c36fd5d69c33718fb7d7fd3be7564a446e043e2cb35', 1705, './proof3.json')
        await testProof(this, '0x1f7129496c461c058fb3daf258d89bf7dacb4efad5742351f66098a00bb6fa53', 5563, './proof4.json')
        await testProof(this, '0xa9cd8eb4dd92ba5f2fef47d68e1d73ac8c57047959f6f8a2dcc664419e74e4b8', 384, './proof5.json')
        await testProof(this, '0xcc3954a51b7c1a86861df8809f79c2bf839741e3e380e28360b8b3970a5d90bd', 377, './proof6.json')
    });

    if (process.env['NEAR_PROOFS_DIR']) {
        it('should able to verify proofs from dump', async function () {
            this.timeout(0);
            let proofFiles = await fs.readdir(process.env['NEAR_PROOFS_DIR']);

            for (let i = 0; i < proofFiles.length; i++) {
                let proof = require(process.env['NEAR_PROOFS_DIR'] + '/' + proofFiles[i]);
                let height = proof.block_header_lite.inner_lite.height;
                await this.bridge.setBlockMerkleRoot(height, '0x' + computeMerkleRoot(proof).toString('hex'));
                proof = borshifyOutcomeProof(proof);
                expect(await this.prover.proveOutcome(proof, height)).to.be.true;
                console.log('proved proof ' + proofFiles[i]);
            }
        })
    }
});

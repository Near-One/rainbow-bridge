const { borshifyOutcomeProof } = require(`../../../environment/lib/borsh`);

const NearProver = artifacts.require('NearProver');
const NearBridgeMock = artifacts.require('NearBridgeMock');

contract('NearProver', function ([_, addr1]) {
    beforeEach(async function () {
        this.bridge = await NearBridgeMock.new();
        this.prover = await NearProver.new(this.bridge.address);
    });

    it('should be ok', async function () {
        await this.bridge.setBlockMerkleRoot(526, '0x727e29b62b5c27d50811916798e32e0464c21eb4bbb81a59eb02c37d65ea668f');
        const proof1 = borshifyOutcomeProof(require('./proof1.json'));
        expect(await this.prover.proveOutcome(proof1, 526)).to.be.true;

        await this.bridge.setBlockMerkleRoot(498, '0x22f00dd154366d758cd3e4fe81c1caed8e0db6227fe4b2b52a8e5a468aa0a723');
        const proof2 = borshifyOutcomeProof(require('./proof2.json'));
        expect(await this.prover.proveOutcome(proof2, 498)).to.be.true;

        await this.bridge.setBlockMerkleRoot(1705, '0x0d0776820a9a81481a559c36fd5d69c33718fb7d7fd3be7564a446e043e2cb35');
        const proof3 = borshifyOutcomeProof(require('./proof3.json'));
        expect(await this.prover.proveOutcome(proof3, 1705)).to.be.true;

        // TODO: Investigate weird block path returned by the node.
        // await this.bridge.setBlockMerkleRoot(5563, '0x82415d76338be5e5a45524042595f1d9e95f1836c59921bc3fab3201a1519581');
        // const proof4 = borshifyOutcomeProof(require('./proof4.json'));
        // expect(await this.prover.proveOutcome(proof4, 5563)).to.be.true;
    });
});

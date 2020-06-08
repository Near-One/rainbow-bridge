
const { expectRevert, time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58')

const NearProver = artifacts.require('NearProver');
const NearBridgeMock = artifacts.require('NearBridgeMock');

function borshifyOutcomeProof(proof) {
    return Buffer.concat([
        // outcome_proof
        web3.utils.toBN(proof.outcome_proof.proof.length).toBuffer('le', 4),
        Buffer.concat(
            // outcome_proof.proof
            proof.outcome_proof.proof.map(
                p => Buffer.concat([
                    bs58.decode(p.hash),
                    Buffer.from([p.direction === 'Right' ? 1 : 0]),
                ])
            )
        ),

        // outcome_proof.block_hash
        bs58.decode(proof.outcome_proof.block_hash),

        // outcome_proof.id
        bs58.decode(proof.outcome_proof.id),

        // outcome_proof.outcome
        Buffer.concat([
            // outcome_proof.outcome.logs
            web3.utils.toBN(proof.outcome_proof.outcome.logs.length).toBuffer('le', 4),
            // TODO: find "logs" example to serialize

            // outcome_proof.outcome.receipt_ids
            web3.utils.toBN(proof.outcome_proof.outcome.receipt_ids.length).toBuffer('le', 4),
            Buffer.concat(
                proof.outcome_proof.outcome.receipt_ids.map(
                    r => bs58.decode(r)
                )
            ),

            // outcome_proof.outcome.gas_burnt
            web3.utils.toBN(proof.outcome_proof.outcome.gas_burnt).toBuffer('le', 8),

            // outcome_proof.outcome.status.SuccessReceiptId
            Buffer.from([3]), // TODO: support other status types
            bs58.decode(proof.outcome_proof.outcome.status.SuccessReceiptId),

            // outcome_root_proof
            web3.utils.toBN(0).toBuffer('le', 4),

            // block_header_lite
            bs58.decode(proof.block_header_lite.prev_block_hash),
            bs58.decode(proof.block_header_lite.inner_rest_hash),
            web3.utils.toBN(proof.block_header_lite.inner_lite.height).toBuffer('le', 8),
            bs58.decode(proof.block_header_lite.inner_lite.epoch_id),
            bs58.decode(proof.block_header_lite.inner_lite.next_epoch_id),
            bs58.decode(proof.block_header_lite.inner_lite.prev_state_root),
            bs58.decode(proof.block_header_lite.inner_lite.outcome_root),
            web3.utils.toBN(proof.block_header_lite.inner_lite.timestamp).toBuffer('le', 8),
            bs58.decode(proof.block_header_lite.inner_lite.next_bp_hash),
            bs58.decode(proof.block_header_lite.inner_lite.block_merkle_root),

            // block_proof
            web3.utils.toBN(proof.block_proof.length).toBuffer('le', 4),
            Buffer.concat(
                proof.block_proof.map(
                    bp => Buffer.concat([
                        bs58.decode(bp.hash),
                        Buffer.from([bp.direction === 'Right' ? 1 : 0]),
                    ])
                )
            ),
        ])
    ]);
}

contract('NearProver', function ([_, addr1]) {
    beforeEach(async function () {
        this.bridge = await NearBridgeMock.new();
        await this.bridge.setBlockHashes(9, "0x2b37a318a99b18b16ae47869dbf0ccf78bc05f2096a9f18e8ca990edbe68c7db");
        this.prover = await NearProver.new(this.bridge.address);
    });

    it('should be ok', async function () {
        const proof_9 = borshifyOutcomeProof(require('./proof_9.json'));
        expect(await this.prover.proveOutcome(proof_9)).to.be.true;
    });
});

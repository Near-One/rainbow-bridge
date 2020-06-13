
const { expectRevert, time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58')

const NearProver = artifacts.require('NearProver');
const NearBridgeMock = artifacts.require('NearBridgeMock');

function borshifyOutcomeProof(proof) {
    const statusToBuffer = status => {
        console.log(status.SuccessValue);
        if ('SuccessValue' in status) {
            return Buffer.concat([
                Buffer.from([2]),
                Buffer.from([]),
            ]);
        } else if ('SuccessReceiptId' in status) {
            return Buffer.concat([
                Buffer.from([3]),
                bs58.decode(status.SuccessReceiptId),
            ]);
        } else {
            throw new Error("status not supported");
        }
    };
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

            statusToBuffer(proof.outcome_proof.outcome.status),
            // outcome_proof.outcome.status.SuccessReceiptId
            // Buffer.from([3]), // TODO: support other status types
            // bs58.decode(proof.outcome_proof.outcome.status.SuccessReceiptId),

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
        this.prover = await NearProver.new(this.bridge.address);
    });

    it('should be ok', async function () {
        const proof1 = borshifyOutcomeProof(require('./proof_9.json'));
        const lightClientBlockMerkleRoot = '0xf53f99cb3f82145317d18eb2949370a3a0be00ef068b5199a3315b5fe7a14fc8';
        expect(await this.prover.proveOutcome(proof1, lightClientBlockMerkleRoot)).to.be.true;
    });
});

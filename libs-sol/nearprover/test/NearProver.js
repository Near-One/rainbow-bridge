
const { expectRevert, time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58')

const NearProver = artifacts.require('NearProver');
const NearBridgeMock = artifacts.require('NearBridgeMock');

function borshifyOutcomeProof(proof) {
    const statusToBuffer = status => {
        console.log(status.SuccessValue);
        if ('SuccessValue' in status) {
            let data = Buffer.from(status.SuccessValue, 'base64');
            console.log(`LEN ${data.length}`);
            console.log(`CONTENT ${JSON.stringify(data)}`);
            return Buffer.concat([
                Buffer.from([2]),
                web3.utils.toBN(data.length).toBuffer('le', 4),
                data
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
    let outcome_proof = Buffer.concat([
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
            web3.utils.toBN(proof.outcome_proof.outcome.tokens_burnt).toBuffer('le', 16),
            web3.utils.toBN(proof.outcome_proof.outcome.executor_id.length).toBuffer('le', 4),
            Buffer.from(proof.outcome_proof.outcome.executor_id, 'utf8'),

            statusToBuffer(proof.outcome_proof.outcome.status),
    ])
        ]);
    const outcome_proof_hex = web3.utils.bytesToHex(outcome_proof);
    console.log(`HEX outcome_proof: ${outcome_proof_hex}`);

    const status_hex = web3.utils.bytesToHex(statusToBuffer(proof.outcome_proof.outcome.status));
    console.log(`HEX status: ${status_hex}`);

    let outcome_root_proof = web3.utils.toBN(0).toBuffer('le', 4);

    const outcome_root_proof_hex = web3.utils.bytesToHex(outcome_root_proof);
    console.log(`HEX outcome_root_proof: ${outcome_root_proof_hex}`);

    let outcome_header_light = Buffer.concat([
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
    ]);

    const outcome_header_light_hex = web3.utils.bytesToHex(outcome_header_light);
    console.log(`HEX outcome_header_light: ${outcome_header_light_hex}`);

    let block_proof = Buffer.concat([
        web3.utils.toBN(proof.block_proof.length).toBuffer('le', 4),
        Buffer.concat(
            proof.block_proof.map(
                bp => Buffer.concat([
                    bs58.decode(bp.hash),
                    Buffer.from([bp.direction === 'Right' ? 1 : 0]),
                ])
            )
        ),
    ]);

    const block_proof_hex = web3.utils.bytesToHex(block_proof);
    console.log(`HEX block_proof: ${block_proof_hex}`);



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
            web3.utils.toBN(proof.outcome_proof.outcome.tokens_burnt).toBuffer('le', 16),
            web3.utils.toBN(proof.outcome_proof.outcome.executor_id.length).toBuffer('le', 4),
            Buffer.from(proof.outcome_proof.outcome.executor_id, 'utf8'),

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
        // await this.bridge.setBlockMerkleRoot(2558, '0x703bde7dded360be8f24a1c53dc119bd714f0e7298a1e44edc46026858d65ce0');
        // const proof1 = borshifyOutcomeProof(require('./proof1.json'));
        // expect(await this.prover.proveOutcome(proof1, 2558)).to.be.true;
        //
        // await this.bridge.setBlockMerkleRoot(49, '0x3a8ca5dfa850600c14233d6e1a31319d7f5285f0cd391416aaf9aaa8070b9040');
        // const proof2 = borshifyOutcomeProof(require('./proof2.json'));
        // expect(await this.prover.proveOutcome(proof2, 49)).to.be.true;
        //
        // await this.bridge.setBlockMerkleRoot(1350, '0x49702b4b256be142958fdc48e46284221c0033a0df714f6cf309ab29356cc1b1');
        // const proof3 = borshifyOutcomeProof(require('./proof3.json'));
        // expect(await this.prover.proveOutcome(proof3, 1350)).to.be.true;

        await this.bridge.setBlockMerkleRoot(526, '0x727e29b62b5c27d50811916798e32e0464c21eb4bbb81a59eb02c37d65ea668f');
        await this.bridge.setBlockHash(526, '0x829d39bd7cb2f78f983f0b49d0598e269b91fe37e7d06d92f0343c9005501bee');
        const proof4 = borshifyOutcomeProof(require('./proof4.json'));
        const h = web3.utils.bytesToHex(proof4);
        console.log(`HEX proof: ${h}`);
        expect(await this.prover.proveOutcome(proof4, 526)).to.be.true;
    });
});

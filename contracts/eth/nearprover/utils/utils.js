const crypto = require('crypto');
const BN = require("bn.js");
const bs58 = require('bs58');

function computeMerkleRoot(proof) {
    const inner_lite_hash = blockHeaderInnerLiteHash(proof.block_header_lite.inner_lite)

    const header_hash = combine(combine(inner_lite_hash, bs58.decode(proof.block_header_lite.inner_rest_hash)), bs58.decode(proof.block_header_lite.prev_block_hash))

    return computeRoot(header_hash, proof.block_proof)
}

function computeRoot(node, proof) {
    proof.forEach((step) => {
        if (step.direction == 'Left') {
            node = combine(bs58.decode(step.hash), node)
        } else {
            node = combine(node, bs58.decode(step.hash))
        }
    })
    return node
}

function combine(data0, data1) {
    const buffer = Buffer.concat([data0, data1])
    return crypto.createHash('sha256').update(buffer).digest()
}

function blockHeaderInnerLiteHash(data) {
    const buffer = Buffer.concat([
        (new BN(data.height)).toBuffer("le", 8),
        bs58.decode(data.epoch_id),
        bs58.decode(data.next_epoch_id),
        bs58.decode(data.prev_state_root),
        bs58.decode(data.outcome_root),
        (new BN(data.timestamp_nanosec || data.timestamp)).toBuffer("le", 8),
        bs58.decode(data.next_bp_hash),
        bs58.decode(data.block_merkle_root),
    ]);
    return crypto.createHash('sha256').update(buffer).digest()
}

exports.computeMerkleRoot = computeMerkleRoot
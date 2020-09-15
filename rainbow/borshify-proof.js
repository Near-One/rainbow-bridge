const Web3 = require('web3')
const bs58 = require('bs58')

function borshifyOutcomeProof(proof) {
  const statusToBuffer = (status) => {
    console.log(status)
    if ('SuccessValue' in status) {
      const data = Buffer.from(status.SuccessValue, 'base64')
      return Buffer.concat([
        Buffer.from([2]),
        Web3.utils.toBN(data.length).toBuffer('le', 4),
        data,
      ])
    } else if ('SuccessReceiptId' in status) {
      return Buffer.concat([
        Buffer.from([3]),
        bs58.decode(status.SuccessReceiptId),
      ])
    } else {
      throw new Error('status not supported')
    }
  }
  return Buffer.concat([
    Web3.utils.toBN(proof.outcome_proof.proof.length).toBuffer('le', 4),
    Buffer.concat(
      proof.outcome_proof.proof.map((p) =>
        Buffer.concat([
          bs58.decode(p.hash),
          Buffer.from([p.direction === 'Right' ? 1 : 0]),
        ])
      )
    ),

    bs58.decode(proof.outcome_proof.block_hash),

    bs58.decode(proof.outcome_proof.id),

    Buffer.concat([
      Web3.utils
        .toBN(proof.outcome_proof.outcome.logs.length)
        .toBuffer('le', 4),

      Web3.utils
        .toBN(proof.outcome_proof.outcome.receipt_ids.length)
        .toBuffer('le', 4),
      Buffer.concat(
        proof.outcome_proof.outcome.receipt_ids.map((r) => bs58.decode(r))
      ),

      Web3.utils.toBN(proof.outcome_proof.outcome.gas_burnt).toBuffer('le', 8),
      Web3.utils
        .toBN(proof.outcome_proof.outcome.tokens_burnt)
        .toBuffer('le', 16),
      Web3.utils
        .toBN(proof.outcome_proof.outcome.executor_id.length)
        .toBuffer('le', 4),
      Buffer.from(proof.outcome_proof.outcome.executor_id, 'utf8'),

      statusToBuffer(proof.outcome_proof.outcome.status),

      Web3.utils.toBN(0).toBuffer('le', 4),

      bs58.decode(proof.block_header_lite.prev_block_hash),
      bs58.decode(proof.block_header_lite.inner_rest_hash),
      Web3.utils
        .toBN(proof.block_header_lite.inner_lite.height)
        .toBuffer('le', 8),
      bs58.decode(proof.block_header_lite.inner_lite.epoch_id),
      bs58.decode(proof.block_header_lite.inner_lite.next_epoch_id),
      bs58.decode(proof.block_header_lite.inner_lite.prev_state_root),
      bs58.decode(proof.block_header_lite.inner_lite.outcome_root),
      // for backward compatible in tests with old dumps
      Web3.utils
        .toBN(
          proof.block_header_lite.inner_lite.timestamp_nanosec ||
            proof.block_header_lite.inner_lite.timestamp
        )
        .toBuffer('le', 8),
      bs58.decode(proof.block_header_lite.inner_lite.next_bp_hash),
      bs58.decode(proof.block_header_lite.inner_lite.block_merkle_root),

      Web3.utils.toBN(proof.block_proof.length).toBuffer('le', 4),
      Buffer.concat(
        proof.block_proof.map((bp) =>
          Buffer.concat([
            bs58.decode(bp.hash),
            Buffer.from([bp.direction === 'Right' ? 1 : 0]),
          ])
        )
      ),
    ]),
  ])
}

exports.borshifyOutcomeProof = borshifyOutcomeProof

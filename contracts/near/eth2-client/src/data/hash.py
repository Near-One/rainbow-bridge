from eth2spec.deneb.mainnet import (
    SignedBeaconBlock,
    get_generalized_index,
    compute_merkle_proof,
    EXECUTION_PAYLOAD_GINDEX,
)

import json

with open("body.json", "r") as f:
    beacon_body_dict = json.load(f)


signed_block = SignedBeaconBlock.from_obj(beacon_body_dict)
l1_execution_payload_proof = compute_merkle_proof(
    signed_block.message.body, EXECUTION_PAYLOAD_GINDEX
)
for i in l1_execution_payload_proof:
    print(i.hex())

print("---")

BLOCK_HASH_GINDEX = get_generalized_index(
    signed_block.message.body.execution_payload, "block_hash"
)
print(BLOCK_HASH_GINDEX)
l2_proof = compute_merkle_proof(
    signed_block.message.body.execution_payload, BLOCK_HASH_GINDEX
)
for i in l2_proof:
    print(i.hex())

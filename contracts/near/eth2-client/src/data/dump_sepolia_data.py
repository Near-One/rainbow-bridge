#!/usr/bin/env python3
import json
import os
import time
from pathlib import Path
from typing import Dict, List

import requests
from eth2spec.electra.mainnet import (
    EXECUTION_PAYLOAD_GINDEX,
    SignedBeaconBlock,
    compute_merkle_proof,
    get_generalized_index,
)
from tqdm import tqdm
from web3 import Web3
from web3.types import BlockData, HexBytes

MAX_RETRIES = 3
RETRY_DELAY = 1

NETWORK = os.getenv("NETWORK", "sepolia")
OUT_DIR = Path(f"./{NETWORK}")
OUT_DIR.mkdir(exist_ok=True)

# Sepolia RPC endpoint
EXECUTION_RPC = os.getenv(
    "EXECUTION_RPC", "https://ethereum-sepolia-rpc.publicnode.com"
)

# Beacon-node REST API endpoint
CONSENSUS_API = os.getenv(
    "CONSENSUS_API", "http://unstable.sepolia.beacon-api.nimbus.team"
)
BLOCK_WINDOW = int(os.getenv("BLOCK_WINDOW", "50"))

# Batch size for RPC calls
BATCH_SIZE = int(os.getenv("BATCH_SIZE", "50"))

w3 = Web3(Web3.HTTPProvider(EXECUTION_RPC))


def batch_fetch_execution_blocks(numbers: List[int]) -> Dict[int, BlockData]:
    """Fetch multiple blocks using Web3.py batch requests."""
    batch = w3.batch_requests()

    # Add all block requests to the batch
    for number in numbers:
        batch.add(w3.eth.get_block(number, full_transactions=True))

    # Execute the batch
    results = batch.execute()

    # Map results back to block numbers
    blocks: Dict[int, BlockData] = {}
    for i, block in enumerate(results):
        blocks[numbers[i]] = block  # type: ignore

    return blocks


def fetch_execution_block(number: int) -> BlockData:
    """Fetch full block by number via Web3."""
    block = w3.eth.get_block(number, full_transactions=True)
    return block


def normalize_block(raw: BlockData) -> dict:
    return {
        "parent_hash": raw.get("parentHash"),
        "uncles_hash": raw.get("sha3Uncles"),
        "author": raw.get("miner"),
        "state_root": raw.get("stateRoot"),
        "transactions_root": raw.get("transactionsRoot"),
        "receipts_root": raw.get("receiptsRoot"),
        "log_bloom": raw.get("logsBloom"),
        "difficulty": hex(raw.get("difficulty", 0)),
        "number": hex(raw.get("number", 0)),
        "gas_limit": hex(raw.get("gasLimit", 0)),
        "gas_used": hex(raw.get("gasUsed", 0)),
        "timestamp": hex(raw.get("timestamp", 0)),
        "extra_data": raw.get("extraData"),
        "mix_hash": raw.get("mixHash"),
        "nonce": raw.get("nonce"),
        "base_fee_per_gas": hex(raw.get("baseFeePerGas", 0))
        if raw.get("baseFeePerGas") is not None
        else None,  # Convert to hex
        "withdrawals_root": raw.get("withdrawalsRoot"),
        "blob_gas_used": hex(raw.get("blobGasUsed", 0)),
        "excess_blob_gas": hex(raw.get("excessBlobGas", 0)),
        "parent_beacon_block_root": raw.get("parentBeaconBlockRoot"),
        "requests_hash": raw.get("requestsHash"),
    }


def dump_execution_blocks(start: int, end: int):
    """
    Dump execution blocks from start to end inclusive, with progress, retries, and streaming to file.
    Uses batch requests by default for better performance.
    """

    def hb_default(o):
        if isinstance(o, (bytes, HexBytes)):
            return "0x" + o.hex()
        raise TypeError

    total = end - start + 1
    path = OUT_DIR / f"execution_blocks_{start}_{end}.json"
    print(f"⛏ Dumping execution blocks {start}→{end} (batch size: {BATCH_SIZE})")

    with path.open("w") as f:
        f.write("[\n")
        with tqdm(total=total, desc="Fetching blocks") as pbar:
            # Process blocks in batches
            for batch_start in range(start, end + 1, BATCH_SIZE):
                batch_end = min(batch_start + BATCH_SIZE - 1, end)
                batch_numbers = list(range(batch_start, batch_end + 1))

                # Retry logic for batch
                for attempt in range(1, MAX_RETRIES + 1):
                    try:
                        blocks = batch_fetch_execution_blocks(batch_numbers)
                        break
                    except Exception as e:
                        print(
                            f"⚠️ Error fetching batch {batch_start}-{batch_end} (attempt {attempt}/{MAX_RETRIES}): {e}"
                        )
                        if attempt < MAX_RETRIES:
                            time.sleep(RETRY_DELAY)
                        else:
                            # Fall back to individual fetching for this batch
                            print(
                                f"⚠️ Falling back to individual fetching for batch {batch_start}-{batch_end}"
                            )
                            blocks = {}
                            for number in batch_numbers:
                                for single_attempt in range(1, MAX_RETRIES + 1):
                                    try:
                                        raw = w3.eth.get_block(
                                            number, full_transactions=True
                                        )
                                        blocks[number] = raw
                                        break
                                    except Exception as e:
                                        print(
                                            f"⚠️ Error fetching block {number} (attempt {single_attempt}/{MAX_RETRIES}): {e}"
                                        )
                                        if single_attempt < MAX_RETRIES:
                                            time.sleep(RETRY_DELAY)
                                        else:
                                            raise RuntimeError(
                                                f"Failed to fetch block {number} after {MAX_RETRIES} attempts"
                                            )

                # Write blocks in order
                for number in batch_numbers:
                    if number in blocks:
                        block = normalize_block(blocks[number])
                        f.write(json.dumps(block, default=hb_default))
                        if number < end:
                            f.write(",\n")
                        else:
                            f.write("\n")
                        f.flush()
                    pbar.update(1)

        f.write("]")

    print(f"✔ Wrote {total} blocks → {path}")


def fetch_beacon_header(block_id: str) -> dict:
    url = f"{CONSENSUS_API}/eth/v1/beacon/headers/{block_id}"
    resp = requests.get(url)
    resp.raise_for_status()
    return resp.json().get("data", {}).get("header", {})


def dump_beacon_header_with_slot() -> int:
    header = fetch_beacon_header("finalized")
    slot = int(header["message"]["slot"])
    path = OUT_DIR / f"beacon_header_{slot}.json"
    with path.open("w") as f:
        json.dump(header["message"], f, indent=2)
    print(f"✔ Wrote Beacon header message (slot {slot}) → {path}")
    return slot


def get_recent_light_client_updates(count: int = 4) -> List[int]:
    header = fetch_beacon_header("finalized")
    current_slot = int(header["message"]["slot"])
    current_period = current_slot // 8192  # 8192 slots per period
    periods = [current_period - i for i in range(count)]

    print(f"Current slot: {current_slot}")
    print(f"Current period: {current_period}")
    print(f"Fetching periods: {periods}")

    dump_light_client_updates(periods)
    return periods


def fetch_light_client_update(period: int) -> dict:
    url = f"{CONSENSUS_API}/eth/v1/beacon/light_client/updates"
    resp = requests.get(url, params={"start_period": period, "count": 1})
    resp.raise_for_status()
    with open("raw.json", "w") as f:
        f.write(resp.text)
    return resp.json()[0].get("data", {})


def fetch_beacon_block_body(slot: int):
    url = f"{CONSENSUS_API}/eth/v2/beacon/blocks/{slot}"
    resp = requests.get(url)
    resp.raise_for_status()
    with open("body.json", "w") as f:
        f.write(resp.text)
    return resp.json()["data"]


def get_proof(beacon_body: dict) -> tuple[str, list[str]]:
    signed_block = SignedBeaconBlock.from_obj(beacon_body)
    l1_execution_payload_proof = compute_merkle_proof(
        signed_block.message.body, EXECUTION_PAYLOAD_GINDEX
    )
    l1_proof = ["0x" + i.hex() for i in l1_execution_payload_proof]

    BLOCK_HASH_GINDEX = get_generalized_index(
        signed_block.message.body.execution_payload, "block_hash"
    )
    block_proof = compute_merkle_proof(
        signed_block.message.body.execution_payload, BLOCK_HASH_GINDEX
    )
    block_proof = ["0x" + i.hex() for i in block_proof]
    full_proof = block_proof + l1_proof

    block_hash = str(signed_block.message.body.execution_payload.block_hash)
    return block_hash, full_proof


def convert_to_old_format(update: dict) -> dict:
    """
    Convert a light client update from the new format to match the Rust output format.
    """
    old_format = {}

    # 1. Convert attested_header.beacon to attested_beacon_header (just the beacon part)
    if "attested_header" in update and "beacon" in update["attested_header"]:
        old_format["attested_beacon_header"] = update["attested_header"]["beacon"]

    # 2. Copy sync_aggregate as is
    if "sync_aggregate" in update:
        old_format["sync_aggregate"] = update["sync_aggregate"]

    # 3. Copy signature_slot if present
    if "signature_slot" in update:
        old_format["signature_slot"] = update["signature_slot"]

    # 4. Handle finality_update - this is the key difference
    finality_update = {}

    if "finalized_header" in update:
        # Create the header_update structure
        header_update = {}

        # Set beacon_header from finalized_header.beacon
        if "beacon" in update["finalized_header"]:
            header_update["beacon_header"] = update["finalized_header"]["beacon"]

        finality_update["header_update"] = header_update

    # Copy finality_branch if present (at the root level)
    if "finality_branch" in update:
        finality_update["finality_branch"] = update["finality_branch"]

    old_format["finality_update"] = finality_update

    slot = update["finalized_header"]["beacon"]["slot"]
    body = fetch_beacon_block_body(slot)
    block_hash, full_proof = get_proof(body)
    old_format["finality_update"]["header_update"]["execution_block_hash"] = block_hash
    old_format["finality_update"]["header_update"]["execution_hash_branch"] = full_proof

    # 5. Handle sync_committee_update - make it optional as Some(update) in Rust
    sync_committee_update = {}

    # Add next_sync_committee if present
    if "next_sync_committee" in update:
        sync_committee_update["next_sync_committee"] = update["next_sync_committee"]

    # Add next_sync_committee_branch if present
    if "next_sync_committee_branch" in update:
        sync_committee_update["next_sync_committee_branch"] = update[
            "next_sync_committee_branch"
        ]

    # Only add sync_committee_update if it has content (similar to Some() in Rust)
    if sync_committee_update:
        old_format["sync_committee_update"] = sync_committee_update
    else:
        # Rust might expect None, but in JSON this could be null or omitted
        # You might need to adjust based on what your Rust code expects
        old_format["sync_committee_update"] = None

    return old_format


def dump_light_client_updates(periods: List[int]):
    for p in periods:
        try:
            print(f"🔄 Fetching LightClientUpdate for period {p}")
            upd = fetch_light_client_update(p)
            if not upd:
                print(f"⚠️ No updates for period {p}, skipping.")
                continue

            formatted = convert_to_old_format(upd)
            path = OUT_DIR / f"light_client_update_period_{p}.json"
            with path.open("w") as f:
                json.dump(formatted, f, indent=2)
            print(f"✔ Wrote update → {path}")
        except requests.HTTPError as e:
            print(f"⚠️ Skipping period {p}: {e}")


def load_update(period: int) -> dict:
    path = OUT_DIR / f"light_client_update_period_{period}.json"
    with path.open() as f:
        return json.load(f)


def get_block_number_for_period(period: int) -> int:
    upd = load_update(period)
    block_hash = upd["finality_update"]["header_update"]["execution_block_hash"]
    block = w3.eth.get_block(block_hash, full_transactions=False)
    return block["number"]


def main():
    periods = get_recent_light_client_updates(count=4)
    print(f"🎉 Updates fetched for periods: {periods}")

    rev = list(reversed(periods))
    start = get_block_number_for_period(rev[1])
    end = get_block_number_for_period(rev[2])
    next_end = get_block_number_for_period(rev[3])
    dump_execution_blocks(start, end)
    dump_execution_blocks(end + 1, next_end)


if __name__ == "__main__":
    main()

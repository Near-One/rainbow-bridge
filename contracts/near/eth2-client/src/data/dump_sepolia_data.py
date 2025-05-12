#!/usr/bin/env python3
import os
import json
import requests
from web3 import Web3
from typing import List

# ─────────── CONFIG ───────────

# Fallback list of completely free Sepolia RPC endpoints
EXECUTION_RPCS = [
    os.environ.get("EXECUTION_RPC", ""),
    "https://ethereum-sepolia-rpc.publicnode.com",  # PublicNode
    "https://rpc.sepolia.org",  # RPC.Sepolia.org
    "https://ethereum-sepolia.blockpi.network/v1/rpc/public",
    "https://endpoints.omniatech.io/v1/eth/sepolia/public",
]

# Beacon-node REST API (must support /eth/v1/beacon/headers and /eth/v1/beacon/light_client/updates)
CONSENSUS_API = (
    os.environ.get("CONSENSUS_API") or "http://unstable.sepolia.beacon-api.nimbus.team"
)

NETWORK = "sepolia"
OUT_DIR = f"./{NETWORK}"
os.makedirs(OUT_DIR, exist_ok=True)

# HARDCODED TARGET
TARGET_BLOCK_NUMBER = 8303247
TARGET_SLOT = 7602176


# How many execution blocks to pull (counting backwards from head)
BLOCK_WINDOW = int(os.environ.get("BLOCK_WINDOW", "50"))

# How many past completed sync-committee periods to dump
PERIOD_WINDOW = int(os.environ.get("PERIOD_WINDOW", "20"))

# ─────────── HELPERS ───────────


def pick_rpc(candidates: List[str]) -> str:
    for url in candidates:
        if not url:
            continue
        try:
            w3 = Web3(Web3.HTTPProvider(url))
            if w3.is_connected():
                print(f"✅ Connected to execution RPC: {url}")
                return url
        except Exception:
            pass
    raise SystemExit("❌ No execution RPC available!")


# Pick a working execution RPC
EXECUTION_RPC = pick_rpc(EXECUTION_RPCS)
w3 = Web3(Web3.HTTPProvider(EXECUTION_RPC))


def fetch_execution_block(number: int) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": [hex(number), True],
        "id": 1,
    }
    r = requests.post(EXECUTION_RPC, json=payload)
    r.raise_for_status()
    blk = r.json().get("result")
    if blk is None:
        raise RuntimeError(f"No block data for {number}")
    return blk


def dump_execution_blocks(start: int, window: int):
    end = start + window
    print(f"⛏ Dumping execution blocks {start}→{end}")
    blocks_raw = [fetch_execution_block(n) for n in range(start, end + 1)]
    blocks = [normalize_block(b) for b in blocks_raw]
    path = os.path.join(OUT_DIR, f"execution_blocks_{start}_{end}.json")
    with open(path, "w") as f:
        json.dump(blocks, f, indent=2)
    print(f"✔ Wrote {len(blocks)} blocks → {path}")


def fetch_beacon_header(block_id: str) -> dict:
    """
    Fetch a SignedBeaconHeader at block_id ("finalized", "head" or a slot number).
    """
    url = f"{CONSENSUS_API}/eth/v1/beacon/headers/{block_id}"
    r = requests.get(url)
    r.raise_for_status()
    return r.json().get("data", {}).get("header", {})


def dump_beacon_header_with_slot():
    header = fetch_beacon_header("finalized")
    slot = int(header["message"]["slot"])
    filename = f"beacon_header_{slot}.json"
    path = os.path.join(OUT_DIR, filename)
    with open(path, "w") as f:
        # Dump only the message part
        json.dump(header["message"], f, indent=2)
    print(f"✔ Wrote Beacon header message (slot {slot}) → {path}")
    return slot


def compute_period(slot: int) -> int:
    # 32 slots/epoch × 256 epochs/period = 8192 slots/period
    return slot // 8192


def fetch_light_client_update(period: int) -> dict:
    """
    Call Lodestar's light-client updates endpoint.
    """
    url = f"{CONSENSUS_API}/eth/v1/beacon/light_client/updates"
    r = requests.get(url, params={"start_period": period, "count": 1})
    r.raise_for_status()
    # extract the list of updates directly
    data = r.json()[0].get("data", [])
    return data


def convert_to_old_format(update: dict) -> dict:
    """
    Convert a light client update from the new format to the old format.
    """
    old_format = {}

    # Convert attested_header.beacon to attested_beacon_header
    if "attested_header" in update and "beacon" in update["attested_header"]:
        old_format["attested_beacon_header"] = update["attested_header"]["beacon"]

    att = update["attested_header"]
    if "execution" in att:
        old_format["attested_beacon_header"]["execution_block_hash"] = att["execution"][
            "block_hash"
        ]
    if "execution_branch" in att:
        old_format["attested_beacon_header"]["execution_branch"] = att[
            "execution_branch"
        ]

    # Copy sync_aggregate as is (structure seems the same)
    if "sync_aggregate" in update:
        old_format["sync_aggregate"] = update["sync_aggregate"]

    # Copy signature_slot if present
    if "signature_slot" in update:
        old_format["signature_slot"] = update["signature_slot"]

    # Handle finality_update
    if "finalized_header" in update:
        # Create the header_update structure
        header_update = {
            "beacon_header": update.get("finalized_header", {}).get("beacon", {})
        }

        # Add execution_block_hash from the finalized_header.execution.block_hash
        if "execution" in update.get("finalized_header", {}):
            execution_block_hash = update["finalized_header"]["execution"].get(
                "block_hash"
            )
            if execution_block_hash:
                header_update["execution_block_hash"] = execution_block_hash

        # Add execution_hash_branch from the finalized_header.execution_branch
        if "execution_branch" in update.get("finalized_header", {}):
            header_update["execution_hash_branch"] = update["finalized_header"][
                "execution_branch"
            ]

        # Set up the finality_update structure
        old_format["finality_update"] = {"header_update": header_update}

        # Copy finality_branch if present
        if "finality_branch" in update:
            old_format["finality_update"]["finality_branch"] = update["finality_branch"]

    # Handle sync_committee_update
    old_format["sync_committee_update"] = {}

    # Add next_sync_committee if present
    if "next_sync_committee" in update:
        old_format["sync_committee_update"]["next_sync_committee"] = update[
            "next_sync_committee"
        ]

    # Check for next_sync_committee_branch in various possible locations
    if "next_sync_committee_branch" in update:
        old_format["sync_committee_update"]["next_sync_committee_branch"] = update[
            "next_sync_committee_branch"
        ]

    return old_format


def dump_light_client_updates(periods: List[int]):
    for p in periods:
        try:
            print(f"🔄  Fetching LightClientUpdate for period {p}")
            update = fetch_light_client_update(p)
            if not update:
                print(f"⚠️  No updates available for period {p}, skipping.")
                continue

            # Convert each update to old format
            old_format_update = convert_to_old_format(update)

            path = os.path.join(OUT_DIR, f"light_client_update_period_{p}.json")
            with open(path, "w") as f:
                json.dump(old_format_update, f, indent=2)
            print(f"✔ Wrote {len(old_format_update)} update(s) → {path}")
        except requests.HTTPError as e:
            print(f"⚠️  Skipping period {p}: {e}")


def normalize_block(raw: dict) -> dict:
    return {
        # Core fields (must all be present)
        "parent_hash": raw["parentHash"],
        "uncles_hash": raw.get("sha3Uncles", "0x1dcc4de8dec75..."),
        "author": raw["miner"],
        "state_root": raw["stateRoot"],
        "transactions_root": raw["transactionsRoot"],
        "receipts_root": raw["receiptsRoot"],
        "log_bloom": raw["logsBloom"],
        "difficulty": raw["difficulty"],
        "number": raw["number"],
        "gas_limit": raw["gasLimit"],
        "gas_used": raw["gasUsed"],
        "timestamp": raw["timestamp"],
        "extra_data": raw["extraData"],
        "mix_hash": raw["mixHash"],
        "nonce": raw["nonce"],
        "base_fee_per_gas": raw.get("baseFeePerGas"),
        "withdrawals_root": raw.get("withdrawalsRoot"),
        # drop everything else
    }


# ─────────── MAIN ───────────

if __name__ == "__main__":
    # 1) Fetch and dump the execution blocks
    # dump_execution_blocks(TARGET_BLOCK_NUMBER, BLOCK_WINDOW)

    # 2) Fetch and dump the beacon header for the target slot
    print(f"🔄 Fetching beacon header for slot {TARGET_SLOT}")
    beacon_header = fetch_beacon_header(str(TARGET_SLOT))
    beacon_path = os.path.join(OUT_DIR, f"beacon_header_{TARGET_SLOT}.json")
    with open(beacon_path, "w") as f:
        # Dump only the message part
        json.dump(beacon_header["message"], f, indent=2)
    print(f"✔ Wrote beacon header → {beacon_path}")

    # 3) Fetch the light client update for the period containing this slot
    period = TARGET_SLOT // 8192
    print(f"🔄 Fetching light client update for period {period}")
    try:
        update = fetch_light_client_update(period)
        if update:
            old_format_update = convert_to_old_format(update)
            update_path = os.path.join(
                OUT_DIR, f"light_client_update_period_{period}.json"
            )
            with open(update_path, "w") as f:
                json.dump(old_format_update, f, indent=2)
            print(f"✔ Wrote light client update → {update_path}")
    except Exception as e:
        print(f"⚠️ Could not fetch light client update: {e}")

    dump_light_client_updates([926, 927])
    print("🎉 All done!")

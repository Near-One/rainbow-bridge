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
    os.environ.get("CONSENSUS_API") or "https://lodestar-sepolia.chainsafe.io"
)

NETWORK = "sepolia"
OUT_DIR = f"./{NETWORK}"
os.makedirs(OUT_DIR, exist_ok=True)

# How many execution blocks to pull (counting backwards from head)
BLOCK_WINDOW = int(os.environ.get("BLOCK_WINDOW", "50"))

# How many past completed sync-committee periods to dump
PERIOD_WINDOW = int(os.environ.get("PERIOD_WINDOW", "4"))

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


def dump_recent_execution_blocks(window: int):
    head = w3.eth.block_number
    start = max(0, head - window + 1)
    print(f"⛏ Dumping execution blocks {start}→{head}")
    blocks_raw = [fetch_execution_block(n) for n in range(start, head + 1)]
    blocks = [normalize_block(b) for b in blocks_raw]
    path = os.path.join(OUT_DIR, f"execution_blocks_{start}_{head}.json")
    with open(path, "w") as f:
        json.dump(blocks, f, indent=2)
    print(f"✔ Wrote {len(blocks)} blocks → {path}")


def fetch_beacon_header(block_id: str) -> dict:
    """
    Fetch a SignedBeaconHeader at block_id (“finalized”, “head” or a slot number).
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


def fetch_light_client_update(period: int) -> List[dict]:
    """
    Call Lodestar's light-client updates endpoint.
    """
    url = f"{CONSENSUS_API}/eth/v1/beacon/light_client/updates"
    r = requests.get(url, params={"start_period": period, "count": 1})
    r.raise_for_status()
    # extract the list of updates directly
    data = r.json()[0].get("data", [])
    return data


def dump_light_client_updates(periods: List[int]):
    for p in periods:
        try:
            print(f"🔄  Fetching LightClientUpdate for period {p}")
            updates = fetch_light_client_update(p)
            if not updates:
                print(f"⚠️  No updates available for period {p}, skipping.")
                continue
            path = os.path.join(OUT_DIR, f"light_client_update_period_{p}.json")
            with open(path, "w") as f:
                json.dump(updates, f, indent=2)
            print(f"✔ Wrote {len(updates)} update(s) → {path}")
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
    # 1) Dump execution blocks
    dump_recent_execution_blocks(BLOCK_WINDOW)

    # 2) Dump the finalized beacon header with numeric slot in filename
    finalized_slot = dump_beacon_header_with_slot()

    # 3) Determine which periods to fetch updates for
    current_period = compute_period(finalized_slot)
    periods = [
        current_period - i
        for i in range(1, PERIOD_WINDOW + 1)
        if current_period - i >= 0
    ]
    dump_light_client_updates(periods)

    print("🎉 All done!")

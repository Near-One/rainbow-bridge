#!/usr/bin/env python3
import os
import json
import logging
from pathlib import Path
import requests
from web3 import Web3
from web3.types import BlockData
from typing import List

# ─────────── CONFIG ───────────
NETWORK = os.getenv("NETWORK", "sepolia")
OUT_DIR = Path(f"./{NETWORK}")
OUT_DIR.mkdir(exist_ok=True)

# Sepolia RPC endpoints (fallbacks)
EXECUTION_RPCS = [
    os.getenv("EXECUTION_RPC"),
    "https://ethereum-sepolia-rpc.publicnode.com",
    "https://rpc.sepolia.org",
    "https://ethereum-sepolia.blockpi.network/v1/rpc/public",
    "https://endpoints.omniatech.io/v1/eth/sepolia/public",
]

# Beacon-node REST API endpoint
CONSENSUS_API = os.getenv(
    "CONSENSUS_API", "http://unstable.sepolia.beacon-api.nimbus.team"
)
BLOCK_WINDOW = int(os.getenv("BLOCK_WINDOW", "50"))

# Initialize logging
logging.basicConfig(level=logging.INFO, format="%(message)s")
logger = logging.getLogger(__name__)

# Pick a working execution RPC and initialize Web3


def pick_rpc(candidates: List[str]) -> str:
    for url in filter(None, candidates):
        try:
            w3 = Web3(Web3.HTTPProvider(url))
            if w3.is_connected():
                logger.info(f"✅ Connected to execution RPC: {url}")
                return url
        except Exception:
            continue
    raise SystemExit("❌ No execution RPC available!")


EXECUTION_RPC = pick_rpc(EXECUTION_RPCS)
w3 = Web3(Web3.HTTPProvider(EXECUTION_RPC))

# ─────────── HELPERS ───────────


def fetch_execution_block(number: int) -> BlockData:
    """Fetch full block by number via Web3."""
    block = w3.eth.get_block(number, full_transactions=True)
    return block


def normalize_block(raw: BlockData) -> dict:
    return {
        "parent_hash": raw.get("parentHash") or raw.get("parent_hash"),
        "uncles_hash": raw.get(
            "sha3Uncles", raw.get("unclesHash", "0x1dcc4de8dec75...")
        ),
        "author": raw.get("miner") or raw.get("author"),
        "state_root": raw.get("stateRoot") or raw.get("state_root"),
        "transactions_root": raw.get("transactionsRoot")
        or raw.get("transactions_root"),
        "receipts_root": raw.get("receiptsRoot") or raw.get("receipts_root"),
        "log_bloom": raw.get("logsBloom") or raw.get("logs_bloom"),
        "difficulty": raw.get("difficulty"),
        "number": hex(raw.get("number", 0)),
        "gas_limit": raw.get("gasLimit") or raw.get("gas_limit"),
        "gas_used": raw.get("gasUsed") or raw.get("gas_used"),
        "timestamp": raw.get("timestamp"),
        "extra_data": raw.get("extraData") or raw.get("extra_data"),
        "mix_hash": raw.get("mixHash") or raw.get("mix_hash"),
        "nonce": raw.get("nonce"),
        "base_fee_per_gas": raw.get("baseFeePerGas") or raw.get("base_fee_per_gas"),
        "withdrawals_root": raw.get("withdrawalsRoot") or raw.get("withdrawals_root"),
    }


def dump_execution_blocks(start: int, end: int):
    logger.info(f"⛏ Dumping execution blocks {start}→{end}")
    blocks = [normalize_block(fetch_execution_block(n)) for n in range(start, end + 1)]
    path = OUT_DIR / f"execution_blocks_{start}_{end}.json"
    with path.open("w") as f:
        json.dump(blocks, f, indent=2)
    logger.info(f"✔ Wrote {len(blocks)} blocks → {path}")


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
    logger.info(f"✔ Wrote Beacon header message (slot {slot}) → {path}")
    return slot


def compute_period(slot: int) -> int:
    return slot // 8192


def get_recent_light_client_updates(count: int = 4) -> List[int]:
    header = fetch_beacon_header("finalized")
    current_slot = int(header["message"]["slot"])
    current_period = compute_period(current_slot)
    periods = [current_period - i for i in range(count)]

    logger.info(f"Current slot: {current_slot}")
    logger.info(f"Current period: {current_period}")
    logger.info(f"Fetching periods: {periods}")

    dump_light_client_updates(periods)
    return periods


def fetch_light_client_update(period: int) -> dict:
    url = f"{CONSENSUS_API}/eth/v1/beacon/light_client/updates"
    resp = requests.get(url, params={"start_period": period, "count": 1})
    resp.raise_for_status()
    return resp.json()[0].get("data", {})


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
            logger.info(f"🔄 Fetching LightClientUpdate for period {p}")
            upd = fetch_light_client_update(p)
            if not upd:
                logger.warning(f"⚠️ No updates for period {p}, skipping.")
                continue

            formatted = convert_to_old_format(upd)
            path = OUT_DIR / f"light_client_update_period_{p}.json"
            with path.open("w") as f:
                json.dump(formatted, f, indent=2)
            logger.info(f"✔ Wrote update → {path}")
        except requests.HTTPError as e:
            logger.warning(f"⚠️ Skipping period {p}: {e}")


def load_update(period: int) -> dict:
    path = OUT_DIR / f"light_client_update_period_{period}.json"
    with path.open() as f:
        return json.load(f)


def get_block_number_for_period(period: int) -> int:
    upd = load_update(period)
    block_hash = upd["attested_beacon_header"]["execution_block_hash"]
    block = w3.eth.get_block(block_hash, full_transactions=False)
    return block["number"]


def main():
    periods = get_recent_light_client_updates(count=4)
    logger.info(f"🎉 Updates fetched for periods: {periods}")

    rev = list(reversed(periods))
    start = get_block_number_for_period(rev[1])
    end = get_block_number_for_period(rev[2])
    logger.info(f"Blocks range: {start} - {end}")

    dump_execution_blocks(start, end)
    dump_execution_blocks(end + 1, end + 1 + BLOCK_WINDOW)


if __name__ == "__main__":
    main()

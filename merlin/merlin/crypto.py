"""
Merlin Cryptographic Utilities

EIP-191 v1 signing with keccak256 hashing.
All epoch seals and state transitions are signed.
Keys never leave the machine.
"""

import json
import hashlib
from typing import Any, Dict, Tuple
from datetime import datetime, timezone

from eth_account import Account
from eth_account.messages import encode_defunct
from eth_utils import keccak, to_checksum_address
from loguru import logger


class MerlinSigner:
    """
    Cryptographic signer for Merlin.
    
    Uses EIP-191 (personal_sign) for all signatures.
    Maintains the private key in memory only.
    """
    
    def __init__(self, private_key: str):
        """Initialize signer with private key"""
        # Normalize private key
        if not private_key.startswith("0x"):
            private_key = f"0x{private_key}"
        
        self.private_key = private_key
        self.account = Account.from_key(private_key)
        self.address = self.account.address
        
        logger.info(f"Signer initialized: {self.address}")
    
    def sign_snapshot(self, data: Dict[str, Any]) -> str:
        """
        Sign a snapshot using EIP-191.
        
        Process:
        1. Serialize to canonical JSON (sorted keys, compact)
        2. Hash with keccak256
        3. Sign with EIP-191 prefix
        4. Return hex signature
        """
        # Remove any existing signature
        clean_data = {k: v for k, v in data.items() if k != "sig"}
        
        # Canonical JSON
        json_str = canonical_json(clean_data)
        
        # Hash
        message_hash = keccak(text=json_str)
        
        # Sign with EIP-191
        message = encode_defunct(primitive=message_hash)
        signed = self.account.sign_message(message)
        
        return signed.signature.hex()
    
    def sign_and_add(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Sign snapshot and add signature to data"""
        sig = self.sign_snapshot(data)
        return {**data, "sig": f"0x{sig}" if not sig.startswith("0x") else sig}


def canonical_json(data: Any) -> str:
    """
    Serialize to canonical JSON.
    
    - Keys sorted alphabetically (recursive)
    - Compact (no whitespace)
    - Deterministic output
    """
    return json.dumps(sort_keys_recursive(data), separators=(",", ":"), sort_keys=True)


def sort_keys_recursive(obj: Any) -> Any:
    """Recursively sort dictionary keys"""
    if isinstance(obj, dict):
        return {k: sort_keys_recursive(v) for k, v in sorted(obj.items())}
    elif isinstance(obj, list):
        return [sort_keys_recursive(item) for item in obj]
    else:
        return obj


def keccak256(data: bytes) -> str:
    """Compute keccak256 hash"""
    return f"0x{keccak(data).hex()}"


def keccak256_str(data: str) -> str:
    """Compute keccak256 hash of string"""
    return keccak256(data.encode("utf-8"))


def compute_merkle_root(items: list[str]) -> str:
    """
    Compute Merkle root from list of CIDs/hashes.
    
    Simple implementation:
    - Sort items
    - Pair and hash recursively
    - Return root hash
    """
    if not items:
        return "0x" + "0" * 64
    
    # Sort for determinism
    sorted_items = sorted(items)
    
    # Convert to hashes if they're CIDs
    hashes = []
    for item in sorted_items:
        if item.startswith("0x"):
            hashes.append(bytes.fromhex(item[2:]))
        else:
            # Hash the CID
            hashes.append(keccak(item.encode("utf-8")))
    
    # Build tree
    while len(hashes) > 1:
        if len(hashes) % 2 == 1:
            hashes.append(hashes[-1])  # Duplicate last if odd
        
        new_level = []
        for i in range(0, len(hashes), 2):
            combined = hashes[i] + hashes[i + 1]
            new_level.append(keccak(combined))
        hashes = new_level
    
    return f"0x{hashes[0].hex()}"


def verify_signature(data: Dict[str, Any], signature: str, expected_address: str) -> bool:
    """
    Verify a signature was made by the expected address.
    
    Returns True if signature is valid and from expected address.
    """
    try:
        # Remove signature from data for verification
        clean_data = {k: v for k, v in data.items() if k != "sig"}
        
        # Canonical JSON and hash
        json_str = canonical_json(clean_data)
        message_hash = keccak(text=json_str)
        
        # Recover signer
        message = encode_defunct(primitive=message_hash)
        
        # Normalize signature
        if signature.startswith("0x"):
            signature = signature[2:]
        sig_bytes = bytes.fromhex(signature)
        
        recovered = Account.recover_message(message, signature=sig_bytes)
        
        # Compare addresses
        return to_checksum_address(recovered) == to_checksum_address(expected_address)
    
    except Exception as e:
        logger.warning(f"Signature verification failed: {e}")
        return False


def generate_nonce() -> str:
    """Generate random nonce for replay protection"""
    import secrets
    return secrets.token_hex(16)


def generate_snapshot_id(prefix: str) -> str:
    """Generate unique snapshot ID"""
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d%H%M%S")
    nonce = generate_nonce()[:8]
    return f"{prefix}-{timestamp}-{nonce}"

"""
Merlin Configuration

Environment-based configuration with sensible defaults.
All secrets come from environment variables.
"""

import os
from dataclasses import dataclass, field
from typing import Optional
from pathlib import Path

from loguru import logger


@dataclass
class MerlinConfig:
    """Merlin daemon configuration"""
    
    # Identity
    identity: str = "merlin.swarmos.eth"
    private_key: str = ""  # Required, from MERLIN_PRIVATE_KEY
    
    # Pool
    pool: str = "swarmpool.eth"
    
    # IPFS
    ipfs_api: str = "http://localhost:5001"
    ipfs_gateway: str = "https://ipfs.io/ipfs"
    
    # Epoch settings
    epoch_duration_seconds: int = 3600  # 1 hour default
    epoch_name_alphabet: str = "NATO"  # NATO phonetic alphabet
    
    # Polling
    poll_interval_seconds: int = 10
    
    # Settlement
    provider_share: float = 0.75  # 75%
    network_ops_share: float = 0.25  # 25%
    
    # Paths (IPFS MFS)
    swarmpool_root: str = "/swarmpool"
    swarmledger_root: str = "/swarmledger"
    
    # Directories
    jobs_dir: str = field(default="")
    claims_dir: str = field(default="")
    proofs_dir: str = field(default="")
    epochs_dir: str = field(default="")
    
    def __post_init__(self):
        # Set directory paths
        self.jobs_dir = f"{self.swarmpool_root}/jobs"
        self.claims_dir = f"{self.swarmpool_root}/claims"
        self.proofs_dir = f"{self.swarmpool_root}/proofs"
        self.epochs_dir = f"{self.swarmledger_root}/epochs"
        
        # Validate
        if not self.private_key:
            raise ValueError("MERLIN_PRIVATE_KEY is required")


def load_config() -> MerlinConfig:
    """Load configuration from environment"""
    
    private_key = os.environ.get("MERLIN_PRIVATE_KEY", "")
    if not private_key:
        logger.error("MERLIN_PRIVATE_KEY environment variable is required")
        raise ValueError("MERLIN_PRIVATE_KEY is required")
    
    config = MerlinConfig(
        identity=os.environ.get("MERLIN_IDENTITY", "merlin.swarmos.eth"),
        private_key=private_key,
        pool=os.environ.get("SWARM_POOL", "swarmpool.eth"),
        ipfs_api=os.environ.get("IPFS_API", "http://localhost:5001"),
        ipfs_gateway=os.environ.get("IPFS_GATEWAY", "https://ipfs.io/ipfs"),
        epoch_duration_seconds=int(os.environ.get("EPOCH_DURATION_SECONDS", "3600")),
        poll_interval_seconds=int(os.environ.get("POLL_INTERVAL_SECONDS", "10")),
        provider_share=float(os.environ.get("PROVIDER_SHARE", "0.75")),
        network_ops_share=float(os.environ.get("NETWORK_OPS_SHARE", "0.25")),
    )
    
    logger.info(f"Configuration loaded:")
    logger.info(f"  Identity: {config.identity}")
    logger.info(f"  Pool: {config.pool}")
    logger.info(f"  IPFS API: {config.ipfs_api}")
    logger.info(f"  Epoch Duration: {config.epoch_duration_seconds}s")
    logger.info(f"  Provider Share: {config.provider_share * 100}%")
    
    return config


# NATO phonetic alphabet for epoch names
NATO_ALPHABET = [
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot",
    "Golf", "Hotel", "India", "Juliet", "Kilo", "Lima",
    "Mike", "November", "Oscar", "Papa", "Quebec", "Romeo",
    "Sierra", "Tango", "Uniform", "Victor", "Whiskey", "X-ray",
    "Yankee", "Zulu",
]


def get_epoch_name(epoch_number: int) -> str:
    """Generate epoch name from NATO alphabet"""
    return NATO_ALPHABET[epoch_number % len(NATO_ALPHABET)]

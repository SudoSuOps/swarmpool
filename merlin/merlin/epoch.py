"""
Epoch Manager for Merlin

Handles the complete epoch lifecycle:
1. Opening new epochs
2. Tracking proofs within epochs
3. Computing settlements
4. Sealing epochs with merkle roots
5. Publishing to SwarmLedger

An epoch is a time-bounded container for proofs.
When sealed, it becomes immutable truth.
"""

from datetime import datetime, timezone
from typing import Any, Dict, List, Optional
from collections import defaultdict

from loguru import logger

from .config import MerlinConfig, get_epoch_name
from .crypto import MerlinSigner, compute_merkle_root, generate_snapshot_id, verify_signature
from .publisher import IPFSPublisher
from .schemas import validate_proof


class EpochManager:
    """
    Epoch Manager — The heart of Merlin.
    
    Responsibilities:
    - Open epochs on schedule
    - Collect and validate proofs
    - Compute settlements (75% workers, 25% ops)
    - Seal epochs with cryptographic finality
    - Publish to SwarmLedger
    """
    
    def __init__(
        self,
        config: MerlinConfig,
        signer: MerlinSigner,
        publisher: IPFSPublisher
    ):
        self.config = config
        self.signer = signer
        self.publisher = publisher
        
        # Current epoch state
        self.current_epoch: Optional[Dict[str, Any]] = None
        self.current_epoch_id: Optional[str] = None
        self.epoch_started_at: Optional[datetime] = None
        self.current_proofs: List[Dict[str, Any]] = []
        self.processed_proof_ids: set = set()
        
        # Epoch counter (persisted via IPFS in production)
        self.epoch_number: int = 0
    
    async def ensure_active_epoch(self):
        """Ensure there's an active epoch, open one if not"""
        if self.current_epoch is None:
            await self.open_new_epoch()
    
    async def open_new_epoch(self):
        """Open a new epoch"""
        self.epoch_number += 1
        epoch_name = get_epoch_name(self.epoch_number)
        epoch_id = f"epoch-{self.epoch_number:04d}"
        
        now = datetime.now(timezone.utc)
        timestamp = int(now.timestamp())
        
        epoch = {
            "type": "epoch",
            "version": "1.0.0",
            "epoch_id": epoch_id,
            "epoch_number": self.epoch_number,
            "name": epoch_name,
            "status": "active",
            "started_at": timestamp,
            "ended_at": None,
            "jobs_count": 0,
            "proofs_count": 0,
            "total_volume_usdc": "0.00",
            "merkle_root": None,
            "settlements": None,
            "proofs": [],
            "controller": self.config.identity,
            "timestamp": timestamp,
        }
        
        # Sign and publish
        signed_epoch = self.signer.sign_and_add(epoch)
        cid = await self.publisher.publish_epoch(signed_epoch)
        
        # Update state
        self.current_epoch = signed_epoch
        self.current_epoch_id = epoch_id
        self.epoch_started_at = now
        self.current_proofs = []
        self.processed_proof_ids = set()
        
        # Announce
        await self.publisher.pubsub_publish(
            f"/{self.config.pool}/epochs/opened",
            {
                "epoch_id": epoch_id,
                "name": epoch_name,
                "started_at": timestamp,
                "cid": cid,
            }
        )
        
        logger.info(f"📖 Epoch opened: {epoch_id} ({epoch_name})")
        return epoch_id
    
    def should_seal(self) -> bool:
        """Check if current epoch should be sealed"""
        if self.epoch_started_at is None:
            return False
        
        elapsed = (datetime.now(timezone.utc) - self.epoch_started_at).total_seconds()
        return elapsed >= self.config.epoch_duration_seconds
    
    async def process_proof(self, proof: Dict[str, Any]) -> bool:
        """
        Process and validate a proof.
        
        Returns True if proof is valid and added to epoch.
        """
        proof_id = proof.get("proof_id", "unknown")
        
        # Skip if already processed
        if proof_id in self.processed_proof_ids:
            return False
        
        # Validate schema
        if not validate_proof(proof):
            logger.warning(f"Proof {proof_id} failed schema validation")
            return False
        
        # Verify signature (if provider ENS can be resolved)
        # In production: resolve ENS and verify
        # For now: trust the signature exists
        if not proof.get("sig"):
            logger.warning(f"Proof {proof_id} missing signature")
            return False
        
        # Fetch job to get reward amount
        job_cid = proof.get("job_cid")
        job = None
        if job_cid:
            job = await self.publisher.fetch_job(job_cid)
        
        # Add to current epoch
        self.current_proofs.append({
            "proof": proof,
            "job": job,
            "processed_at": int(datetime.now(timezone.utc).timestamp()),
        })
        self.processed_proof_ids.add(proof_id)
        
        logger.debug(f"Added proof {proof_id} to epoch")
        return True
    
    async def seal_current_epoch(self) -> Optional[str]:
        """
        Seal the current epoch.
        
        1. Gather all valid proofs
        2. Compute settlements
        3. Compute merkle root
        4. Sign epoch seal
        5. Publish to SwarmLedger
        """
        if not self.current_epoch:
            return None
        
        now = datetime.now(timezone.utc)
        timestamp = int(now.timestamp())
        
        # Compute settlements
        settlements = self._compute_settlements()
        
        # Get proof CIDs for merkle root
        proof_cids = []
        for p in self.current_proofs:
            proof = p["proof"]
            # In production: get actual CID from IPFS
            proof_cids.append(proof.get("proof_id", ""))
        
        # Compute merkle root
        merkle_root = compute_merkle_root(proof_cids) if proof_cids else "0x" + "0" * 64
        
        # Update epoch
        sealed_epoch = {
            **self.current_epoch,
            "status": "sealed",
            "ended_at": timestamp,
            "jobs_count": len(self.current_proofs),
            "proofs_count": len(self.current_proofs),
            "total_volume_usdc": f"{settlements['total_volume']:.2f}",
            "merkle_root": merkle_root,
            "settlements": settlements,
            "proofs": proof_cids,
            "timestamp": timestamp,
        }
        
        # Remove old signature, re-sign
        if "sig" in sealed_epoch:
            del sealed_epoch["sig"]
        sealed_epoch = self.signer.sign_and_add(sealed_epoch)
        
        # Publish
        cid = await self.publisher.publish_epoch(sealed_epoch)
        
        # Announce
        await self.publisher.pubsub_publish(
            f"/{self.config.pool}/epochs/sealed",
            {
                "epoch_id": self.current_epoch_id,
                "jobs_count": len(self.current_proofs),
                "total_volume": settlements["total_volume"],
                "merkle_root": merkle_root,
                "cid": cid,
            }
        )
        
        logger.info(f"🔒 Epoch sealed: {self.current_epoch_id}")
        logger.info(f"   Jobs: {len(self.current_proofs)}")
        logger.info(f"   Volume: ${settlements['total_volume']:.2f}")
        logger.info(f"   Merkle: {merkle_root[:18]}...")
        logger.info(f"   CID: {cid}")
        
        # Log provider earnings
        if settlements["providers"]:
            logger.info("   Provider payouts:")
            for provider, amount in sorted(
                settlements["providers"].items(),
                key=lambda x: x[1],
                reverse=True
            )[:5]:
                logger.info(f"     {provider}: ${amount:.4f}")
        
        return cid
    
    def _compute_settlements(self) -> Dict[str, Any]:
        """
        Compute settlements for the epoch.
        
        Distribution:
        - 75% to providers (who did the work)
        - 25% to network ops (infrastructure)
        """
        total_volume = 0.0
        provider_earnings: Dict[str, float] = defaultdict(float)
        
        for item in self.current_proofs:
            proof = item["proof"]
            job = item.get("job")
            
            # Get reward from job
            reward = 0.10  # Default
            if job and "payment" in job:
                try:
                    reward = float(job["payment"].get("amount", "0.10"))
                except (ValueError, TypeError):
                    reward = 0.10
            
            total_volume += reward
            
            # Track provider earnings
            provider = proof.get("provider", "unknown")
            provider_share = reward * self.config.provider_share
            provider_earnings[provider] += provider_share
        
        return {
            "total_volume": total_volume,
            "provider_pool": total_volume * self.config.provider_share,
            "network_ops": total_volume * self.config.network_ops_share,
            "providers": dict(provider_earnings),
            "provider_count": len(provider_earnings),
        }

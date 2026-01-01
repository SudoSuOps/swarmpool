"""
Proof Watcher for Merlin

Watches /swarmpool/proofs/ for new proof snapshots.
Polls IPFS directory and returns new proofs for processing.

NO inbound connections. Merlin only READS from IPFS.
"""

from typing import Any, Dict, List, Set
from datetime import datetime, timezone

from loguru import logger

from .config import MerlinConfig
from .publisher import IPFSPublisher


class ProofWatcher:
    """
    Proof Watcher — Merlin's eyes on the pool.
    
    Polls /swarmpool/proofs/ for new proof snapshots.
    Maintains state of seen proofs to avoid reprocessing.
    """
    
    def __init__(self, config: MerlinConfig, publisher: IPFSPublisher):
        self.config = config
        self.publisher = publisher
        
        # Track seen proofs
        self.seen_proofs: Set[str] = set()
        
        # Stats
        self.total_seen: int = 0
        self.last_poll: datetime = datetime.now(timezone.utc)
    
    async def poll(self) -> List[Dict[str, Any]]:
        """
        Poll for new proofs.
        
        Returns list of new proof snapshots not previously seen.
        """
        new_proofs: List[Dict[str, Any]] = []
        
        try:
            # List all proofs in directory
            proof_ids = await self.publisher.list_proofs()
            
            for proof_id in proof_ids:
                # Skip if already seen
                if proof_id in self.seen_proofs:
                    continue
                
                # Fetch proof
                proof = await self.publisher.fetch_proof(proof_id)
                
                if proof:
                    new_proofs.append(proof)
                    self.seen_proofs.add(proof_id)
                    self.total_seen += 1
                    logger.debug(f"New proof found: {proof_id}")
            
            self.last_poll = datetime.now(timezone.utc)
            
            if new_proofs:
                logger.info(f"Found {len(new_proofs)} new proof(s)")
            
        except Exception as e:
            logger.error(f"Error polling proofs: {e}")
        
        return new_proofs
    
    def reset(self):
        """Reset seen proofs (for new epoch)"""
        # Keep seen proofs to avoid reprocessing
        # In production, you might want epoch-scoped tracking
        pass
    
    @property
    def stats(self) -> Dict[str, Any]:
        """Get watcher statistics"""
        return {
            "total_seen": self.total_seen,
            "known_proofs": len(self.seen_proofs),
            "last_poll": self.last_poll.isoformat(),
        }

"""
merlin.swarmos.eth — SwarmOS Controller Daemon

The calm control-plane daemon for SwarmOS.
Merlin does NOT compute and does NOT assign jobs.
Merlin opens epochs, validates proofs, and seals final settlement.

WHAT MERLIN IS:
  ⏱ Epoch Clock — opens/closes epochs deterministically
  🧾 Settlement Pen — computes payouts and writes final truth
  🔐 Verifier — validates schemas + signatures + proof integrity
  📦 Publisher — publishes signed snapshots to IPFS canonical layout
  🧠 Source of Truth for Ledger Seals — produces EPOCH_SEAL snapshots

WHAT MERLIN IS NOT:
  ❌ Not a scheduler
  ❌ Not a load balancer
  ❌ Not a worker
  ❌ Not an API gateway
  ❌ Not a cloud service
  ❌ Not a centralized authority over compute

Merlin never instructs miners to run.
Miners choose jobs from Swarmpool voluntarily.
"""

import asyncio
import os
import sys
from pathlib import Path
from datetime import datetime, timezone
from typing import Optional

from loguru import logger

from .config import MerlinConfig, load_config
from .epoch import EpochManager
from .watcher import ProofWatcher
from .publisher import IPFSPublisher
from .crypto import MerlinSigner

# Configure logging
logger.remove()
logger.add(
    sys.stderr,
    format="<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan> - {message}",
    level="INFO",
)
logger.add(
    "logs/merlin_{time:YYYY-MM-DD}.log",
    rotation="1 day",
    retention="30 days",
    level="DEBUG",
)


class Merlin:
    """
    Merlin Daemon — SwarmOS Controller
    
    A state machine that:
    1. Opens epochs on a schedule
    2. Watches for proofs in /swarmpool/proofs/
    3. Validates proofs (schema, signature, integrity)
    4. Seals epochs with settlements
    5. Publishes final truth to /swarmledger/epochs/
    
    Merlin has NO:
    - HTTP endpoints
    - Inbound connections
    - Job assignment logic
    - Worker management
    """
    
    def __init__(self, config: MerlinConfig):
        self.config = config
        self.identity = config.identity  # merlin.swarmos.eth
        
        # Components
        self.signer = MerlinSigner(config.private_key)
        self.publisher = IPFSPublisher(config.ipfs_api)
        self.epoch_manager = EpochManager(config, self.signer, self.publisher)
        self.proof_watcher = ProofWatcher(config, self.publisher)
        
        # State
        self.running = False
        self.start_time: Optional[datetime] = None
        
        logger.info(f"🧙‍♂️ Merlin initialized: {self.identity}")
    
    async def start(self):
        """Start the Merlin daemon"""
        self.running = True
        self.start_time = datetime.now(timezone.utc)
        
        logger.info("=" * 60)
        logger.info("🧙‍♂️ MERLIN DAEMON STARTING")
        logger.info(f"   Identity: {self.identity}")
        logger.info(f"   Pool: {self.config.pool}")
        logger.info(f"   Epoch Duration: {self.config.epoch_duration_seconds}s")
        logger.info("=" * 60)
        
        # Initialize IPFS directories
        await self.publisher.init_directories()
        
        # Open initial epoch if needed
        await self.epoch_manager.ensure_active_epoch()
        
        # Start the main loop
        await self._run_loop()
    
    async def stop(self):
        """Stop the Merlin daemon gracefully"""
        logger.info("🧙‍♂️ Merlin shutting down...")
        self.running = False
        
        # Seal current epoch if it has work
        if self.epoch_manager.current_epoch:
            logger.info("Sealing current epoch before shutdown...")
            await self.epoch_manager.seal_current_epoch()
        
        logger.info("🧙‍♂️ Merlin stopped.")
    
    async def _run_loop(self):
        """Main daemon loop — watch and seal"""
        logger.info("Starting main loop...")
        
        while self.running:
            try:
                # 1. Check for new proofs
                new_proofs = await self.proof_watcher.poll()
                
                for proof in new_proofs:
                    # Validate and add to current epoch
                    if await self.epoch_manager.process_proof(proof):
                        logger.info(f"✅ Valid proof: {proof.get('proof_id', 'unknown')}")
                    else:
                        logger.warning(f"❌ Invalid proof rejected")
                
                # 2. Check if epoch should be sealed
                if self.epoch_manager.should_seal():
                    await self.epoch_manager.seal_current_epoch()
                    await self.epoch_manager.open_new_epoch()
                
                # 3. Publish heartbeat
                await self._publish_heartbeat()
                
                # Sleep between iterations
                await asyncio.sleep(self.config.poll_interval_seconds)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error in main loop: {e}")
                await asyncio.sleep(5)  # Back off on error
    
    async def _publish_heartbeat(self):
        """Publish controller heartbeat"""
        if not hasattr(self, '_last_heartbeat'):
            self._last_heartbeat = 0
        
        now = datetime.now(timezone.utc).timestamp()
        if now - self._last_heartbeat < 30:  # Every 30 seconds
            return
        
        self._last_heartbeat = now
        
        heartbeat = {
            "type": "heartbeat",
            "controller": self.identity,
            "current_epoch": self.epoch_manager.current_epoch_id,
            "epoch_proofs": len(self.epoch_manager.current_proofs),
            "uptime_seconds": int(now - self.start_time.timestamp()) if self.start_time else 0,
            "timestamp": int(now),
        }
        
        try:
            await self.publisher.pubsub_publish(
                f"/{self.config.pool}/heartbeats",
                heartbeat
            )
        except Exception as e:
            logger.debug(f"Heartbeat publish failed: {e}")


async def main():
    """Entry point for Merlin daemon"""
    # Load configuration
    config = load_config()
    
    # Create and start Merlin
    merlin = Merlin(config)
    
    # Handle shutdown signals
    import signal
    
    def signal_handler(sig, frame):
        logger.info(f"Received signal {sig}")
        asyncio.create_task(merlin.stop())
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    try:
        await merlin.start()
    except KeyboardInterrupt:
        await merlin.stop()


if __name__ == "__main__":
    asyncio.run(main())

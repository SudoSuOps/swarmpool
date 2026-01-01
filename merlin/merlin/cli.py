#!/usr/bin/env python3
"""
Merlin CLI — SwarmOS Controller

Usage:
  merlin run              Start the daemon
  merlin seal <epoch_id>  Manually seal an epoch
  merlin status           Show current status
  merlin epochs           List epochs
"""

import asyncio
import sys
import argparse
from datetime import datetime, timezone

from loguru import logger

from . import Merlin, main as daemon_main
from .config import load_config, MerlinConfig
from .crypto import MerlinSigner
from .publisher import IPFSPublisher
from .epoch import EpochManager


def cli():
    """CLI entry point"""
    parser = argparse.ArgumentParser(
        description="Merlin — SwarmOS Controller Daemon",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  merlin run              Start the Merlin daemon
  merlin seal epoch-0042  Manually seal an epoch
  merlin status           Show controller status
  merlin epochs           List recent epochs
        """
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Run command
    run_parser = subparsers.add_parser("run", help="Start the Merlin daemon")
    
    # Seal command
    seal_parser = subparsers.add_parser("seal", help="Manually seal an epoch")
    seal_parser.add_argument("epoch_id", help="Epoch ID to seal")
    
    # Status command
    status_parser = subparsers.add_parser("status", help="Show controller status")
    status_parser.add_argument("--json", action="store_true", help="Output as JSON")
    
    # Epochs command
    epochs_parser = subparsers.add_parser("epochs", help="List epochs")
    epochs_parser.add_argument("--limit", type=int, default=10, help="Number of epochs")
    epochs_parser.add_argument("--id", dest="epoch_id", help="Show specific epoch")
    
    args = parser.parse_args()
    
    if args.command == "run":
        print_banner()
        asyncio.run(daemon_main())
    
    elif args.command == "seal":
        asyncio.run(manual_seal(args.epoch_id))
    
    elif args.command == "status":
        asyncio.run(show_status(json_output=args.json))
    
    elif args.command == "epochs":
        asyncio.run(list_epochs(limit=args.limit, epoch_id=args.epoch_id))
    
    else:
        parser.print_help()


def print_banner():
    """Print Merlin banner"""
    banner = """
    ╔══════════════════════════════════════════════════════════════╗
    ║                                                              ║
    ║   🧙‍♂️ MERLIN — SwarmOS Controller                            ║
    ║                                                              ║
    ║   merlin.swarmos.eth                                        ║
    ║                                                              ║
    ║   Epoch Clock · Settlement Pen · Truth Sealer               ║
    ║                                                              ║
    ╚══════════════════════════════════════════════════════════════╝
    """
    print(banner)


async def manual_seal(epoch_id: str):
    """Manually seal an epoch"""
    print(f"Sealing epoch: {epoch_id}")
    
    config = load_config()
    signer = MerlinSigner(config.private_key)
    publisher = IPFSPublisher(config.ipfs_api)
    
    # Check connection
    if not await publisher.check_connection():
        print("❌ IPFS connection failed")
        sys.exit(1)
    
    # Create minimal epoch manager and seal
    manager = EpochManager(config, signer, publisher)
    
    # In production: load existing epoch state
    # For now: just demonstrate the seal process
    print("⚠️  Manual seal not fully implemented")
    print("    Use 'merlin run' to let Merlin manage epochs automatically")
    
    await publisher.close()


async def show_status(json_output: bool = False):
    """Show controller status"""
    config = load_config()
    publisher = IPFSPublisher(config.ipfs_api)
    
    # Check IPFS
    ipfs_ok = await publisher.check_connection()
    
    status = {
        "controller": config.identity,
        "pool": config.pool,
        "ipfs_connected": ipfs_ok,
        "epoch_duration_seconds": config.epoch_duration_seconds,
        "provider_share": config.provider_share,
        "network_ops_share": config.network_ops_share,
    }
    
    if json_output:
        import json
        print(json.dumps(status, indent=2))
    else:
        print("\n🧙‍♂️ Merlin Status")
        print("─" * 40)
        print(f"  Controller: {config.identity}")
        print(f"  Pool: {config.pool}")
        print(f"  IPFS: {'✅ Connected' if ipfs_ok else '❌ Disconnected'}")
        print(f"  Epoch Duration: {config.epoch_duration_seconds}s")
        print(f"  Provider Share: {config.provider_share * 100}%")
        print()
    
    await publisher.close()


async def list_epochs(limit: int = 10, epoch_id: str = None):
    """List epochs"""
    config = load_config()
    publisher = IPFSPublisher(config.ipfs_api)
    
    if not await publisher.check_connection():
        print("❌ IPFS connection failed")
        await publisher.close()
        return
    
    epochs = await publisher.list_directory("/swarmledger/epochs")
    
    if not epochs:
        print("No epochs found")
        await publisher.close()
        return
    
    # Sort by epoch ID (descending)
    epochs.sort(reverse=True)
    
    if epoch_id:
        # Show specific epoch
        epoch = await publisher.fetch_file(f"/swarmledger/epochs/{epoch_id}.json")
        if epoch:
            import json
            print(json.dumps(epoch, indent=2))
        else:
            print(f"Epoch not found: {epoch_id}")
    else:
        # List epochs
        print("\n📜 Epochs")
        print("─" * 60)
        print(f"{'ID':<16} {'Name':<12} {'Status':<10} {'Jobs':<8} {'Volume':<12}")
        print("─" * 60)
        
        for eid in epochs[:limit]:
            epoch = await publisher.fetch_file(f"/swarmledger/epochs/{eid}.json")
            if epoch:
                status = epoch.get("status", "?")
                name = epoch.get("name", "?")
                jobs = epoch.get("jobs_count", 0)
                volume = epoch.get("total_volume_usdc", "0.00")
                
                status_icon = "🟢" if status == "active" else "✅"
                print(f"{eid:<16} {name:<12} {status_icon} {status:<8} {jobs:<8} ${volume}")
        
        print()
    
    await publisher.close()


if __name__ == "__main__":
    cli()

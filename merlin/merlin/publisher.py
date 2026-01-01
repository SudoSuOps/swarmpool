"""
IPFS Publisher for Merlin

Handles all IPFS operations:
- Publishing snapshots to canonical paths
- Fetching proofs and jobs
- Pubsub announcements
- Directory management

NO inbound connections. Merlin only WRITES to IPFS.
"""

import json
import asyncio
import aiohttp
from typing import Any, Dict, List, Optional
from datetime import datetime, timezone

from loguru import logger


class IPFSPublisher:
    """
    IPFS Publisher for Merlin.
    
    Manages all IPFS interactions:
    - MFS (Mutable File System) for canonical paths
    - Pin for persistence
    - Pubsub for announcements
    """
    
    def __init__(self, api_url: str = "http://localhost:5001"):
        self.api_url = api_url.rstrip("/")
        self.api_base = f"{self.api_url}/api/v0"
        self._session: Optional[aiohttp.ClientSession] = None
    
    async def _get_session(self) -> aiohttp.ClientSession:
        """Get or create aiohttp session"""
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession()
        return self._session
    
    async def close(self):
        """Close the session"""
        if self._session and not self._session.closed:
            await self._session.close()
    
    # =========================================================================
    # CONNECTION
    # =========================================================================
    
    async def check_connection(self) -> bool:
        """Check if IPFS daemon is accessible"""
        try:
            session = await self._get_session()
            async with session.post(f"{self.api_base}/id") as resp:
                if resp.status == 200:
                    data = await resp.json()
                    logger.info(f"IPFS connected: {data.get('ID', 'unknown')[:16]}...")
                    return True
                return False
        except Exception as e:
            logger.error(f"IPFS connection failed: {e}")
            return False
    
    # =========================================================================
    # DIRECTORY MANAGEMENT
    # =========================================================================
    
    async def init_directories(self):
        """Initialize canonical directory structure"""
        dirs = [
            "/swarmpool",
            "/swarmpool/jobs",
            "/swarmpool/claims",
            "/swarmpool/proofs",
            "/swarmpool/genesis",
            "/swarmledger",
            "/swarmledger/epochs",
            "/swarmledger/settlements",
        ]
        
        session = await self._get_session()
        for dir_path in dirs:
            try:
                await session.post(
                    f"{self.api_base}/files/mkdir",
                    params={"arg": dir_path, "parents": "true"}
                )
            except Exception:
                pass  # Directory may already exist
        
        logger.info("IPFS directories initialized")
    
    # =========================================================================
    # PUBLISHING
    # =========================================================================
    
    async def publish_snapshot(
        self,
        data: Dict[str, Any],
        directory: str,
        snapshot_id: str
    ) -> str:
        """
        Publish a snapshot to IPFS.
        
        1. Add JSON to IPFS → get CID
        2. Copy to canonical path
        3. Pin the CID
        4. Return CID
        """
        session = await self._get_session()
        
        # 1. Serialize to canonical JSON
        json_str = json.dumps(data, separators=(",", ":"), sort_keys=True)
        
        # 2. Add to IPFS
        form = aiohttp.FormData()
        form.add_field(
            "file",
            json_str.encode("utf-8"),
            filename=f"{snapshot_id}.json",
            content_type="application/json"
        )
        
        async with session.post(
            f"{self.api_base}/add",
            data=form,
            params={"cid-version": "1"}
        ) as resp:
            if resp.status != 200:
                raise Exception(f"IPFS add failed: {resp.status}")
            result = await resp.json()
            cid = result["Hash"]
        
        # 3. Copy to canonical path
        canonical_path = f"{directory}/{snapshot_id}.json"
        
        # Remove if exists (MFS doesn't overwrite)
        try:
            await session.post(
                f"{self.api_base}/files/rm",
                params={"arg": canonical_path, "force": "true"}
            )
        except Exception:
            pass
        
        # Copy
        async with session.post(
            f"{self.api_base}/files/cp",
            params={"arg": f"/ipfs/{cid}", "arg": canonical_path}
        ) as resp:
            if resp.status != 200:
                logger.warning(f"Failed to copy to canonical path: {canonical_path}")
        
        # 4. Pin
        try:
            await session.post(
                f"{self.api_base}/pin/add",
                params={"arg": cid}
            )
        except Exception as e:
            logger.warning(f"Failed to pin {cid}: {e}")
        
        logger.debug(f"Published: {cid} → {canonical_path}")
        return cid
    
    async def publish_epoch(self, epoch_data: Dict[str, Any]) -> str:
        """Publish epoch snapshot to SwarmLedger"""
        epoch_id = epoch_data.get("epoch_id", "unknown")
        return await self.publish_snapshot(
            epoch_data,
            "/swarmledger/epochs",
            epoch_id
        )
    
    # =========================================================================
    # FETCHING
    # =========================================================================
    
    async def list_directory(self, path: str) -> List[str]:
        """List files in an IPFS MFS directory"""
        session = await self._get_session()
        
        try:
            async with session.post(
                f"{self.api_base}/files/ls",
                params={"arg": path, "long": "true"}
            ) as resp:
                if resp.status != 200:
                    return []
                result = await resp.json()
                entries = result.get("Entries", []) or []
                return [
                    e["Name"].replace(".json", "")
                    for e in entries
                    if e["Name"].endswith(".json")
                ]
        except Exception as e:
            logger.debug(f"Failed to list {path}: {e}")
            return []
    
    async def fetch_file(self, path: str) -> Optional[Dict[str, Any]]:
        """Fetch JSON file from IPFS MFS path"""
        session = await self._get_session()
        
        try:
            async with session.post(
                f"{self.api_base}/files/read",
                params={"arg": path}
            ) as resp:
                if resp.status != 200:
                    return None
                content = await resp.text()
                return json.loads(content)
        except Exception as e:
            logger.debug(f"Failed to fetch {path}: {e}")
            return None
    
    async def fetch_by_cid(self, cid: str) -> Optional[Dict[str, Any]]:
        """Fetch JSON by CID"""
        session = await self._get_session()
        
        try:
            async with session.post(
                f"{self.api_base}/cat",
                params={"arg": cid}
            ) as resp:
                if resp.status != 200:
                    return None
                content = await resp.text()
                return json.loads(content)
        except Exception as e:
            logger.debug(f"Failed to fetch CID {cid}: {e}")
            return None
    
    async def get_file_cid(self, path: str) -> Optional[str]:
        """Get CID for a file at MFS path"""
        session = await self._get_session()
        
        try:
            async with session.post(
                f"{self.api_base}/files/stat",
                params={"arg": path}
            ) as resp:
                if resp.status != 200:
                    return None
                result = await resp.json()
                return result.get("Hash")
        except Exception:
            return None
    
    # =========================================================================
    # PUBSUB
    # =========================================================================
    
    async def pubsub_publish(self, topic: str, data: Dict[str, Any]):
        """Publish message to IPFS pubsub topic"""
        session = await self._get_session()
        
        try:
            import base64
            json_str = json.dumps(data)
            encoded = base64.b64encode(json_str.encode()).decode()
            
            await session.post(
                f"{self.api_base}/pubsub/pub",
                params={"arg": topic},
                data=encoded
            )
            logger.debug(f"Pubsub published to {topic}")
        except Exception as e:
            logger.debug(f"Pubsub publish failed: {e}")
    
    # =========================================================================
    # PROOFS
    # =========================================================================
    
    async def list_proofs(self) -> List[str]:
        """List all proof IDs in /swarmpool/proofs/"""
        return await self.list_directory("/swarmpool/proofs")
    
    async def fetch_proof(self, proof_id: str) -> Optional[Dict[str, Any]]:
        """Fetch a specific proof"""
        return await self.fetch_file(f"/swarmpool/proofs/{proof_id}.json")
    
    async def fetch_job(self, job_cid: str) -> Optional[Dict[str, Any]]:
        """Fetch a job by CID"""
        return await self.fetch_by_cid(job_cid)

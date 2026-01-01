"""
Schema Validation for Merlin

Validates proof snapshots before accepting them.
Invalid proofs are rejected — schema is law.
"""

from typing import Any, Dict
import re

from loguru import logger


def validate_proof(proof: Dict[str, Any]) -> bool:
    """
    Validate a proof snapshot against schema.
    
    Required fields:
    - type: "proof"
    - version: semver
    - proof_id: string
    - job_cid: IPFS CID
    - output_cid: IPFS CID
    - metrics: object with inference_seconds, confidence
    - provider: ENS name
    - timestamp: integer
    - proof_hash: 0x-prefixed hash
    - sig: 0x-prefixed signature
    """
    errors = []
    
    # Type check
    if proof.get("type") != "proof":
        errors.append(f"type must be 'proof', got '{proof.get('type')}'")
    
    # Required string fields
    required_strings = ["proof_id", "job_cid", "output_cid", "provider", "proof_hash", "sig"]
    for field in required_strings:
        if not proof.get(field):
            errors.append(f"missing required field: {field}")
        elif not isinstance(proof.get(field), str):
            errors.append(f"{field} must be a string")
    
    # Timestamp
    if not isinstance(proof.get("timestamp"), (int, float)):
        errors.append("timestamp must be a number")
    
    # Metrics validation
    metrics = proof.get("metrics")
    if not isinstance(metrics, dict):
        errors.append("metrics must be an object")
    else:
        if not isinstance(metrics.get("inference_seconds"), (int, float)):
            errors.append("metrics.inference_seconds must be a number")
        if not isinstance(metrics.get("confidence"), (int, float)):
            errors.append("metrics.confidence must be a number")
        else:
            conf = metrics.get("confidence", 0)
            if conf < 0 or conf > 1:
                errors.append("metrics.confidence must be between 0 and 1")
    
    # CID format (basic check)
    cid_pattern = re.compile(r'^(bafy|Qm)[a-zA-Z0-9]+')
    for field in ["job_cid", "output_cid"]:
        value = proof.get(field, "")
        if value and not cid_pattern.match(value):
            errors.append(f"{field} does not look like a valid CID")
    
    # Hash format
    if proof.get("proof_hash") and not proof["proof_hash"].startswith("0x"):
        errors.append("proof_hash must be 0x-prefixed")
    
    # Signature format
    if proof.get("sig") and not proof["sig"].startswith("0x"):
        errors.append("sig must be 0x-prefixed")
    
    # ENS format (basic check)
    provider = proof.get("provider", "")
    if provider and not provider.endswith(".eth"):
        errors.append("provider must be an ENS name (ending in .eth)")
    
    # Log errors
    if errors:
        logger.warning(f"Proof validation failed: {errors}")
        return False
    
    return True


def validate_job(job: Dict[str, Any]) -> bool:
    """Validate a job snapshot"""
    errors = []
    
    if job.get("type") != "job":
        errors.append("type must be 'job'")
    
    required = ["job_id", "model", "input_cid", "client", "timestamp", "sig"]
    for field in required:
        if not job.get(field):
            errors.append(f"missing required field: {field}")
    
    # Payment
    payment = job.get("payment", {})
    if not payment.get("amount"):
        errors.append("payment.amount is required")
    
    if errors:
        logger.warning(f"Job validation failed: {errors}")
        return False
    
    return True


def validate_epoch(epoch: Dict[str, Any]) -> bool:
    """Validate an epoch snapshot"""
    errors = []
    
    if epoch.get("type") != "epoch":
        errors.append("type must be 'epoch'")
    
    if epoch.get("status") not in ["active", "sealed"]:
        errors.append("status must be 'active' or 'sealed'")
    
    required = ["epoch_id", "name", "started_at", "controller", "timestamp", "sig"]
    for field in required:
        if not epoch.get(field):
            errors.append(f"missing required field: {field}")
    
    # Sealed epochs need additional fields
    if epoch.get("status") == "sealed":
        sealed_required = ["ended_at", "merkle_root", "settlements"]
        for field in sealed_required:
            if epoch.get(field) is None:
                errors.append(f"sealed epoch missing: {field}")
    
    if errors:
        logger.warning(f"Epoch validation failed: {errors}")
        return False
    
    return True

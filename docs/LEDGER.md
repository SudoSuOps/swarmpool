# SwarmLedger

SwarmLedger is the immutable book of record.

---

## What It Contains

- Epoch seals
- Payout tables
- Final compute totals
- Provider earnings
- Settlement proofs

---

## Properties

- **Deterministic** — Same inputs produce same outputs
- **Auditable** — Anyone can verify calculations
- **Append-only** — Sealed epochs are immutable
- **IPFS-backed** — Content-addressed storage

---

## Epoch Seal Structure

```json
{
  "snapshot_type": "epoch",
  "epoch_id": "048",
  "epoch_name": "Golf",
  "start_time": 1735689600,
  "end_time": 1735693200,
  "total_jobs": 156,
  "total_volume": 15.60,
  "miner_pool": 11.70,
  "hive_ops": 3.90,
  "settlements": { ... },
  "sig": "0x..."
}
```

---

## Canon Rule

> If it is not sealed in the ledger, it did not happen.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

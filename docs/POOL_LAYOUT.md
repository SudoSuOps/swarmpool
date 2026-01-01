# IPFS Pool Layout

Swarmpool uses a canonical directory structure on IPFS MFS.

---

## Directory Structure

```
/swarmpool/
├── genesis/
│   └── {provider}.json
├── jobs/
│   └── {job_id}.json
├── claims/
│   └── {claim_id}.json
├── proofs/
│   └── {proof_id}.json
├── epochs/
│   └── {epoch_id}.json
└── index/
    └── latest.json
```

---

## Snapshot Types

| Type | Location | Publisher |
|------|----------|-----------|
| Genesis | `/genesis/{provider}.json` | Provider |
| Job | `/jobs/{job_id}.json` | Client |
| Claim | `/claims/{claim_id}.json` | Provider |
| Proof | `/proofs/{proof_id}.json` | Provider |
| Epoch | `/epochs/{epoch_id}.json` | Merlin |

---

## Canon Rule

> Content-addressed. Append-only. Signed.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

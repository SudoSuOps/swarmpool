# Swarm Architecture

Swarm is built as a **strictly separated tri-layer system**.

Each layer answers one question — and only one.

---

## Swarmpool — Opportunity
**What work is available right now?**

- Public, IPFS-backed job mempool
- Append-only, signed snapshots
- No schedulers, no assignment
- Readable by anyone

Swarmpool exposes opportunity. It does not settle truth.

---

## SwarmLedger — Truth
**What actually happened, and who earned what?**

- Aggregates proofs
- Computes deterministic payouts
- Seals epochs immutably
- Source of record

SwarmLedger never executes compute.

---

## SwarmOrb — Meaning
**What does the system state imply?**

- Observes Swarmpool + SwarmLedger
- Produces health and capacity signals
- Read-only, no authority

If Orb disappears, nothing breaks.

---

## Data Flow

```
Clients → Swarmpool → SwarmLedger → SwarmOrb
```

No circular dependencies.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

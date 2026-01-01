# Merlin

Merlin is the SwarmOS epoch controller.

**ENS:** `merlin.swarmos.eth`

---

## Responsibilities

- Open epochs
- Verify proofs
- Calculate settlements
- Seal epochs to SwarmLedger
- Publish ledger truth

---

## Non-Responsibilities

- No compute execution
- No job scheduling
- No provider assignment
- No data processing

---

## Epoch Lifecycle

1. **Open** — New epoch begins, jobs accumulate
2. **Active** — Providers claim and prove
3. **Closing** — Grace period for final proofs
4. **Sealed** — Settlement calculated, epoch immutable

---

## Settlement Process

1. Aggregate all valid proofs in epoch
2. Group by job
3. Apply SOLO/PPL payout logic
4. Calculate miner pool (75%) and hive ops (25%)
5. Publish sealed epoch to `/epochs/{id}.json`

---

## Canon Rule

> Merlin governs time and truth — never execution.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

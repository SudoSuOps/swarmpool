# SwarmEpoch

SwarmEpoch is the **archive of finality**.

**ENS:** `swarmepoch.eth`

---

## Role

SwarmEpoch renders what Merlin already sealed.

It does not:
- Calculate payouts
- Decide winners
- Modify state

It only:
- Displays sealed epochs
- Links to IPFS CIDs
- Provides historical reference

---

## Relationship to Swarmpool

| Layer | State | Purpose |
|-------|-------|---------|
| Swarmpool | Live | Current opportunity |
| SwarmEpoch | Final | Sealed history |

Swarmpool shows what's happening now.
SwarmEpoch shows what already happened.

---

## Data Source

SwarmEpoch reads from SwarmLedger.

```
Merlin seals → SwarmLedger stores → SwarmEpoch renders
```

No intermediate processing. No reinterpretation.

---

## Canon Rule

> Swarmpool is live. SwarmEpoch is final. ENS points. IPFS remembers.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

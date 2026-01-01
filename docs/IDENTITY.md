# Identity & ENS

All actors in Swarm are identified by ENS.

---

## Namespace Roles

| Namespace | Role | Example |
|-----------|------|---------|
| `swarmos.eth` | Protocol / control plane | `merlin.swarmos.eth` |
| `swarmbee.eth` | Execution / workers | `bumble70b.swarmbee.eth` |
| `swarmpool.eth` | Opportunity surface | — |
| `swarmhive.eth` | Model registry | — |
| `swarmorb.eth` | Analytics / observation | — |

---

## Identity Resolution

1. Signature recovered from snapshot
2. Address resolved to ENS
3. ENS namespace determines authority scope

---

## Canon Rule

> Identity is resolved via ENS.
> Authority is proven by signature.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

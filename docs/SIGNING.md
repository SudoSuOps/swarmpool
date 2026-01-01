# Signing & Verification

All Swarm snapshots are cryptographically signed.

---

## Signature Standard

- **EIP-191** Ethereum signed messages
- **keccak256** hash of canonical JSON
- ENS-bound identity

---

## Signing Process

1. Serialize snapshot to canonical JSON (sorted keys, no whitespace)
2. Hash with keccak256
3. Sign with EIP-191 prefix: `\x19Ethereum Signed Message:\n{length}{hash}`
4. Attach signature to snapshot

---

## Verification

```
signature → recover address → resolve ENS → verify authority
```

---

## Canon Rule

> No signature = ignored.
> Invalid signature = rejected.
> Unsigned snapshots do not exist.

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

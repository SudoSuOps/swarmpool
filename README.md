# SwarmPool

**A Miner-Honest Compute Pool**

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║   CLIENTS SUBMIT JOBS → MINERS DECIDE → PROOFS SETTLE → EVERYONE PAID      ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

## What Is SwarmPool?

Swarmpool is a visible, IPFS-backed mempool where sovereign compute providers choose when and how to work.

- Jobs are published openly
- Miners decide
- Proof settles

**No schedulers. No dark room. No token.**

## Why SwarmPool Exists

Traditional cloud compute hides demand behind opaque schedulers.
Compute providers submit capacity blindly and wait.

SwarmPool flips the model.

Here, opportunity is visible before power is committed — enabling intelligent, miner-native behavior.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           SWARMPOOL                                          │
│                      "What work is available?"                               │
│                                                                              │
│  /swarmpool/jobs/      ← job submissions                                     │
│  /swarmpool/claims/    ← miner intent (SOLO/PPL)                            │
│  /swarmpool/proofs/    ← compute results                                     │
│  /swarmpool/genesis/   ← provider registrations                              │
└──────────────────────────────────────────────────────────────────────────────┘
                                   │
                          Merlin seals ↓
                                   │
┌──────────────────────────────────────────────────────────────────────────────┐
│                          SWARMLEDGER                                         │
│                      "What happened? Who earned?"                            │
│                                                                              │
│  /swarmpool/epochs/    ← sealed epoch snapshots                              │
│  /swarmpool/index/     ← provider balances, state                            │
└──────────────────────────────────────────────────────────────────────────────┘
                                   │
                         read-only ↓
                                   │
┌──────────────────────────────────────────────────────────────────────────────┐
│                           SWARMORB                                           │
│                      "What does it mean?"                                    │
│                                                                              │
│  Observes ledger, interprets trends, shows system health                     │
│  NEVER writes, NEVER overrides                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Components

### `/cli` — Rust CLI (`swarm`)

| Command | Description |
|---------|-------------|
| `swarm init` | Initialize provider (one-time genesis) |
| `swarm watch` | Watch pool for available jobs |
| `swarm submit` | Submit job to pool (client action) |
| `swarm claim` | Claim job with SOLO or PPL mode |
| `swarm prove` | Process job and submit proof |
| `swarm seal` | Seal epoch (Merlin only) |
| `swarm status` | Check network/provider status |
| `swarm withdraw` | Withdraw earnings |
| `swarm epochs` | View epoch history |

### `/landing` — SwarmPool Website

- Astro + Tailwind
- Medical/clinical aesthetic
- Live network stats
- Provider onboarding

## Execution Modes

| Mode | Economics | Best For |
|------|-----------|----------|
| **SOLO** | Winner takes miner pool (75%) | Fast hardware, low competition |
| **PPL** | Proportional by compute_seconds | Steady operators, shared workloads |

```
SOLO pays for being first.
PPL pays for contributing.
```

## Payout Math

```
Job: $0.10 USDC

Distribution:
├── Hive Ops (25%):   $0.025
└── Miner Pool (75%): $0.075

SOLO Example:
  Winner: $0.075

PPL Example (100 compute-seconds total):
  Miner A (40s): $0.030
  Miner B (35s): $0.02625
  Miner C (25s): $0.01875
```

## Quick Start

### For Clients
```bash
cargo install swarm-cli

swarm submit \
  --model queenbee-spine \
  --input ./scan.nii.gz \
  --client clinic.clientswarm.eth
```

### For Compute Providers
```bash
cargo install swarm-cli

# One-time registration
swarm init \
  --provider myprovider.swarmbee.eth \
  --wallet 0x... \
  --gpus "RTX 5090,RTX 5090"

# Watch for jobs
swarm watch --models queenbee-spine,queenbee-chest

# Claim and prove
swarm claim --job bafybei... --mode SOLO
swarm prove --job bafybei... --claim bafybei...
```

## Canon Alignment

| Layer | Declares | Writes? |
|-------|----------|---------|
| Swarmpool | Opportunity | Yes |
| SwarmLedger | Truth | Yes (Merlin) |
| SwarmOrb | Meaning | No |

**Swarmpool declares opportunity.**
**SwarmLedger declares truth.**
**SwarmOrb declares meaning.**

Nothing overlaps. Nothing conflicts.

## Core Properties

- **Visible Opportunity** — Miners see jobs, rewards, and modes before committing
- **Sovereign Participation** — Join once. Idle freely. Leave anytime. No penalties.
- **Proof Over Promises** — If you compute, you prove. If you prove, you earn.
- **Zero Inbound Surface** — No APIs. No open ports. No schedulers.

## Links

- **Landing**: [swarmpool.eth.limo](https://swarmpool.eth.limo)
- **Orb**: [swarmorb.eth.limo](https://swarmorb.eth.limo)
- **Hive**: [swarmhive.eth.limo](https://swarmhive.eth.limo)
- **GitHub**: [github.com/sudohash](https://github.com/sudohash)

---

**Built by SudoHash LLC** | Florida, USA | Solar Powered ☀️

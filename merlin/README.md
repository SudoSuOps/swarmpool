# 🧙‍♂️ Merlin

**SwarmOS Controller · Epoch Manager · Settlement Sealer**

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║   🧙‍♂️ MERLIN — SwarmOS Controller                            ║
║                                                              ║
║   merlin.swarmos.eth                                        ║
║                                                              ║
║   Epoch Clock · Settlement Pen · Truth Sealer               ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

## What Merlin Is

- ⏱ **Epoch Clock** — opens/closes epochs deterministically
- 🧾 **Settlement Pen** — computes payouts and writes final truth
- 🔐 **Verifier** — validates schemas + signatures + proof integrity
- 📦 **Publisher** — publishes signed snapshots to IPFS canonical layout
- 🧠 **Source of Truth for Ledger Seals** — produces `EPOCH_SEAL` snapshots

## What Merlin Is Not

- ❌ Not a scheduler
- ❌ Not a load balancer
- ❌ Not a worker
- ❌ Not an API gateway
- ❌ Not a cloud service
- ❌ Not a centralized authority over compute

**Merlin never instructs miners to run. Miners choose jobs from Swarmpool voluntarily.**

## Installation

```bash
# Clone
git clone https://github.com/sudohash/merlin
cd merlin

# Install dependencies
pip install -r requirements.txt

# Or install as package
pip install -e .
```

## Configuration

Set environment variables:

```bash
# Required
export MERLIN_PRIVATE_KEY="0x..."

# Optional (with defaults)
export MERLIN_IDENTITY="merlin.swarmos.eth"
export SWARM_POOL="swarmpool.eth"
export IPFS_API="http://localhost:5001"
export EPOCH_DURATION_SECONDS="3600"  # 1 hour
export PROVIDER_SHARE="0.75"          # 75%
export NETWORK_OPS_SHARE="0.25"       # 25%
```

## Usage

### Start the Daemon

```bash
merlin run
```

### Check Status

```bash
merlin status
merlin status --json
```

### List Epochs

```bash
merlin epochs
merlin epochs --limit 20
merlin epochs --id epoch-0042
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   CLIENTS submit jobs to /swarmpool/jobs/                          │
│                           │                                         │
│                           ▼                                         │
│   WORKERS watch pool, claim, execute, prove                        │
│                           │                                         │
│                           ▼                                         │
│   PROOFS land in /swarmpool/proofs/                                │
│                           │                                         │
│                           ▼                                         │
│   🧙‍♂️ MERLIN watches /proofs/                                       │
│   ├── Validates schema + signature                                 │
│   ├── Tracks proofs in current epoch                               │
│   ├── When epoch time expires:                                     │
│   │   ├── Computes settlements (75% workers / 25% ops)            │
│   │   ├── Computes merkle root of all proofs                      │
│   │   ├── Signs epoch seal                                         │
│   │   └── Publishes to /swarmledger/epochs/                        │
│   └── Opens new epoch                                              │
│                           │                                         │
│                           ▼                                         │
│   SWARMLEDGER contains immutable truth                             │
│                           │                                         │
│                           ▼                                         │
│   SWARMORB reads and displays                                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Architecture

```
merlin/
├── merlin/
│   ├── __init__.py     # Main daemon
│   ├── cli.py          # CLI entry point
│   ├── config.py       # Configuration
│   ├── crypto.py       # EIP-191 signing, keccak256
│   ├── epoch.py        # Epoch lifecycle management
│   ├── publisher.py    # IPFS publishing
│   ├── schemas.py      # Snapshot validation
│   └── watcher.py      # Proof watching
├── requirements.txt
├── setup.py
└── README.md
```

## Epoch Lifecycle

1. **Open** — Merlin publishes epoch with `status: active`
2. **Collect** — Proofs accumulate in `/swarmpool/proofs/`
3. **Seal** — After duration expires:
   - Gather all valid proofs
   - Compute settlements (75/25 split)
   - Compute merkle root
   - Sign and publish with `status: sealed`
4. **Open** — New epoch begins

## Settlement

```
Job Reward:        $0.10 (typical)
Provider Share:    75% ($0.075)
Network Ops:       25% ($0.025)

No proof = no pay.
Proofs are verified before inclusion.
```

## Systemd Service

```ini
# /etc/systemd/system/merlin.service
[Unit]
Description=Merlin SwarmOS Controller
After=network.target ipfs.service

[Service]
Type=simple
User=merlin
Environment=MERLIN_PRIVATE_KEY=0x...
Environment=IPFS_API=http://localhost:5001
ExecStart=/usr/local/bin/merlin run
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable merlin
sudo systemctl start merlin
sudo journalctl -u merlin -f
```

## Links

- **Pool**: [swarmpool.eth.limo](https://swarmpool.eth.limo)
- **Ledger**: [swarmledger.eth.limo](https://swarmledger.eth.limo)
- **Explorer**: [swarmorb.eth.limo](https://swarmorb.eth.limo)

---

**SwarmPool is not a service. It is a visible field of opportunity where sovereign compute chooses when to work.**

**Merlin is the notary, not the boss.**

MIT License — SudoHash LLC

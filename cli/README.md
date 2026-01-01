# SwarmPool CLI

**Rust CLI for the SwarmPool Decentralized Medical Compute Network**

```
   _____ _       __   ___    ____  __  ___
  / ___/| |     / /  /   |  / __ \/  |/  /
  \__ \ | | /| / /  / /| | / /_/ / /|_/ /
 ___/ / | |/ |/ /  / ___ |/ _, _/ /  / /
/____/  |__/|__/  /_/  |_/_/ |_/_/  /_/
```

## Philosophy

Swarmpool is a **miner-honest** compute pool.

- Jobs are published openly
- Miners decide when and how to work
- Proof settles

No schedulers. No dark room. No token.

## Installation

### From Cargo
```bash
cargo install swarm-cli
```

### From Source
```bash
git clone https://github.com/sudohash/swarm-cli
cd swarm-cli
cargo build --release
./target/release/swarm --help
```

## Commands

### Initialize Provider (One-Time Genesis)
```bash
swarm init \
  --provider myprovider.swarmbee.eth \
  --wallet 0x... \
  --gpus "RTX 5090,RTX 5090"
```

### Watch for Jobs
```bash
swarm watch --models queenbee-spine,queenbee-chest
```

### Submit a Job (Clients)
```bash
swarm submit \
  --model queenbee-spine \
  --input ./scan.nii.gz \
  --client clinic.clientswarm.eth
```

### Claim a Job
```bash
# SOLO: Winner takes full miner pool (75%)
swarm claim --job bafybei... --mode SOLO

# PPL: Proportional payout by compute_seconds
swarm claim --job bafybei... --mode PPL
```

### Submit Proof
```bash
swarm prove --job bafybei... --claim bafybei...
```

### Seal Epoch (Merlin Only)
```bash
swarm seal --epoch epoch-048
```

### Check Status
```bash
# Network status
swarm status

# Provider status
swarm status --provider myprovider.swarmbee.eth
```

### Withdraw Earnings
```bash
swarm withdraw --amount all --provider myprovider.swarmbee.eth
```

### View Epochs
```bash
swarm epochs
swarm epochs --id epoch-047
```

### List Models
```bash
swarm models
```

## Execution Modes

| Mode | Economics | Risk |
|------|-----------|------|
| **SOLO** | Winner takes R × 75% | High variance, higher upside |
| **PPL** | Proportional by compute_seconds | Low variance, steady returns |

Hive always receives R × 25%.

## Payout Math

```
Job: $0.10 USDC
├── Hive Ops (25%):  $0.025
└── Miner Pool (75%): $0.075

SOLO: Winner takes $0.075
PPL:  Split by compute_seconds
      A: 40s → $0.030
      B: 35s → $0.02625
      C: 25s → $0.01875
```

## Configuration

Config file: `~/.config/swarm-cli/config.toml`

```toml
provider_ens = "myprovider.swarmbee.eth"
wallet = "0x..."
gpus = ["RTX 5090", "RTX 5090"]
models = ["queenbee-spine", "queenbee-chest"]
pool = "swarmpool.eth"
ipfs_api = "http://localhost:5001"
```

## Environment Variables

```bash
export SWARM_PRIVATE_KEY="0x..."
export SWARM_PROVIDER_ENS="myprovider.swarmbee.eth"
export SWARM_CLIENT_ENS="clinic.clientswarm.eth"
export SWARM_WALLET="0x..."
```

## IPFS Directory Layout

```
/swarmpool/
├── genesis/     # Provider registrations
├── epochs/      # Sealed epoch snapshots
├── jobs/        # Job submissions
├── claims/      # Job claims (SOLO/PPL)
├── proofs/      # Completed proofs
└── index/       # State & provider balances
```

## Architecture

```
Swarmpool    →  "What work is available?"
SwarmLedger  →  "What happened? Who earned?"
SwarmOrb     →  "What does it mean?"
```

Merlin is a communicator and sealer — not a brain.
Orb is a brain — not a hand.

## Requirements

- IPFS daemon running (`ipfs daemon`)
- Ethereum wallet with private key
- ENS name (for identity)
- NVIDIA GPU with CUDA 12.0+ (for providers)

## Genesis Block

The first sealed epoch. The anchor of truth.

```
Epoch:        epoch-0001 (Bravo)
Status:       SEALED
Merkle Root:  0x4d97224eb3f4e8516305e4cda3011f7e9c9adc3e2553c4d12712b1604312bbe5
CID:          bafkreibg6e7lkkwuz4dmtkebp5ol74dpx5sgp3zwkmzs2diqnfgpes6vnm
Signer:       merlin.swarmos.eth
Sealed:       2026-01-01 17:02:32 UTC
```

## Links

- **Pool Dashboard**: [swarmpool.eth.limo](https://swarmpool.eth.limo)
- **SwarmOrb**: [swarmorb.eth.limo](https://swarmorb.eth.limo)
- **GitHub**: [github.com/sudohash/swarm-cli](https://github.com/sudohash/swarm-cli)
- **Genesis Block**: [ipfs.io/ipfs/bafkreibg6e7lkkwuz4dmtkebp5ol74dpx5sgp3zwkmzs2diqnfgpes6vnm](https://ipfs.io/ipfs/bafkreibg6e7lkkwuz4dmtkebp5ol74dpx5sgp3zwkmzs2diqnfgpes6vnm)

## License

MIT License - SudoHash LLC

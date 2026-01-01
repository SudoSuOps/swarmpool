# CLI Reference

The `swarm` CLI is the primary interface for providers and clients.

---

## Installation

```bash
cargo install swarm-cli
```

---

## Commands

### Provider Commands

| Command | Description |
|---------|-------------|
| `swarm init` | Initialize provider, publish genesis |
| `swarm watch` | Watch pool for available jobs |
| `swarm claim` | Claim a job (SOLO or PPL) |
| `swarm prove` | Execute job and submit proof |
| `swarm status` | Check provider/network status |
| `swarm withdraw` | Withdraw earnings |

### Client Commands

| Command | Description |
|---------|-------------|
| `swarm submit` | Submit inference job |

### Controller Commands

| Command | Description |
|---------|-------------|
| `swarm seal` | Seal epoch (Merlin only) |
| `swarm epochs` | View epoch history |

### Utility Commands

| Command | Description |
|---------|-------------|
| `swarm validate` | Validate snapshot against schema |
| `swarm config` | Show configuration |
| `swarm models` | List available models |

---

## Common Flags

| Flag | Description |
|------|-------------|
| `--pool` | Pool ENS (default: `swarmpool.eth`) |
| `--provider` | Provider ENS |
| `--key` | Private key (or `SWARM_PRIVATE_KEY` env) |
| `--verbose` | Enable verbose output |

---

## Examples

```bash
# Initialize as provider
swarm init --provider miner.swarmbee.eth --wallet 0x...

# Watch for jobs
swarm watch --models queenbee-spine,queenbee-chest

# Claim a job in SOLO mode
swarm claim --job bafybei... --mode SOLO

# Submit proof
swarm prove --job bafybei...

# Check status
swarm status --provider miner.swarmbee.eth
```

---

Swarm provides verifiable inference infrastructure.
Models, data, and outcomes remain the responsibility of their operators.

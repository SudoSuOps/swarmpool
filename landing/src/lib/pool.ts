// Pool data fetching from IPFS index

export interface PoolIndex {
  version: string;
  updated: number;
  cid: string;
  epoch: EpochData;
  stats: PoolStats;
  jobs: JobData[];
  proofs: ProofData[];
  providers: ProviderData[];
}

export interface EpochData {
  id: string;
  name: string;
  start_time: number;
  jobs: number;
  volume: number;
  miner_pool: number;
  hive_ops: number;
  providers: number;
  time_remaining: number;
}

export interface PoolStats {
  total_jobs: number;
  total_volume: number;
  registered_providers: number;
  watching_pool: number;
  epochs_sealed: number;
  current_epoch: string;
}

export interface JobData {
  cid: string;
  job_id: string;
  model: string;
  reward: number;
  age: number;
  status: 'pending' | 'claimed' | 'processing';
  mode: 'SOLO' | 'PPL' | null;
  provider: string | null;
}

export interface ProofData {
  job_cid: string;
  provider: string;
  confidence: number;
  compute_seconds: number;
  earned: number;
  ago: number;
  mode: 'SOLO' | 'PPL';
}

export interface ProviderData {
  rank: number;
  ens: string;
  jobs: number;
  earnings: number;
  status: 'online' | 'busy' | 'offline';
  solo_wins: number;
  ppl_jobs: number;
}

// IPFS gateways to try in order
const GATEWAYS = [
  'https://w3s.link/ipfs',
  'https://dweb.link/ipfs',
  'https://ipfs.io/ipfs',
];

// Pool root CID - update this when index changes
// In production, resolve from swarmpool.eth contenthash
export const POOL_ROOT = 'bafybeihc6aeq2e6ys5uuccdm6bfd3gwhj467mdd3w2rbtkspwk3ylo6o2i';

export async function fetchPoolIndex(): Promise<PoolIndex | null> {
  // First try local index (for same-origin deployment)
  try {
    const localResponse = await fetch('/index/latest.json', {
      signal: AbortSignal.timeout(2000)
    });
    if (localResponse.ok) {
      return await localResponse.json();
    }
  } catch (e) {
    console.warn('Local index not available, trying IPFS gateways');
  }

  // Fall back to IPFS gateways
  for (const gateway of GATEWAYS) {
    try {
      const url = `${gateway}/${POOL_ROOT}/index/latest.json`;
      const response = await fetch(url, {
        signal: AbortSignal.timeout(5000)
      });

      if (response.ok) {
        return await response.json();
      }
    } catch (e) {
      console.warn(`Gateway ${gateway} failed:`, e);
      continue;
    }
  }

  return null;
}

// Format time ago
export function formatAge(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

// Format time remaining
export function formatTimeRemaining(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// Truncate CID for display
export function truncateCid(cid: string): string {
  if (cid.length <= 16) return cid;
  return `${cid.slice(0, 8)}...${cid.slice(-4)}`;
}

// Truncate ENS for display
export function truncateEns(ens: string): string {
  if (ens.length <= 20) return ens;
  return `${ens.slice(0, 10)}...${ens.slice(-6)}`;
}

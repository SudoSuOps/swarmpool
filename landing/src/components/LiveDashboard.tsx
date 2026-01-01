import { useState, useEffect } from 'react';
import type { PoolIndex, JobData, ProofData, ProviderData, EpochData, PoolStats } from '../lib/pool';
import { fetchPoolIndex, formatAge, formatTimeRemaining, truncateCid, truncateEns } from '../lib/pool';

// Fallback mock data
const MOCK_DATA: PoolIndex = {
  version: '1.0.0',
  updated: Date.now(),
  cid: 'bafybei...mock',
  epoch: {
    id: '048',
    name: 'Golf',
    start_time: Date.now() - 3600000,
    jobs: 156,
    volume: 15.60,
    miner_pool: 11.70,
    hive_ops: 3.90,
    providers: 18,
    time_remaining: 2538,
  },
  stats: {
    total_jobs: 12847,
    total_volume: 1284.70,
    registered_providers: 23,
    watching_pool: 18,
    epochs_sealed: 47,
    current_epoch: '048',
  },
  jobs: [
    { cid: 'bafybei...a3f2', job_id: 'j001', model: 'queenbee-spine', reward: 0.10, age: 3, status: 'pending', mode: null, provider: null },
    { cid: 'bafybei...b7c1', job_id: 'j002', model: 'queenbee-chest', reward: 0.10, age: 8, status: 'pending', mode: null, provider: null },
    { cid: 'bafybei...d4e9', job_id: 'j003', model: 'queenbee-spine', reward: 0.10, age: 12, status: 'claimed', mode: 'SOLO', provider: 'bumble70b.swarmbee.eth' },
    { cid: 'bafybei...f2a8', job_id: 'j004', model: 'queenbee-foot', reward: 0.08, age: 15, status: 'claimed', mode: 'PPL', provider: 'miner.alice.eth' },
    { cid: 'bafybei...g9h3', job_id: 'j005', model: 'queenbee-spine', reward: 0.10, age: 22, status: 'processing', mode: 'SOLO', provider: 'gpu-farm.eth' },
  ],
  proofs: [
    { job_cid: 'bafybei...x7z2', provider: 'bumble70b.swarmbee.eth', confidence: 72, compute_seconds: 95, earned: 0.075, ago: 12, mode: 'SOLO' },
    { job_cid: 'bafybei...y3w1', provider: 'miner.alice.eth', confidence: 68, compute_seconds: 102, earned: 0.038, ago: 34, mode: 'PPL' },
    { job_cid: 'bafybei...z9v4', provider: 'miner.bob.eth', confidence: 71, compute_seconds: 88, earned: 0.075, ago: 60, mode: 'SOLO' },
    { job_cid: 'bafybei...a2u8', provider: 'gpu-farm.eth', confidence: 65, compute_seconds: 110, earned: 0.045, ago: 120, mode: 'PPL' },
    { job_cid: 'bafybei...b5t3', provider: 'bumble70b.swarmbee.eth', confidence: 74, compute_seconds: 91, earned: 0.075, ago: 180, mode: 'SOLO' },
  ],
  providers: [
    { rank: 1, ens: 'bumble70b.swarmbee.eth', jobs: 2847, earnings: 213.52, status: 'online', solo_wins: 1842, ppl_jobs: 1005 },
    { rank: 2, ens: 'miner.alice.eth', jobs: 1923, earnings: 144.22, status: 'online', solo_wins: 856, ppl_jobs: 1067 },
    { rank: 3, ens: 'miner.bob.eth', jobs: 1456, earnings: 109.20, status: 'online', solo_wins: 1123, ppl_jobs: 333 },
    { rank: 4, ens: 'gpu-farm.eth', jobs: 1201, earnings: 90.07, status: 'online', solo_wins: 234, ppl_jobs: 967 },
    { rank: 5, ens: 'inference-labs.eth', jobs: 854, earnings: 64.05, status: 'busy', solo_wins: 512, ppl_jobs: 342 },
  ],
};

const modelLabels: Record<string, string> = {
  'queenbee-spine': 'Spine MRI',
  'queenbee-chest': 'Chest CT',
  'queenbee-foot': 'Foot/Ankle',
  'queenbee-brain': 'Brain MRI',
  'queenbee-knee': 'Knee MRI',
};

const statusColors = {
  pending: { bg: 'bg-amber-500/10', text: 'text-amber-400' },
  claimed: { bg: 'bg-blue-500/10', text: 'text-blue-400' },
  processing: { bg: 'bg-yellow-500/10', text: 'text-yellow-400' },
};

const modeColors = {
  SOLO: { bg: 'bg-amber-500/10', text: 'text-amber-400' },
  PPL: { bg: 'bg-blue-500/10', text: 'text-blue-400' },
};

const providerStatusColors = {
  online: { dot: 'bg-emerald-400', text: 'text-emerald-400' },
  busy: { dot: 'bg-amber-400', text: 'text-amber-400' },
  offline: { dot: 'bg-red-400', text: 'text-red-400' },
};

export default function LiveDashboard() {
  const [data, setData] = useState<PoolIndex>(MOCK_DATA);
  const [isLive, setIsLive] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadData() {
      try {
        const poolData = await fetchPoolIndex();
        if (poolData) {
          setData(poolData);
          setIsLive(true);
          setLastUpdate(new Date());
          setError(null);
        } else {
          setError('Using cached data');
        }
      } catch (e) {
        setError('Using cached data');
      }
    }

    loadData();

    // Refresh every 30 seconds
    const interval = setInterval(loadData, 30000);
    return () => clearInterval(interval);
  }, []);

  const { epoch, stats, jobs, proofs, providers } = data;

  return (
    <div className="space-y-8">
      {/* Status Badge */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full ${isLive ? 'bg-emerald-400 animate-pulse' : 'bg-gray-500'}`}></span>
          <span className="text-sm text-gray-400">
            {isLive ? 'Live from IPFS' : 'Cached Data'}
          </span>
        </div>
        {lastUpdate && (
          <span className="text-xs text-gray-500 font-mono">
            Updated {lastUpdate.toLocaleTimeString()}
          </span>
        )}
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard label="Total Jobs" value={stats.total_jobs.toLocaleString()} change={`+${Math.floor(stats.total_jobs * 0.012)} today`} />
        <StatCard label="Volume (USDC)" value={`$${stats.total_volume.toFixed(2)}`} change={`+$${(stats.total_volume * 0.012).toFixed(2)} today`} />
        <StatCard label="Registered Providers" value={stats.registered_providers.toString()} change={`${stats.watching_pool} watching pool`} highlight />
        <StatCard label="Epochs Sealed" value={stats.epochs_sealed.toString()} change={`#${stats.current_epoch} in progress`} />
      </div>

      {/* Current Epoch */}
      <div className="bg-slate-800/50 border border-slate-700 rounded-xl p-4">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
              <span className="text-sm font-medium text-gray-400">CURRENT EPOCH</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-lg font-bold text-white">#{epoch.id}</span>
              <span className="px-2 py-0.5 bg-blue-500/20 text-blue-400 text-sm font-medium rounded">{epoch.name}</span>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-6 text-sm">
            <div className="flex items-center gap-2">
              <span className="text-gray-500">Jobs:</span>
              <span className="font-semibold text-white">{epoch.jobs}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-gray-500">Volume:</span>
              <span className="font-semibold text-white">${epoch.volume.toFixed(2)}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-gray-500">Miner Pool:</span>
              <span className="font-semibold text-emerald-400">${epoch.miner_pool.toFixed(2)}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-gray-500">Hive Ops:</span>
              <span className="font-semibold text-blue-400">${epoch.hive_ops.toFixed(2)}</span>
            </div>
            <div className="flex items-center gap-2 px-3 py-1.5 bg-slate-700/50 rounded-lg">
              <svg className="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
              <span className="font-mono text-white">{formatTimeRemaining(epoch.time_remaining)}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Job Queue */}
        <div className="lg:col-span-2 bg-slate-800/50 border border-slate-700 rounded-xl">
          <div className="px-6 py-4 border-b border-slate-700 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <h3 className="font-semibold text-white">Job Queue</h3>
              <span className="px-2 py-0.5 bg-slate-700 text-gray-400 text-xs rounded-full">
                {jobs.filter(j => j.status === 'pending').length} pending
              </span>
            </div>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-left text-xs text-gray-500 uppercase tracking-wider">
                  <th className="px-6 py-3 font-medium">Job ID</th>
                  <th className="px-6 py-3 font-medium">Model</th>
                  <th className="px-6 py-3 font-medium">Reward</th>
                  <th className="px-6 py-3 font-medium">Mode</th>
                  <th className="px-6 py-3 font-medium">Age</th>
                  <th className="px-6 py-3 font-medium">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700">
                {jobs.map((job, i) => (
                  <tr key={i} className="hover:bg-slate-700/30 transition-colors">
                    <td className="px-6 py-4">
                      <span className="font-mono text-sm text-white">{truncateCid(job.cid)}</span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-gray-400">{modelLabels[job.model] || job.model}</span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm font-medium text-white">${job.reward.toFixed(2)}</span>
                    </td>
                    <td className="px-6 py-4">
                      {job.mode ? (
                        <span className={`px-2 py-0.5 text-xs font-bold rounded ${modeColors[job.mode].bg} ${modeColors[job.mode].text}`}>
                          {job.mode}
                        </span>
                      ) : (
                        <span className="text-sm text-gray-600">-</span>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-gray-500">{formatAge(job.age)}</span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <span className={`px-2 py-1 text-xs font-medium rounded ${statusColors[job.status].bg} ${statusColors[job.status].text}`}>
                          {job.status.charAt(0).toUpperCase() + job.status.slice(1)}
                        </span>
                        {job.provider && (
                          <span className="text-xs text-gray-500 truncate max-w-[100px]" title={job.provider}>
                            {truncateEns(job.provider)}
                          </span>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Recent Proofs */}
        <div className="lg:col-span-1 bg-slate-800/50 border border-slate-700 rounded-xl h-fit">
          <div className="px-6 py-4 border-b border-slate-700 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <h3 className="font-semibold text-white">Recent Completions</h3>
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            </div>
          </div>

          <div className="divide-y divide-slate-700">
            {proofs.map((proof, i) => (
              <div key={i} className="px-6 py-4 hover:bg-slate-700/30 transition-colors">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <svg className="w-4 h-4 text-emerald-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"/>
                      </svg>
                      <span className="font-mono text-sm text-white truncate">{truncateCid(proof.job_cid)}</span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <span className="truncate" title={proof.provider}>{truncateEns(proof.provider)}</span>
                      <span>•</span>
                      <span>{formatAge(proof.ago)}</span>
                    </div>
                  </div>
                  <div className="text-right flex-shrink-0">
                    <div className={`text-sm font-semibold ${proof.confidence >= 70 ? 'text-emerald-400' : proof.confidence >= 60 ? 'text-amber-400' : 'text-red-400'}`}>
                      {proof.confidence}%
                    </div>
                    <div className="text-xs text-gray-500">{proof.compute_seconds}s</div>
                  </div>
                </div>
                <div className="mt-2 flex items-center gap-2">
                  <span className={`px-1.5 py-0.5 text-xs font-bold rounded ${modeColors[proof.mode].bg} ${modeColors[proof.mode].text}`}>
                    {proof.mode}
                  </span>
                  <span className="text-xs text-emerald-400">+${proof.earned.toFixed(3)}</span>
                  <span className="text-xs text-gray-500">earned</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Provider Highlights */}
      <div className="bg-slate-800/50 border border-slate-700 rounded-xl">
        <div className="px-6 py-4 border-b border-slate-700 flex items-center justify-between">
          <h3 className="font-semibold text-white">Top Compute Providers</h3>
          <a href="/providers" className="text-sm text-blue-400 hover:text-blue-300 transition-colors">
            View Leaderboard →
          </a>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 divide-y md:divide-y-0 md:divide-x divide-slate-700">
          {providers.map((provider, i) => {
            const status = providerStatusColors[provider.status];
            return (
              <div key={i} className={`p-6 ${i === 0 ? 'bg-gradient-to-br from-amber-500/5 to-transparent' : ''}`}>
                <div className="flex items-center justify-between mb-4">
                  <div className={`w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold ${i === 0 ? 'bg-amber-500/20 text-amber-400' : 'bg-slate-700 text-gray-400'}`}>
                    {i === 0 ? '👑' : `#${provider.rank}`}
                  </div>
                  <div className="flex items-center gap-1.5">
                    <span className={`w-2 h-2 rounded-full ${status.dot}`}></span>
                    <span className={`text-xs ${status.text}`}>{provider.status}</span>
                  </div>
                </div>

                <div className="mb-4">
                  <div className="font-mono text-sm text-white truncate" title={provider.ens}>
                    {truncateEns(provider.ens)}
                  </div>
                </div>

                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-500">Jobs</span>
                    <span className="font-medium text-white">{provider.jobs.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Earnings</span>
                    <span className="font-medium text-emerald-400">${provider.earnings.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Mode Split</span>
                    <span className="font-medium">
                      <span className="text-amber-400">{provider.solo_wins}</span>
                      <span className="text-gray-600">/</span>
                      <span className="text-blue-400">{provider.ppl_jobs}</span>
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* CID Footer */}
      <div className="text-center text-xs text-gray-600">
        <span className="font-mono">CID: {truncateCid(data.cid)}</span>
        {isLive && <span className="ml-2 text-emerald-500">● synced</span>}
      </div>
    </div>
  );
}

function StatCard({ label, value, change, highlight = false }: { label: string; value: string; change: string; highlight?: boolean }) {
  return (
    <div className={`p-6 rounded-xl border transition-all ${highlight ? 'bg-gradient-to-br from-blue-500/10 to-slate-800/50 border-blue-500/30' : 'bg-slate-800/50 border-slate-700 hover:border-slate-600'}`}>
      <div className="flex items-start justify-between mb-4">
        <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${highlight ? 'bg-blue-500/20' : 'bg-slate-700'}`}>
          <svg className={`w-5 h-5 ${highlight ? 'text-blue-400' : 'text-gray-400'}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
          </svg>
        </div>
        {highlight && (
          <span className="flex items-center gap-1.5 text-xs text-emerald-400">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
            Live
          </span>
        )}
      </div>

      <div className="text-2xl font-bold text-white mb-1">{value}</div>
      <div className="text-sm text-gray-400">{label}</div>

      <div className="mt-3 pt-3 border-t border-slate-700">
        <span className="text-xs text-gray-500">{change}</span>
      </div>
    </div>
  );
}

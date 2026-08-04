export default function GovernancePage() {
  return (
    <div className="max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold text-white mb-1">Governance</h1>
      <p className="text-sm text-gray-500 mb-8">
        Market proposals and protocol governance decisions.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <section>
          <h2 className="text-lg font-semibold mb-3">Market Proposals</h2>
          <div className="card text-center py-10 text-gray-500 text-sm">
            <p className="text-3xl mb-2">📋</p>
            <p>No pending proposals.</p>
            <a href="/propose" className="text-blue-400 underline mt-1 block">Propose a market</a>
          </div>
        </section>

        <section>
          <h2 className="text-lg font-semibold mb-3">Protocol Proposals</h2>
          <div className="card text-center py-10 text-gray-500 text-sm">
            <p className="text-3xl mb-2">🗳️</p>
            <p>No active governance proposals.</p>
          </div>
        </section>
      </div>

      <div className="mt-8 card bg-blue-950/30 border-blue-900/40">
        <h3 className="text-sm font-semibold text-blue-300 mb-1">Maintainer Council</h3>
        <p className="text-xs text-gray-400">
          The StellarMarket maintainer council reviews proposals and governs the protocol.
          See <a href="https://github.com/stellarmarket/stellarmarket/blob/main/GOVERNANCE.md" className="underline text-blue-400" target="_blank" rel="noreferrer">GOVERNANCE.md</a> for the full process.
        </p>
      </div>
    </div>
  );
}
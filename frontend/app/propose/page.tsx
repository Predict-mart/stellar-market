'use client';

import { useState, type FormEvent } from 'react';
import { useWallet } from '@/lib/wallet';

const CATEGORIES = ['Elections', 'Sports', 'Finance', 'Tech', 'Global', 'Other'];

export default function ProposePage() {
  const { isConnected, connect } = useWallet();

  const [question, setQuestion]   = useState('');
  const [outcomes, setOutcomes]   = useState(['YES', 'NO']);
  const [category, setCategory]   = useState('Other');
  const [resDate, setResDate]     = useState('');
  const [criteria, setCriteria]   = useState('');
  const [oracleSource, setOracle] = useState('');
  const [status, setStatus]       = useState<'idle' | 'submitting' | 'success' | 'error'>('idle');
  const [proposalId, setProposalId] = useState<number | null>(null);
  const [error, setError]         = useState<string | null>(null);

  const addOutcome = () => {
    if (outcomes.length < 10) setOutcomes([...outcomes, '']);
  };
  const updateOutcome = (i: number, val: string) => {
    const next = [...outcomes];
    next[i] = val;
    setOutcomes(next);
  };
  const removeOutcome = (i: number) => {
    if (outcomes.length <= 2) return;
    setOutcomes(outcomes.filter((_, idx) => idx !== i));
  };

  const minResDate = new Date(Date.now() + 24 * 3600 * 1000).toISOString().slice(0, 16);

  const valid =
    question.trim().length >= 10 &&
    outcomes.filter(Boolean).length >= 2 &&
    resDate.length > 0 &&
    criteria.trim().length >= 50;

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!valid || !isConnected) return;

    setStatus('submitting');
    setError(null);

    try {
      // TODO: build real Soroban XDR calling MarketFactory.propose_market()
      // See issue #28: Build market proposal submission form
      await new Promise((r) => setTimeout(r, 1000)); // simulate
      setProposalId(Math.floor(Math.random() * 1000) + 1);
      setStatus('success');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Submission failed');
      setStatus('error');
    }
  }

  if (status === 'success' && proposalId) {
    return (
      <div className="max-w-xl mx-auto text-center py-12">
        <div className="text-5xl mb-4">🎉</div>
        <h1 className="text-2xl font-bold text-white mb-2">Proposal Submitted!</h1>
        <p className="text-gray-400 mb-1">Your proposal ID is <strong className="text-white">#{proposalId}</strong></p>
        <p className="text-sm text-gray-500 mb-6">
          The maintainer council will review it within 7 days.
        </p>
        <a href="/governance" className="btn-primary mr-3">Track on Governance</a>
        <button onClick={() => { setStatus('idle'); setQuestion(''); setProposalId(null); }} className="btn-secondary">
          Propose Another
        </button>
      </div>
    );
  }

  return (
    <div className="max-w-xl mx-auto">
      <h1 className="text-2xl font-bold text-white mb-1">Propose a Market</h1>
      <p className="text-sm text-gray-500 mb-6">
        Submit a prediction market for council review. Markets with clear, verifiable resolution criteria are approved fastest.
      </p>

      {!isConnected ? (
        <div className="card text-center space-y-3">
          <p className="text-gray-400 text-sm">Connect your wallet to submit a proposal</p>
          <button onClick={connect} className="btn-primary">Connect Wallet</button>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-5">
          {/* Question */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Question <span className="text-gray-500 text-xs">{question.length}/280</span>
            </label>
            <textarea
              value={question}
              onChange={(e) => setQuestion(e.target.value.slice(0, 280))}
              rows={2}
              placeholder="Will [event] happen by [date]?"
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 resize-none"
            />
          </div>

          {/* Outcomes */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Outcomes</label>
            <div className="space-y-2">
              {outcomes.map((o, i) => (
                <div key={i} className="flex gap-2">
                  <input
                    value={o}
                    onChange={(e) => updateOutcome(i, e.target.value)}
                    placeholder={`Outcome ${i + 1}`}
                    className="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                  {outcomes.length > 2 && (
                    <button type="button" onClick={() => removeOutcome(i)} className="text-gray-500 hover:text-red-400 px-2">✕</button>
                  )}
                </div>
              ))}
            </div>
            {outcomes.length < 10 && (
              <button type="button" onClick={addOutcome} className="text-xs text-blue-400 hover:underline mt-2">
                + Add outcome
              </button>
            )}
          </div>

          {/* Category + Resolution date */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Category</label>
              <select
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
              >
                {CATEGORIES.map((c) => <option key={c}>{c}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Resolution Date</label>
              <input
                type="datetime-local"
                value={resDate}
                min={minResDate}
                onChange={(e) => setResDate(e.target.value)}
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Resolution criteria */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Resolution Criteria <span className="text-gray-500 text-xs">(min 50 chars · {criteria.length})</span>
            </label>
            <textarea
              value={criteria}
              onChange={(e) => setCriteria(e.target.value)}
              rows={3}
              placeholder="Describe exactly how and when this market will be resolved. Reference a specific public data source..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 resize-none"
            />
          </div>

          {/* Oracle source */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Oracle / Data Source URL</label>
            <input
              type="url"
              value={oracleSource}
              onChange={(e) => setOracle(e.target.value)}
              placeholder="https://..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>

          {error && <p className="text-red-400 text-sm">{error}</p>}

          <button
            type="submit"
            disabled={!valid || status === 'submitting'}
            className="btn-primary w-full"
          >
            {status === 'submitting' ? 'Submitting...' : 'Submit Proposal'}
          </button>
        </form>
      )}
    </div>
  );
}
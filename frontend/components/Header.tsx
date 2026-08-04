'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { WalletButton } from './WalletButton';
import { clsx } from 'clsx';

const NAV_LINKS = [
  { href: '/',           label: 'Markets' },
  { href: '/portfolio',  label: 'Portfolio' },
  { href: '/propose',    label: 'Propose' },
  { href: '/governance', label: 'Governance' },
];

export function Header() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-50 bg-gray-950/95 backdrop-blur border-b border-gray-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <Link href="/" className="flex items-center gap-2 font-bold text-lg">
            <span className="text-2xl">⭐</span>
            <span className="text-white">StellarMarket</span>
          </Link>

          {/* Nav */}
          <nav className="hidden md:flex items-center gap-1">
            {NAV_LINKS.map(({ href, label }) => (
              <Link
                key={href}
                href={href}
                className={clsx(
                  'px-3 py-2 rounded-md text-sm font-medium transition-colors',
                  pathname === href
                    ? 'bg-gray-800 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-gray-800/50',
                )}
              >
                {label}
              </Link>
            ))}
          </nav>

          {/* Wallet */}
          <WalletButton />
        </div>
      </div>
    </header>
  );
}
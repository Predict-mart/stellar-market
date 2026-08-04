import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { WalletProvider } from '@/lib/wallet';
import { Header } from '@/components/Header';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'StellarMarket — Predict. Trade. Settle. Trustlessly.',
  description: 'Decentralized prediction markets on the Stellar blockchain.',
  openGraph: {
    title: 'StellarMarket',
    description: 'Decentralized prediction markets on Stellar.',
    type: 'website',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={`${inter.className} bg-gray-950 text-gray-100 min-h-screen`}>
        <WalletProvider>
          <Header />
          <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            {children}
          </main>
          <footer className="border-t border-gray-800 mt-16 py-8 text-center text-sm text-gray-500">
            <p>StellarMarket — Open Source · Apache 2.0 ·{' '}
              <a href="https://github.com/stellarmarket/stellarmarket" className="underline hover:text-gray-300">
                GitHub
              </a>
            </p>
          </footer>
        </WalletProvider>
      </body>
    </html>
  );
}
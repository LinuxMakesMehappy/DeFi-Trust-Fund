import React from 'react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useWallet } from '@solana/wallet-adapter-react';
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';
import { motion } from 'framer-motion';

// Components
import Dashboard from './components/Dashboard';
import DepositForm from './components/DepositForm';
import UserStats from './components/UserStats';
import TierDisplay from './components/TierDisplay';
import YieldCalculator from './components/YieldCalculator';

// Styles
import '@solana/wallet-adapter-react-ui/styles.css';
import './App.css';

const network = WalletAdapterNetwork.Devnet;
const endpoint = clusterApiUrl(network);
const wallets = [new PhantomWalletAdapter()];

function App() {
  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <div className="min-h-screen bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900">
          <Header />
          <MainContent />
        </div>
      </WalletProvider>
    </ConnectionProvider>
  );
}

function Header() {
  return (
    <motion.header 
      className="bg-black/20 backdrop-blur-lg border-b border-white/10"
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      transition={{ duration: 0.8 }}
    >
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center py-4">
          <div className="flex items-center space-x-4">
            <div className="w-10 h-10 bg-gradient-to-r from-purple-500 to-blue-500 rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-lg">DTF</span>
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">DeFi Trust Fund</h1>
              <p className="text-gray-300 text-sm">Self-Perpetuating Proof of Loyalty</p>
            </div>
          </div>
          <WalletMultiButton className="bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-all duration-200" />
        </div>
      </div>
    </motion.header>
  );
}

function MainContent() {
  const { connected } = useWallet();

  if (!connected) {
    return (
      <motion.div 
        className="flex items-center justify-center min-h-[80vh]"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <div className="text-center">
          <div className="w-24 h-24 bg-gradient-to-r from-purple-500 to-blue-500 rounded-full flex items-center justify-center mx-auto mb-6">
            <svg className="w-12 h-12 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h2 className="text-3xl font-bold text-white mb-4">Connect Your Wallet</h2>
          <p className="text-gray-300 text-lg mb-8 max-w-md mx-auto">
            Connect your Solana wallet to start earning yields with our revolutionary day-based commitment system.
          </p>
          <div className="space-y-4">
            <div className="bg-white/10 backdrop-blur-lg rounded-lg p-6 max-w-md mx-auto">
              <h3 className="text-white font-semibold mb-3">🚀 Key Features</h3>
              <ul className="text-gray-300 text-sm space-y-2">
                <li>• Day-based commitment system with penalties</li>
                <li>• Tiered yields based on loyalty (11.64% - 23.27% APY)</li>
                <li>• Self-perpetuating fund growth</li>
                <li>• NFT-based authorization</li>
                <li>• Auto-reinvest options</li>
              </ul>
            </div>
          </div>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div 
      className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Main Dashboard */}
        <div className="lg:col-span-2 space-y-8">
          <Dashboard />
          <DepositForm />
          <YieldCalculator />
        </div>
        
        {/* Sidebar */}
        <div className="space-y-8">
          <UserStats />
          <TierDisplay />
        </div>
      </div>
    </motion.div>
  );
}

export default App;

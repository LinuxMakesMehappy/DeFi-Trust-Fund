import React from 'react';
import { motion } from 'framer-motion';
import { useWallet } from '@solana/wallet-adapter-react';
import { User, Clock, Award, TrendingUp, Shield, Calendar } from 'lucide-react';

interface UserStatsProps {}

const UserStats: React.FC<UserStatsProps> = () => {
  const { publicKey } = useWallet();

  // Mock user data - in real implementation, this would come from the smart contract
  const userData = {
    depositAmount: 25.5, // SOL
    committedDays: 30,
    daysStaked: 12,
    lifetimeStakedDays: 45,
    tier: 2,
    autoReinvestPercentage: 20,
    estimatedApy: 17.45,
    totalEarned: 3.2, // SOL
    nextRebalance: '2024-02-15',
    hasStakeNft: true,
    hasTierNft: true,
  };

  const tierInfo = {
    1: { name: 'Bronze', color: 'from-amber-500 to-orange-500', apy: 11.64 },
    2: { name: 'Silver', color: 'from-gray-400 to-gray-600', apy: 17.45 },
    3: { name: 'Gold', color: 'from-yellow-400 to-yellow-600', apy: 23.27 },
  };

  const currentTier = tierInfo[userData.tier as keyof typeof tierInfo];

  const formatSOL = (amount: number) => {
    return `${amount.toFixed(2)} SOL`;
  };

  const calculateProgress = (current: number, total: number) => {
    return Math.min((current / total) * 100, 100);
  };

  const commitmentProgress = calculateProgress(userData.daysStaked, userData.committedDays);

  return (
    <motion.div
      className="space-y-6"
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5 }}
    >
      {/* User Profile */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <div className="flex items-center space-x-4 mb-6">
          <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-blue-500 rounded-full flex items-center justify-center">
            <User className="w-6 h-6 text-white" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white">Your Profile</h3>
            <p className="text-sm text-gray-400">
              {publicKey ? `${publicKey.toString().slice(0, 4)}...${publicKey.toString().slice(-4)}` : 'Not connected'}
            </p>
          </div>
        </div>

        {/* Current Tier */}
        <div className="mb-6">
          <div className={`w-full h-16 bg-gradient-to-r ${currentTier.color} rounded-lg flex items-center justify-center mb-3`}>
            <div className="text-center">
              <p className="text-white font-bold text-lg">{currentTier.name} Tier</p>
              <p className="text-white/80 text-sm">{currentTier.apy}% APY</p>
            </div>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Current APY</span>
            <span className="text-white font-semibold">{userData.estimatedApy}%</span>
          </div>
        </div>

        {/* NFT Status */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Shield className={`w-4 h-4 ${userData.hasStakeNft ? 'text-green-400' : 'text-red-400'}`} />
              <span className="text-sm text-gray-300">Stake Pass NFT</span>
            </div>
            <span className={`text-sm font-medium ${userData.hasStakeNft ? 'text-green-400' : 'text-red-400'}`}>
              {userData.hasStakeNft ? 'Active' : 'Inactive'}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Award className={`w-4 h-4 ${userData.hasTierNft ? 'text-green-400' : 'text-red-400'}`} />
              <span className="text-sm text-gray-300">Tier Badge NFT</span>
            </div>
            <span className={`text-sm font-medium ${userData.hasTierNft ? 'text-green-400' : 'text-red-400'}`}>
              {userData.hasTierNft ? 'Active' : 'Inactive'}
            </span>
          </div>
        </div>
      </div>

      {/* Commitment Progress */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <div className="flex items-center space-x-2 mb-4">
          <Clock className="w-5 h-5 text-purple-400" />
          <h3 className="text-lg font-semibold text-white">Commitment Progress</h3>
        </div>
        
        <div className="mb-4">
          <div className="flex justify-between text-sm mb-2">
            <span className="text-gray-400">Current Period</span>
            <span className="text-white">{userData.daysStaked} / {userData.committedDays} days</span>
          </div>
          <div className="w-full bg-white/10 rounded-full h-2">
            <div 
              className="bg-gradient-to-r from-purple-500 to-blue-500 h-2 rounded-full transition-all duration-300"
              style={{ width: `${commitmentProgress}%` }}
            ></div>
          </div>
        </div>

        <div className="space-y-3 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-400">Lifetime Staked</span>
            <span className="text-white font-medium">{userData.lifetimeStakedDays} days</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Auto-Reinvest</span>
            <span className="text-white font-medium">{userData.autoReinvestPercentage}%</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Next Rebalance</span>
            <span className="text-white font-medium">{userData.nextRebalance}</span>
          </div>
        </div>
      </div>

      {/* Earnings Summary */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <div className="flex items-center space-x-2 mb-4">
          <TrendingUp className="w-5 h-5 text-green-400" />
          <h3 className="text-lg font-semibold text-white">Earnings Summary</h3>
        </div>
        
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <span className="text-gray-400">Current Deposit</span>
            <span className="text-white font-semibold">{formatSOL(userData.depositAmount)}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-gray-400">Total Earned</span>
            <span className="text-green-400 font-semibold">+{formatSOL(userData.totalEarned)}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-gray-400">Projected Annual</span>
            <span className="text-blue-400 font-semibold">+{formatSOL(userData.depositAmount * (userData.estimatedApy / 100))}</span>
          </div>
        </div>

        <div className="mt-6 p-4 bg-gradient-to-r from-green-500/10 to-blue-500/10 border border-green-500/20 rounded-lg">
          <div className="flex items-center space-x-2 mb-2">
            <Calendar className="w-4 h-4 text-green-400" />
            <span className="text-sm font-medium text-green-300">Loyalty Multiplier</span>
          </div>
          <p className="text-sm text-gray-300">
            Your {userData.lifetimeStakedDays} lifetime days provide a loyalty boost to your yields.
          </p>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <h3 className="text-lg font-semibold text-white mb-4">Quick Actions</h3>
        <div className="space-y-3">
          <button className="w-full bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-all duration-200">
            Claim Yields
          </button>
          <button className="w-full bg-white/10 hover:bg-white/20 text-white font-medium py-2 px-4 rounded-lg border border-white/20 transition-all duration-200">
            Adjust Auto-Reinvest
          </button>
          <button className="w-full bg-white/10 hover:bg-white/20 text-white font-medium py-2 px-4 rounded-lg border border-white/20 transition-all duration-200">
            View Transaction History
          </button>
        </div>
      </div>
    </motion.div>
  );
};

export default UserStats;

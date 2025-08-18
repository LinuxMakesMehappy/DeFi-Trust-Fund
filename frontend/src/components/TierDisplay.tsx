import React from 'react';
import { motion } from 'framer-motion';
import { Crown, Star, Zap, Target, Users, TrendingUp } from 'lucide-react';

interface TierDisplayProps {}

const TierDisplay: React.FC<TierDisplayProps> = () => {
  const tiers = [
    {
      id: 1,
      name: 'Bronze Tier',
      color: 'from-amber-500 to-orange-500',
      apy: 11.64,
      requirements: {
        score: '1-30',
        users: '70%',
        description: 'Entry level tier for new users'
      },
      benefits: [
        '20% protocol reinvestment',
        'Basic NFT rewards',
        'Standard support'
      ],
      icon: Star
    },
    {
      id: 2,
      name: 'Silver Tier',
      color: 'from-gray-400 to-gray-600',
      apy: 17.45,
      requirements: {
        score: '31-60',
        users: '20%',
        description: 'Mid-tier for active participants'
      },
      benefits: [
        '20% protocol reinvestment',
        'Enhanced NFT rewards',
        'Priority support',
        'Loyalty multiplier boost'
      ],
      icon: Crown
    },
    {
      id: 3,
      name: 'Gold Tier',
      color: 'from-yellow-400 to-yellow-600',
      apy: 23.27,
      requirements: {
        score: '61-100',
        users: '10%',
        description: 'Elite tier for top performers'
      },
      benefits: [
        '0% protocol reinvestment',
        'Premium NFT rewards',
        'VIP support',
        'Maximum loyalty multiplier',
        'Exclusive governance rights'
      ],
      icon: Crown
    }
  ];

  const currentUserTier = 2; // Mock data

  return (
    <motion.div
      className="space-y-6"
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5 }}
    >
      {/* Tier System Overview */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <div className="flex items-center space-x-2 mb-6">
          <Crown className="w-6 h-6 text-yellow-400" />
          <h3 className="text-lg font-semibold text-white">Tier System</h3>
        </div>
        
        <div className="space-y-4">
          {tiers.map((tier, index) => (
            <motion.div
              key={tier.id}
              className={`relative p-4 rounded-lg border transition-all duration-200 ${
                currentUserTier === tier.id
                  ? 'bg-gradient-to-r from-purple-500/20 to-blue-500/20 border-purple-500/40'
                  : 'bg-white/5 border-white/20 hover:bg-white/10'
              }`}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: index * 0.1 }}
            >
              {currentUserTier === tier.id && (
                <div className="absolute -top-2 -right-2 w-6 h-6 bg-gradient-to-r from-purple-500 to-blue-500 rounded-full flex items-center justify-center">
                  <div className="w-2 h-2 bg-white rounded-full"></div>
                </div>
              )}
              
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center space-x-3">
                  <div className={`w-10 h-10 bg-gradient-to-r ${tier.color} rounded-lg flex items-center justify-center`}>
                    <tier.icon className="w-5 h-5 text-white" />
                  </div>
                  <div>
                    <h4 className="text-white font-semibold">{tier.name}</h4>
                    <p className="text-sm text-gray-400">{tier.requirements.description}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-2xl font-bold text-white">{tier.apy}%</p>
                  <p className="text-sm text-gray-400">APY</p>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-gray-400">Score Range</p>
                  <p className="text-white font-medium">{tier.requirements.score}</p>
                </div>
                <div>
                  <p className="text-gray-400">User Share</p>
                  <p className="text-white font-medium">{tier.requirements.users}</p>
                </div>
              </div>

              <div className="mt-3">
                <p className="text-xs text-gray-400 mb-2">Benefits:</p>
                <ul className="space-y-1">
                  {tier.benefits.map((benefit, benefitIndex) => (
                    <li key={benefitIndex} className="text-xs text-gray-300 flex items-center space-x-2">
                      <div className="w-1 h-1 bg-green-400 rounded-full"></div>
                      <span>{benefit}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </motion.div>
          ))}
        </div>
      </div>

      {/* Tier Requirements */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <div className="flex items-center space-x-2 mb-4">
          <Target className="w-5 h-5 text-purple-400" />
          <h3 className="text-lg font-semibold text-white">How to Advance</h3>
        </div>
        
        <div className="space-y-4">
          <div className="p-4 bg-gradient-to-r from-purple-500/10 to-blue-500/10 border border-purple-500/20 rounded-lg">
            <h4 className="text-white font-medium mb-2">Score Calculation</h4>
            <p className="text-sm text-gray-300 mb-3">
              Your tier score is calculated based on:
            </p>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-400">Deposit Amount:</span>
                <span className="text-white">5x multiplier</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Days Staked:</span>
                <span className="text-white">5x multiplier</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Loyalty Multiplier:</span>
                <span className="text-white">Up to 2x boost</span>
              </div>
            </div>
          </div>

          <div className="p-4 bg-gradient-to-r from-green-500/10 to-emerald-500/10 border border-green-500/20 rounded-lg">
            <h4 className="text-white font-medium mb-2">Loyalty Multiplier</h4>
            <p className="text-sm text-gray-300">
              Your loyalty multiplier increases by 0.2x for each year staked, up to a maximum of 2x after 5 years.
            </p>
          </div>

          <div className="p-4 bg-gradient-to-r from-blue-500/10 to-cyan-500/10 border border-blue-500/20 rounded-lg">
            <h4 className="text-white font-medium mb-2">Monthly Rebalancing</h4>
            <p className="text-sm text-gray-300">
              Tiers are recalculated monthly based on updated scores. Top performers advance, while inactive users may drop tiers.
            </p>
          </div>
        </div>
      </div>

      {/* Current Progress */}
      <div className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20">
        <div className="flex items-center space-x-2 mb-4">
          <TrendingUp className="w-5 h-5 text-green-400" />
          <h3 className="text-lg font-semibold text-white">Your Progress</h3>
        </div>
        
        <div className="space-y-4">
          <div>
            <div className="flex justify-between text-sm mb-2">
              <span className="text-gray-400">Current Score</span>
              <span className="text-white">1,247</span>
            </div>
            <div className="w-full bg-white/10 rounded-full h-2">
              <div className="bg-gradient-to-r from-purple-500 to-blue-500 h-2 rounded-full" style={{ width: '62%' }}></div>
            </div>
            <p className="text-xs text-gray-400 mt-1">Silver Tier (62% to Gold)</p>
          </div>

          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-gray-400">Days to Gold</p>
              <p className="text-white font-medium">~45 days</p>
            </div>
            <div>
              <p className="text-gray-400">Score Needed</p>
              <p className="text-white font-medium">+753 points</p>
            </div>
          </div>

          <div className="p-3 bg-gradient-to-r from-yellow-500/10 to-orange-500/10 border border-yellow-500/20 rounded-lg">
            <p className="text-sm text-yellow-200">
              💡 <strong>Tip:</strong> Increase your deposit or extend your commitment to advance to Gold Tier faster!
            </p>
          </div>
        </div>
      </div>
    </motion.div>
  );
};

export default TierDisplay;

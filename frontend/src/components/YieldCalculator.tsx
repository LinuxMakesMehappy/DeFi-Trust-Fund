import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { Calculator, TrendingUp, Clock, AlertTriangle, Info } from 'lucide-react';

interface YieldCalculatorProps {}

const YieldCalculator: React.FC<YieldCalculatorProps> = () => {
  const [depositAmount, setDepositAmount] = useState<string>('100');
  const [committedDays, setCommittedDays] = useState<number>(30);
  const [currentTier, setCurrentTier] = useState<number>(1);
  const [lifetimeDays, setLifetimeDays] = useState<number>(0);

  const tierApy = {
    1: 11.64,
    2: 17.45,
    3: 23.27,
  };

  const calculateYield = () => {
    const amount = parseFloat(depositAmount) || 0;
    const apy = tierApy[currentTier as keyof typeof tierApy];
    const days = committedDays;
    const totalDays = days + lifetimeDays;
    
    // Calculate loyalty multiplier (0.2x per year, max 2x)
    const yearsStaked = totalDays / 365;
    const loyaltyMultiplier = Math.min(1 + (yearsStaked * 0.2), 2);
    
    // Calculate yield for the commitment period
    const dailyRate = apy / 365 / 100;
    const yieldAmount = amount * dailyRate * days * loyaltyMultiplier;
    
    // Calculate fee (0.5%)
    const fee = amount * 0.005;
    const netDeposit = amount - fee;
    
    // Calculate total return
    const totalReturn = netDeposit + yieldAmount;
    const roi = ((totalReturn - amount) / amount) * 100;
    
    return {
      yieldAmount,
      fee,
      netDeposit,
      totalReturn,
      roi,
      loyaltyMultiplier,
      dailyYield: yieldAmount / days,
    };
  };

  const results = calculateYield();

  const commitmentOptions = [
    { days: 1, label: '1 Day', description: 'Quick test' },
    { days: 7, label: '1 Week', description: 'Short term' },
    { days: 30, label: '1 Month', description: 'Medium term' },
    { days: 90, label: '3 Months', description: 'Long term' },
    { days: 365, label: '1 Year', description: 'Maximum loyalty' },
  ];

  const tierOptions = [
    { id: 1, name: 'Bronze', apy: tierApy[1], color: 'from-amber-500 to-orange-500' },
    { id: 2, name: 'Silver', apy: tierApy[2], color: 'from-gray-400 to-gray-600' },
    { id: 3, name: 'Gold', apy: tierApy[3], color: 'from-yellow-400 to-yellow-600' },
  ];

  return (
    <motion.div
      className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <div className="flex items-center space-x-2 mb-6">
        <Calculator className="w-6 h-6 text-blue-400" />
        <h2 className="text-xl font-bold text-white">Yield Calculator</h2>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Input Section */}
        <div className="space-y-6">
          {/* Deposit Amount */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Deposit Amount (SOL)
            </label>
            <input
              type="number"
              value={depositAmount}
              onChange={(e) => setDepositAmount(e.target.value)}
              placeholder="100"
              min="0.1"
              step="0.1"
              className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Commitment Period */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Commitment Period
            </label>
            <div className="grid grid-cols-2 gap-2">
              {commitmentOptions.map((option) => (
                <button
                  key={option.days}
                  onClick={() => setCommittedDays(option.days)}
                  className={`p-3 rounded-lg border transition-all duration-200 ${
                    committedDays === option.days
                      ? 'bg-gradient-to-r from-purple-500 to-blue-500 border-purple-500 text-white'
                      : 'bg-white/5 border-white/20 text-gray-300 hover:bg-white/10'
                  }`}
                >
                  <div className="text-sm font-semibold">{option.label}</div>
                  <div className="text-xs opacity-75">{option.description}</div>
                </button>
              ))}
            </div>
          </div>

          {/* Tier Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Tier Level
            </label>
            <div className="space-y-2">
              {tierOptions.map((tier) => (
                <button
                  key={tier.id}
                  onClick={() => setCurrentTier(tier.id)}
                  className={`w-full p-3 rounded-lg border transition-all duration-200 ${
                    currentTier === tier.id
                      ? 'bg-gradient-to-r from-purple-500 to-blue-500 border-purple-500 text-white'
                      : 'bg-white/5 border-white/20 text-gray-300 hover:bg-white/10'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-semibold">{tier.name} Tier</span>
                    <span className="text-sm">{tier.apy}% APY</span>
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Lifetime Days */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Lifetime Staked Days (Optional)
            </label>
            <input
              type="number"
              value={lifetimeDays}
              onChange={(e) => setLifetimeDays(parseInt(e.target.value) || 0)}
              placeholder="0"
              min="0"
              className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="text-xs text-gray-400 mt-1">
              Previous staking history for loyalty multiplier calculation
            </p>
          </div>
        </div>

        {/* Results Section */}
        <div className="space-y-6">
          {/* Yield Summary */}
          <div className="bg-gradient-to-r from-green-500/10 to-emerald-500/10 border border-green-500/20 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Yield Projection</h3>
            
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span className="text-gray-400">Initial Deposit:</span>
                <span className="text-white font-semibold">{parseFloat(depositAmount).toFixed(2)} SOL</span>
              </div>
              
              <div className="flex justify-between items-center">
                <span className="text-gray-400">Protocol Fee (0.5%):</span>
                <span className="text-red-400 font-semibold">-{results.fee.toFixed(2)} SOL</span>
              </div>
              
              <div className="flex justify-between items-center">
                <span className="text-gray-400">Net Deposit:</span>
                <span className="text-white font-semibold">{results.netDeposit.toFixed(2)} SOL</span>
              </div>
              
              <div className="border-t border-white/10 pt-3">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-gray-400">Total Yield:</span>
                  <span className="text-green-400 font-semibold">+{results.yieldAmount.toFixed(2)} SOL</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-gray-400">Total Return:</span>
                  <span className="text-white font-bold text-lg">{results.totalReturn.toFixed(2)} SOL</span>
                </div>
              </div>
              
              <div className="bg-white/10 rounded-lg p-3">
                <div className="text-center">
                  <p className="text-2xl font-bold text-green-400">{results.roi.toFixed(2)}%</p>
                  <p className="text-sm text-gray-400">ROI for {committedDays} days</p>
                </div>
              </div>
            </div>
          </div>

          {/* Additional Metrics */}
          <div className="bg-white/5 rounded-lg p-6">
            <h4 className="text-white font-semibold mb-4">Additional Metrics</h4>
            
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-400">Daily Yield:</span>
                <span className="text-white">{(results.dailyYield * 1000).toFixed(2)} mSOL/day</span>
              </div>
              
              <div className="flex justify-between">
                <span className="text-gray-400">Loyalty Multiplier:</span>
                <span className="text-white">{results.loyaltyMultiplier.toFixed(2)}x</span>
              </div>
              
              <div className="flex justify-between">
                <span className="text-gray-400">Effective APY:</span>
                <span className="text-white">{(tierApy[currentTier as keyof typeof tierApy] * results.loyaltyMultiplier).toFixed(2)}%</span>
              </div>
              
              <div className="flex justify-between">
                <span className="text-gray-400">Total Days:</span>
                <span className="text-white">{committedDays + lifetimeDays} days</span>
              </div>
            </div>
          </div>

          {/* Warning */}
          <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4">
            <div className="flex items-start space-x-3">
              <AlertTriangle className="w-5 h-5 text-yellow-400 mt-0.5 flex-shrink-0" />
              <div className="text-sm text-yellow-200">
                <p className="font-medium mb-1">⚠️ Important Notes</p>
                <ul className="space-y-1 text-xs">
                  <li>• Yields only accrue for full days completed</li>
                  <li>• Early exit forfeits partial day rewards</li>
                  <li>• Tier levels may change during rebalancing</li>
                  <li>• APY rates are estimates and may vary</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
};

export default YieldCalculator;

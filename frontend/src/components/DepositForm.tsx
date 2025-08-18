import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useWallet } from '@solana/wallet-adapter-react';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Clock, AlertTriangle, CheckCircle, Info } from 'lucide-react';

interface DepositFormProps {}

const DepositForm: React.FC<DepositFormProps> = () => {
  const { publicKey } = useWallet();
  const [amount, setAmount] = useState<string>('');
  const [committedDays, setCommittedDays] = useState<number>(1);
  const [isLoading, setIsLoading] = useState(false);
  const [showPenaltyInfo, setShowPenaltyInfo] = useState(false);

  const handleDeposit = async () => {
    if (!publicKey || !amount || parseFloat(amount) <= 0) return;
    
    setIsLoading(true);
    try {
      // TODO: Implement actual deposit logic with Anchor
      console.log('Depositing:', {
        amount: parseFloat(amount) * LAMPORTS_PER_SOL,
        committedDays,
        user: publicKey.toString()
      });
      
      // Simulate transaction
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Reset form
      setAmount('');
      setCommittedDays(1);
    } catch (error) {
      console.error('Deposit failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const calculateFee = (amount: number) => {
    return amount * 0.005; // 0.5% fee
  };

  const netAmount = parseFloat(amount) || 0;
  const feeAmount = calculateFee(netAmount);
  const finalAmount = netAmount - feeAmount;

  const commitmentOptions = [
    { days: 1, label: '1 Day', description: 'Quick test' },
    { days: 7, label: '1 Week', description: 'Short term' },
    { days: 30, label: '1 Month', description: 'Medium term' },
    { days: 90, label: '3 Months', description: 'Long term' },
    { days: 365, label: '1 Year', description: 'Maximum loyalty' },
  ];

  return (
    <motion.div
      className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <div className="flex items-center space-x-2 mb-6">
        <div className="w-8 h-8 bg-gradient-to-r from-green-500 to-emerald-500 rounded-lg flex items-center justify-center">
          <svg className="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
        </div>
        <h2 className="text-xl font-bold text-white">Deposit & Commit</h2>
      </div>

      {/* Amount Input */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Deposit Amount (SOL)
        </label>
        <div className="relative">
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="0.0"
            min="0.1"
            step="0.1"
            className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
          />
          <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400">
            SOL
          </div>
        </div>
      </div>

      {/* Commitment Period */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Commitment Period
        </label>
        <div className="grid grid-cols-2 md:grid-cols-5 gap-2">
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

      {/* Penalty Warning */}
      <div className="mb-6">
        <button
          onClick={() => setShowPenaltyInfo(!showPenaltyInfo)}
          className="flex items-center space-x-2 text-yellow-400 hover:text-yellow-300 transition-colors"
        >
          <AlertTriangle className="w-4 h-4" />
          <span className="text-sm font-medium">Early Exit Penalty</span>
        </button>
        
        {showPenaltyInfo && (
          <motion.div
            className="mt-3 p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg"
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
          >
            <div className="flex items-start space-x-3">
              <Info className="w-5 h-5 text-yellow-400 mt-0.5 flex-shrink-0" />
              <div className="text-sm text-yellow-200">
                <p className="font-medium mb-2">⚠️ Commitment Penalty System</p>
                <ul className="space-y-1 text-xs">
                  <li>• Yields only accrue for <strong>full days completed</strong></li>
                  <li>• Early exit forfeits <strong>partial day rewards</strong></li>
                  <li>• Incomplete commitments return <strong>principal only</strong></li>
                  <li>• No principal loss - only yield penalties</li>
                </ul>
              </div>
            </div>
          </motion.div>
        )}
      </div>

      {/* Fee Breakdown */}
      {netAmount > 0 && (
        <div className="mb-6 p-4 bg-white/5 rounded-lg">
          <h4 className="text-sm font-medium text-gray-300 mb-3">Transaction Breakdown</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Deposit Amount:</span>
              <span className="text-white">{netAmount.toFixed(4)} SOL</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Protocol Fee (0.5%):</span>
              <span className="text-red-400">-{feeAmount.toFixed(4)} SOL</span>
            </div>
            <div className="border-t border-white/10 pt-2 flex justify-between font-medium">
              <span className="text-gray-300">Net Deposit:</span>
              <span className="text-green-400">{finalAmount.toFixed(4)} SOL</span>
            </div>
          </div>
        </div>
      )}

      {/* Commitment Summary */}
      <div className="mb-6 p-4 bg-gradient-to-r from-purple-500/10 to-blue-500/10 border border-purple-500/20 rounded-lg">
        <div className="flex items-center space-x-2 mb-2">
          <Clock className="w-4 h-4 text-purple-400" />
          <span className="text-sm font-medium text-purple-300">Commitment Summary</span>
        </div>
        <div className="text-sm text-gray-300">
          <p>You're committing to stake for <strong className="text-white">{committedDays} day{committedDays > 1 ? 's' : ''}</strong></p>
          <p className="text-xs mt-1">Minimum commitment: 1 day | Maximum: 365 days</p>
        </div>
      </div>

      {/* Deposit Button */}
      <button
        onClick={handleDeposit}
        disabled={!publicKey || !amount || parseFloat(amount) <= 0 || isLoading}
        className="w-full bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 disabled:from-gray-600 disabled:to-gray-600 disabled:cursor-not-allowed text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200 flex items-center justify-center space-x-2"
      >
        {isLoading ? (
          <>
            <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            <span>Processing...</span>
          </>
        ) : (
          <>
            <CheckCircle className="w-4 h-4" />
            <span>Deposit & Commit</span>
          </>
        )}
      </button>

      {/* Disclaimer */}
      <p className="text-xs text-gray-400 mt-4 text-center">
        By depositing, you agree to the commitment period and understand the penalty system for early exits.
      </p>
    </motion.div>
  );
};

export default DepositForm;

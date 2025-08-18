import React from 'react';
import { motion } from 'framer-motion';
import { useWallet } from '@solana/wallet-adapter-react';
import { TrendingUp, Users, DollarSign, Clock, Shield, Zap } from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface DashboardProps {}

const Dashboard: React.FC<DashboardProps> = () => {
  const { publicKey } = useWallet();

  // Mock data - in real implementation, this would come from the smart contract
  const mockData = {
    totalValueLocked: 1250000, // 1.25M SOL
    totalUsers: 847,
    averageApy: 17.45,
    totalFeesCollected: 6250, // 6,250 SOL
    dailyVolume: 45000, // 45K SOL
    fundGrowth: 23.7, // 23.7% growth
  };

  const chartData = [
    { day: 'Mon', tvl: 1200000, users: 820 },
    { day: 'Tue', tvl: 1220000, users: 835 },
    { day: 'Wed', tvl: 1235000, users: 840 },
    { day: 'Thu', tvl: 1242000, users: 842 },
    { day: 'Fri', tvl: 1248000, users: 845 },
    { day: 'Sat', tvl: 1250000, users: 847 },
    { day: 'Sun', tvl: 1250000, users: 847 },
  ];

  const formatSOL = (amount: number) => {
    if (amount >= 1000000) {
      return `${(amount / 1000000).toFixed(1)}M SOL`;
    } else if (amount >= 1000) {
      return `${(amount / 1000).toFixed(1)}K SOL`;
    }
    return `${amount.toFixed(0)} SOL`;
  };

  const metrics = [
    {
      title: 'Total Value Locked',
      value: formatSOL(mockData.totalValueLocked),
      change: '+2.3%',
      icon: DollarSign,
      color: 'from-green-500 to-emerald-500',
    },
    {
      title: 'Active Users',
      value: mockData.totalUsers.toLocaleString(),
      change: '+12',
      icon: Users,
      color: 'from-blue-500 to-cyan-500',
    },
    {
      title: 'Average APY',
      value: `${mockData.averageApy}%`,
      change: '+0.5%',
      icon: TrendingUp,
      color: 'from-purple-500 to-pink-500',
    },
    {
      title: 'Fund Growth',
      value: `${mockData.fundGrowth}%`,
      change: '+1.2%',
      icon: Zap,
      color: 'from-orange-500 to-red-500',
    },
  ];

  return (
    <motion.div
      className="space-y-6"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Dashboard</h1>
          <p className="text-gray-400">Welcome back! Here's your fund overview.</p>
        </div>
        <div className="flex items-center space-x-2 text-sm text-gray-400">
          <Clock className="w-4 h-4" />
          <span>Last updated: {new Date().toLocaleTimeString()}</span>
        </div>
      </div>

      {/* Key Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {metrics.map((metric, index) => (
          <motion.div
            key={metric.title}
            className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: index * 0.1 }}
          >
            <div className="flex items-center justify-between mb-4">
              <div className={`w-10 h-10 bg-gradient-to-r ${metric.color} rounded-lg flex items-center justify-center`}>
                <metric.icon className="w-5 h-5 text-white" />
              </div>
              <span className="text-green-400 text-sm font-medium">{metric.change}</span>
            </div>
            <h3 className="text-gray-400 text-sm font-medium mb-1">{metric.title}</h3>
            <p className="text-2xl font-bold text-white">{metric.value}</p>
          </motion.div>
        ))}
      </div>

      {/* Charts Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* TVL Chart */}
        <motion.div
          className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-white">Total Value Locked</h3>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-gradient-to-r from-green-500 to-emerald-500 rounded-full"></div>
              <span className="text-sm text-gray-400">7-day trend</span>
            </div>
          </div>
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
              <XAxis 
                dataKey="day" 
                stroke="rgba(255,255,255,0.6)" 
                fontSize={12}
              />
              <YAxis 
                stroke="rgba(255,255,255,0.6)" 
                fontSize={12}
                tickFormatter={(value) => `${(value / 1000000).toFixed(1)}M`}
              />
              <Tooltip 
                contentStyle={{
                  backgroundColor: 'rgba(0,0,0,0.8)',
                  border: '1px solid rgba(255,255,255,0.2)',
                  borderRadius: '8px',
                  color: 'white'
                }}
                formatter={(value: number) => [formatSOL(value), 'TVL']}
              />
              <Line 
                type="monotone" 
                dataKey="tvl" 
                stroke="url(#tvlGradient)" 
                strokeWidth={3}
                dot={{ fill: '#10b981', strokeWidth: 2, r: 4 }}
              />
              <defs>
                <linearGradient id="tvlGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0.1}/>
                </linearGradient>
              </defs>
            </LineChart>
          </ResponsiveContainer>
        </motion.div>

        {/* Users Chart */}
        <motion.div
          className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20"
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-white">Active Users</h3>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-gradient-to-r from-blue-500 to-cyan-500 rounded-full"></div>
              <span className="text-sm text-gray-400">7-day trend</span>
            </div>
          </div>
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
              <XAxis 
                dataKey="day" 
                stroke="rgba(255,255,255,0.6)" 
                fontSize={12}
              />
              <YAxis 
                stroke="rgba(255,255,255,0.6)" 
                fontSize={12}
              />
              <Tooltip 
                contentStyle={{
                  backgroundColor: 'rgba(0,0,0,0.8)',
                  border: '1px solid rgba(255,255,255,0.2)',
                  borderRadius: '8px',
                  color: 'white'
                }}
                formatter={(value: number) => [value.toLocaleString(), 'Users']}
              />
              <Line 
                type="monotone" 
                dataKey="users" 
                stroke="url(#usersGradient)" 
                strokeWidth={3}
                dot={{ fill: '#3b82f6', strokeWidth: 2, r: 4 }}
              />
              <defs>
                <linearGradient id="usersGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.1}/>
                </linearGradient>
              </defs>
            </LineChart>
          </ResponsiveContainer>
        </motion.div>
      </div>

      {/* Quick Stats */}
      <motion.div
        className="bg-white/10 backdrop-blur-lg rounded-xl p-6 border border-white/20"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.4 }}
      >
        <h3 className="text-lg font-semibold text-white mb-4">Protocol Statistics</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="text-center">
            <p className="text-2xl font-bold text-white">{formatSOL(mockData.totalFeesCollected)}</p>
            <p className="text-sm text-gray-400">Total Fees Collected</p>
          </div>
          <div className="text-center">
            <p className="text-2xl font-bold text-white">{formatSOL(mockData.dailyVolume)}</p>
            <p className="text-sm text-gray-400">24h Volume</p>
          </div>
          <div className="text-center">
            <p className="text-2xl font-bold text-white">3</p>
            <p className="text-sm text-gray-400">Active Funds</p>
          </div>
          <div className="text-center">
            <p className="text-2xl font-bold text-white">99.8%</p>
            <p className="text-sm text-gray-400">Uptime</p>
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
};

export default Dashboard;

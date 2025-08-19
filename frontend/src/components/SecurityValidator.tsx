import React, { useEffect, useState, ReactNode } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { Connection } from '@solana/web3.js';
import { Shield, AlertTriangle, CheckCircle } from 'lucide-react';

interface SecurityValidatorProps {
  children: ReactNode;
}

interface SecurityCheck {
  id: string;
  name: string;
  status: 'pending' | 'passed' | 'failed';
  message: string;
}

const SecurityValidator: React.FC<SecurityValidatorProps> = ({ children }) => {
  const { wallet, connected } = useWallet();
  const [securityChecks, setSecurityChecks] = useState<SecurityCheck[]>([
    { id: 'wallet', name: 'Wallet Security', status: 'pending', message: 'Checking wallet connection...' },
    { id: 'network', name: 'Network Validation', status: 'pending', message: 'Validating network connection...' },
    { id: 'csp', name: 'Content Security Policy', status: 'pending', message: 'Checking CSP headers...' },
    { id: 'https', name: 'HTTPS Connection', status: 'pending', message: 'Verifying secure connection...' },
  ]);
  const [showSecurityPanel, setShowSecurityPanel] = useState(false);
  const [allChecksPassed, setAllChecksPassed] = useState(false);

  useEffect(() => {
    performSecurityChecks();
  }, [connected, wallet]);

  const performSecurityChecks = async () => {
    const updatedChecks = [...securityChecks];

    // 1. Wallet Security Check
    if (connected && wallet) {
      updatedChecks[0] = {
        ...updatedChecks[0],
        status: 'passed',
        message: `Wallet connected: ${wallet.adapter.name}`
      };
    } else {
      updatedChecks[0] = {
        ...updatedChecks[0],
        status: 'failed',
        message: 'No wallet connected'
      };
    }

    // 2. Network Validation
    try {
      const connection = new Connection(process.env.REACT_APP_RPC_URL || 'https://api.devnet.solana.com');
      const health = await connection.getHealth();
      updatedChecks[1] = {
        ...updatedChecks[1],
        status: health === 'ok' ? 'passed' : 'failed',
        message: health === 'ok' ? 'Network connection healthy' : 'Network connection issues detected'
      };
    } catch (error) {
      updatedChecks[1] = {
        ...updatedChecks[1],
        status: 'failed',
        message: 'Unable to connect to Solana network'
      };
    }

    // 3. CSP Check (basic check for development)
    const hasCsp = document.querySelector('meta[http-equiv="Content-Security-Policy"]');
    updatedChecks[2] = {
      ...updatedChecks[2],
      status: hasCsp ? 'passed' : 'failed',
      message: hasCsp ? 'CSP headers detected' : 'No CSP headers found (consider adding for production)'
    };

    // 4. HTTPS Check
    const isHttps = window.location.protocol === 'https:' || window.location.hostname === 'localhost';
    updatedChecks[3] = {
      ...updatedChecks[3],
      status: isHttps ? 'passed' : 'failed',
      message: isHttps ? 'Secure HTTPS connection' : 'Insecure HTTP connection detected'
    };

    setSecurityChecks(updatedChecks);
    setAllChecksPassed(updatedChecks.every(check => check.status === 'passed'));
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'passed':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'failed':
        return <AlertTriangle className="w-4 h-4 text-red-500" />;
      default:
        return <div className="w-4 h-4 border-2 border-yellow-500 border-t-transparent rounded-full animate-spin" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'passed':
        return 'text-green-500';
      case 'failed':
        return 'text-red-500';
      default:
        return 'text-yellow-500';
    }
  };

  // Input sanitization helper
  const sanitizeInput = (input: string): string => {
    return input
      .replace(/[<>]/g, '') // Remove potential HTML tags
      .replace(/javascript:/gi, '') // Remove javascript: protocol
      .replace(/on\w+=/gi, '') // Remove event handlers
      .trim();
  };

  // Validate numeric inputs
  const validateNumericInput = (value: string, min: number, max: number): number | null => {
    const sanitized = sanitizeInput(value);
    const num = parseFloat(sanitized);
    
    if (isNaN(num) || num < min || num > max) {
      return null;
    }
    
    return num;
  };

  // Export validation functions for use in other components
  React.useEffect(() => {
    // Make validation functions available globally for other components
    (window as any).securityValidator = {
      sanitizeInput,
      validateNumericInput,
    };
  }, []);

  if (!allChecksPassed) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-red-900 via-purple-900 to-indigo-900 flex items-center justify-center p-4">
        <div className="bg-black/30 backdrop-blur-lg rounded-lg p-8 max-w-md w-full border border-red-500/30">
          <div className="text-center mb-6">
            <Shield className="w-16 h-16 text-red-500 mx-auto mb-4" />
            <h2 className="text-2xl font-bold text-white mb-2">Security Validation</h2>
            <p className="text-gray-300">Performing security checks before allowing access...</p>
          </div>
          
          <div className="space-y-3">
            {securityChecks.map((check) => (
              <div key={check.id} className="flex items-center space-x-3 p-3 bg-black/20 rounded-lg">
                {getStatusIcon(check.status)}
                <div className="flex-1">
                  <div className="font-medium text-white">{check.name}</div>
                  <div className={`text-sm ${getStatusColor(check.status)}`}>
                    {check.message}
                  </div>
                </div>
              </div>
            ))}
          </div>
          
          <button
            onClick={performSecurityChecks}
            className="w-full mt-6 bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 text-white font-semibold py-3 px-4 rounded-lg transition-all duration-200"
          >
            Retry Security Checks
          </button>
        </div>
      </div>
    );
  }

  return (
    <>
      {children}
      
      {/* Security Panel Toggle */}
      <div className="fixed bottom-4 right-4 z-50">
        <button
          onClick={() => setShowSecurityPanel(!showSecurityPanel)}
          className="bg-green-600 hover:bg-green-700 text-white p-3 rounded-full shadow-lg transition-all duration-200"
          title="Security Status"
        >
          <Shield className="w-5 h-5" />
        </button>
        
        {showSecurityPanel && (
          <div className="absolute bottom-16 right-0 bg-black/90 backdrop-blur-lg rounded-lg p-4 w-80 border border-green-500/30">
            <h3 className="text-white font-semibold mb-3">Security Status</h3>
            <div className="space-y-2">
              {securityChecks.map((check) => (
                <div key={check.id} className="flex items-center space-x-2 text-sm">
                  {getStatusIcon(check.status)}
                  <span className="text-white">{check.name}</span>
                  <span className={getStatusColor(check.status)}>✓</span>
                </div>
              ))}
            </div>
            <div className="mt-3 pt-3 border-t border-gray-600">
              <p className="text-green-400 text-sm">All security checks passed</p>
            </div>
          </div>
        )}
      </div>
    </>
  );
};

export default SecurityValidator;
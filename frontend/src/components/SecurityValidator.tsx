import React, { useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { AlertTriangle, Shield, CheckCircle } from 'lucide-react';

interface SecurityValidatorProps {
  children: React.ReactNode;
}

const SecurityValidator: React.FC<SecurityValidatorProps> = ({ children }) => {
  const { publicKey, connected } = useWallet();
  const [securityChecks, setSecurityChecks] = useState({
    walletConnected: false,
    validPublicKey: false,
    secureConnection: false,
  });

  useEffect(() => {
    if (connected && publicKey) {
      setSecurityChecks({
        walletConnected: true,
        validPublicKey: publicKey.toString().length === 44, // Valid Solana public key length
        secureConnection: window.location.protocol === 'https:' || window.location.hostname === 'localhost',
      });
    } else {
      setSecurityChecks({
        walletConnected: false,
        validPublicKey: false,
        secureConnection: window.location.protocol === 'https:' || window.location.hostname === 'localhost',
      });
    }
  }, [connected, publicKey]);

  const allChecksPass = Object.values(securityChecks).every(check => check);

  if (!allChecksPass) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900 flex items-center justify-center">
        <div className="bg-red-500/20 backdrop-blur-lg rounded-xl p-8 border border-red-500/30 max-w-md mx-auto">
          <div className="flex items-center space-x-3 mb-6">
            <AlertTriangle className="w-8 h-8 text-red-400" />
            <h2 className="text-xl font-bold text-white">Security Check Failed</h2>
          </div>
          
          <div className="space-y-4">
            <SecurityCheck
              label="Wallet Connected"
              passed={securityChecks.walletConnected}
              description="A valid Solana wallet must be connected"
            />
            <SecurityCheck
              label="Valid Public Key"
              passed={securityChecks.validPublicKey}
              description="Public key must be a valid Solana address"
            />
            <SecurityCheck
              label="Secure Connection"
              passed={securityChecks.secureConnection}
              description="Connection must be secure (HTTPS)"
            />
          </div>
          
          <p className="text-gray-300 text-sm mt-6">
            Please resolve the security issues above before proceeding.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div>
      {/* Security indicator */}
      <div className="fixed top-4 right-4 z-50">
        <div className="bg-green-500/20 backdrop-blur-lg rounded-lg px-3 py-2 border border-green-500/30">
          <div className="flex items-center space-x-2">
            <Shield className="w-4 h-4 text-green-400" />
            <span className="text-green-400 text-sm font-medium">Secure</span>
          </div>
        </div>
      </div>
      {children}
    </div>
  );
};

interface SecurityCheckProps {
  label: string;
  passed: boolean;
  description: string;
}

const SecurityCheck: React.FC<SecurityCheckProps> = ({ label, passed, description }) => {
  return (
    <div className="flex items-start space-x-3">
      {passed ? (
        <CheckCircle className="w-5 h-5 text-green-400 mt-0.5" />
      ) : (
        <AlertTriangle className="w-5 h-5 text-red-400 mt-0.5" />
      )}
      <div>
        <div className={`font-medium ${passed ? 'text-green-400' : 'text-red-400'}`}>
          {label}
        </div>
        <div className="text-gray-400 text-sm">{description}</div>
      </div>
    </div>
  );
};

export default SecurityValidator;
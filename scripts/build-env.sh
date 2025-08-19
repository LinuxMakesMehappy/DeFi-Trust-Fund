#!/bin/bash

# Build script for different environments
# Usage: ./scripts/build-env.sh [localnet|devnet|testnet|mainnet]

set -e

ENVIRONMENT=${1:-localnet}

echo "🔨 Building DeFi Trust Fund for $ENVIRONMENT environment..."

# Validate environment
case $ENVIRONMENT in
  localnet|devnet|testnet|mainnet)
    echo "✅ Valid environment: $ENVIRONMENT"
    ;;
  *)
    echo "❌ Invalid environment: $ENVIRONMENT"
    echo "Valid environments: localnet, devnet, testnet, mainnet"
    exit 1
    ;;
esac

# Clean previous builds
echo "🧹 Cleaning previous builds..."
anchor clean

# Build with environment-specific features
echo "🔨 Building with $ENVIRONMENT feature..."
cargo build-bpf --features $ENVIRONMENT

# Build the Anchor project
echo "📦 Building Anchor project..."
anchor build

# Generate TypeScript types
echo "📝 Generating TypeScript types..."
anchor build --idl target/idl

echo "✅ Build completed for $ENVIRONMENT environment!"
echo ""
echo "📋 Next steps:"
echo "1. Deploy: anchor deploy --provider.cluster $ENVIRONMENT"
echo "2. Test: anchor test --provider.cluster $ENVIRONMENT"
echo "3. Verify deployment: solana program show <program_id> --url $ENVIRONMENT"
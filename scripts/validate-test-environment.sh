#!/bin/bash

# Test Environment Validation Script
# Validates that all testing prerequisites are met

echo "🔍 DeFi Trust Fund - Test Environment Validation"
echo "================================================"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

VALIDATION_ERRORS=0

# Function to check requirement
check_requirement() {
    local name="$1"
    local command="$2"
    local required="$3"
    
    echo -n "Checking $name... "
    
    if eval "$command" &>/dev/null; then
        echo -e "${GREEN}✅ Available${NC}"
        if [[ "$required" == "true" ]]; then
            eval "$command" 2>/dev/null | head -1
        fi
    else
        if [[ "$required" == "true" ]]; then
            echo -e "${RED}❌ REQUIRED - Not found${NC}"
            VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
        else
            echo -e "${YELLOW}⚠️ Optional - Not found${NC}"
        fi
    fi
}

echo "🔧 Checking Core Dependencies"
echo "=============================="

check_requirement "Rust" "rustc --version" "true"
check_requirement "Cargo" "cargo --version" "true"
check_requirement "Node.js" "node --version" "true"
check_requirement "NPM" "npm --version" "true"

echo ""
echo "🔗 Checking Solana Dependencies"
echo "==============================="

check_requirement "Solana CLI" "solana --version" "true"
check_requirement "Anchor CLI" "anchor --version" "true"

echo ""
echo "🧪 Checking Testing Tools"
echo "========================="

check_requirement "Cargo Audit" "cargo audit --version" "false"
check_requirement "Cargo Fmt" "cargo fmt --version" "true"
check_requirement "Cargo Clippy" "cargo clippy --version" "true"
check_requirement "Cargo Tarpaulin" "cargo tarpaulin --version" "false"
check_requirement "Cargo Fuzz" "cargo fuzz --version" "false"

echo ""
echo "🔒 Checking Security Tools"
echo "=========================="

check_requirement "Semgrep" "semgrep --version" "false"
check_requirement "Python3" "python3 --version" "false"

echo ""
echo "📁 Checking Project Structure"
echo "============================="

PROJECT_DIRS=("src" "tests" "frontend" "docs" ".github" "scripts")
PROJECT_FILES=("Cargo.toml" "package.json" "Anchor.toml" "README.md")

for dir in "${PROJECT_DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        echo -e "Directory $dir: ${GREEN}✅ Present${NC}"
    else
        echo -e "Directory $dir: ${RED}❌ Missing${NC}"
        VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
    fi
done

for file in "${PROJECT_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        echo -e "File $file: ${GREEN}✅ Present${NC}"
    else
        echo -e "File $file: ${RED}❌ Missing${NC}"
        VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
    fi
done

echo ""
echo "🧪 Checking Test Files"
echo "======================"

TEST_FILES=(
    "tests/comprehensive-test-suite.ts"
    "tests/security-attack-simulations.ts"
    "tests/end-to-end-integration.ts"
    "tests/performance-load-tests.ts"
    "tests/chaos-engineering-tests.ts"
    "frontend/src/__tests__/comprehensive-frontend-tests.test.tsx"
)

for test_file in "${TEST_FILES[@]}"; do
    if [[ -f "$test_file" ]]; then
        echo -e "Test file $test_file: ${GREEN}✅ Present${NC}"
    else
        echo -e "Test file $test_file: ${YELLOW}⚠️ Missing${NC}"
    fi
done

echo ""
echo "🔄 Checking CI/CD Configuration"
echo "==============================="

CICD_FILES=(
    ".github/workflows/ci-security.yml"
    ".github/workflows/deployment-pipeline.yml"
    ".github/workflows/security-monitoring.yml"
    ".github/workflows/formal-verification.yml"
    ".github/workflows/monitoring-setup.yml"
    ".github/workflows/compliance-audit.yml"
    ".github/dependabot.yml"
)

for cicd_file in "${CICD_FILES[@]}"; do
    if [[ -f "$cicd_file" ]]; then
        echo -e "CI/CD file $cicd_file: ${GREEN}✅ Present${NC}"
    else
        echo -e "CI/CD file $cicd_file: ${YELLOW}⚠️ Missing${NC}"
    fi
done

echo ""
echo "🌐 Checking Network Configuration"
echo "================================="

# Check Solana network configuration
if command -v solana &> /dev/null; then
    SOLANA_CONFIG=$(solana config get 2>/dev/null || echo "")
    if [[ -n "$SOLANA_CONFIG" ]]; then
        echo -e "Solana configuration: ${GREEN}✅ Configured${NC}"
        echo "Current network: $(echo "$SOLANA_CONFIG" | grep "RPC URL" | cut -d: -f2- | xargs)"
    else
        echo -e "Solana configuration: ${YELLOW}⚠️ Not configured${NC}"
    fi
else
    echo -e "Solana CLI: ${RED}❌ Not available${NC}"
    VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
fi

echo ""
echo "💾 Checking Available Resources"
echo "==============================="

# Check disk space
DISK_USAGE=$(df -h . | tail -1 | awk '{print $5}' | sed 's/%//')
if [[ $DISK_USAGE -lt 80 ]]; then
    echo -e "Disk space: ${GREEN}✅ Sufficient (${DISK_USAGE}% used)${NC}"
else
    echo -e "Disk space: ${YELLOW}⚠️ Limited (${DISK_USAGE}% used)${NC}"
fi

# Check memory
if command -v free &> /dev/null; then
    MEMORY_USAGE=$(free | grep Mem | awk '{printf "%.0f", $3/$2 * 100.0}')
    if [[ $MEMORY_USAGE -lt 80 ]]; then
        echo -e "Memory usage: ${GREEN}✅ Good (${MEMORY_USAGE}% used)${NC}"
    else
        echo -e "Memory usage: ${YELLOW}⚠️ High (${MEMORY_USAGE}% used)${NC}"
    fi
fi

echo ""
echo "📊 Environment Validation Summary"
echo "================================="

if [[ $VALIDATION_ERRORS -eq 0 ]]; then
    echo -e "${GREEN}🎉 ENVIRONMENT VALIDATION SUCCESSFUL! 🎉${NC}"
    echo -e "${GREEN}✅ All critical requirements met${NC}"
    echo -e "${GREEN}✅ Testing environment ready${NC}"
    echo -e "${GREEN}🚀 Ready to execute comprehensive test suite${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Run comprehensive tests: npm run test:all"
    echo "2. Execute security simulations: npm run test:security:attacks"
    echo "3. Validate performance: npm run test:performance"
    echo "4. Test chaos scenarios: npm run test:chaos"
    echo ""
    exit 0
else
    echo -e "${RED}❌ ENVIRONMENT VALIDATION FAILED${NC}"
    echo -e "${RED}Found $VALIDATION_ERRORS critical issues${NC}"
    echo ""
    echo -e "${YELLOW}Required actions:${NC}"
    echo "1. Install missing critical dependencies"
    echo "2. Configure Solana CLI and network"
    echo "3. Ensure all project files are present"
    echo "4. Re-run validation: bash scripts/validate-test-environment.sh"
    echo ""
    exit 1
fi
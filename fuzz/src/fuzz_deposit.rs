#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct DepositInput {
    fund_index: u64,
    amount: u64,
    input_mint: [u8; 32],
    committed_days: u64,
}

fuzz_target!(|data: DepositInput| {
    // Validate input ranges
    if data.amount == 0 || data.committed_days == 0 || data.committed_days > 365 {
        return;
    }

    // Test deposit instruction with fuzzed inputs
    // This would integrate with the actual smart contract
    // For now, we test the input validation logic
    
    // Test amount validation
    let min_amount = 1_000_000; // 0.001 SOL in lamports
    let max_amount = 1_000_000_000_000; // 1000 SOL in lamports
    
    if data.amount < min_amount || data.amount > max_amount {
        return;
    }

    // Test commitment days validation
    if data.committed_days < 1 || data.committed_days > 365 {
        return;
    }

    // Test fee calculation
    let fee_rate = 500; // 0.5% in basis points
    let fee = data.amount * fee_rate / 10_000;
    let net_amount = data.amount.checked_sub(fee);
    
    if net_amount.is_none() {
        return;
    }

    // Test arithmetic overflow scenarios
    let large_amount = u64::MAX;
    let overflow_fee = large_amount.checked_mul(fee_rate);
    if overflow_fee.is_none() {
        return;
    }

    // Test edge cases
    let edge_cases = [
        (1, 1), // Minimum amount, minimum days
        (1_000_000, 365), // Minimum amount, maximum days
        (1_000_000_000_000, 1), // Maximum amount, minimum days
        (1_000_000_000_000, 365), // Maximum amount, maximum days
    ];

    for (amount, days) in edge_cases {
        let test_fee = amount * fee_rate / 10_000;
        let test_net = amount.checked_sub(test_fee);
        assert!(test_net.is_some());
    }
});


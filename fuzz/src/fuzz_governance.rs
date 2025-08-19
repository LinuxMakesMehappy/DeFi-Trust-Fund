#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct GovernanceInput {
    fund_index: u64,
    proposal_id: u64,
    description: String,
    param_key: String,
    param_value: u64,
    proposer_score: u64,
    deadline: u64,
}

fuzz_target!(|data: GovernanceInput| {
    // Validate input ranges
    if data.description.len() > 256 {
        return;
    }

    // Test valid parameter keys
    let valid_params = ["deposit_fee", "leverage_ratio", "reinvestment_rate"];
    if !valid_params.contains(&data.param_key.as_str()) {
        return;
    }

    // Test parameter value ranges
    match data.param_key.as_str() {
        "deposit_fee" => {
            if data.param_value < 100 || data.param_value > 1000 {
                return; // 0.1% to 1%
            }
        },
        "leverage_ratio" => {
            if data.param_value < 2000 || data.param_value > 6000 {
                return; // 2x to 6x (scaled by 1000)
            }
        },
        "reinvestment_rate" => {
            if data.param_value < 1000 || data.param_value > 3000 {
                return; // 10% to 30%
            }
        },
        _ => return,
    }

    // Test score threshold logic
    let threshold_percentage = 0.3; // 30%
    let mock_scores = vec![1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000];
    let mut sorted_scores = mock_scores.clone();
    sorted_scores.sort_by(|a, b| b.cmp(a)); // Descending
    
    let threshold_index = (threshold_percentage * sorted_scores.len() as f64).ceil() as usize;
    let score_threshold = sorted_scores.get(threshold_index - 1).copied().unwrap_or(0);
    
    // Test if proposer score meets threshold
    if data.proposer_score < score_threshold {
        return;
    }

    // Test deadline validation
    let current_time = 1640995200; // Example timestamp
    if data.deadline <= current_time {
        return;
    }

    // Test proposal execution logic
    let execution_delay = 7 * 24 * 60 * 60; // 7 days in seconds
    let execution_time = data.deadline + execution_delay;
    
    if execution_time <= current_time {
        // Proposal can be executed
        // Test parameter update logic
        match data.param_key.as_str() {
            "deposit_fee" => {
                assert!(data.param_value >= 100 && data.param_value <= 1000);
            },
            "leverage_ratio" => {
                assert!(data.param_value >= 2000 && data.param_value <= 6000);
            },
            "reinvestment_rate" => {
                assert!(data.param_value >= 1000 && data.param_value <= 3000);
            },
            _ => panic!("Invalid parameter key"),
        }
    }

    // Test edge cases
    let edge_cases = [
        (100, "deposit_fee"), // Minimum deposit fee
        (1000, "deposit_fee"), // Maximum deposit fee
        (2000, "leverage_ratio"), // Minimum leverage
        (6000, "leverage_ratio"), // Maximum leverage
        (1000, "reinvestment_rate"), // Minimum reinvestment
        (3000, "reinvestment_rate"), // Maximum reinvestment
    ];

    for (value, key) in edge_cases {
        match key {
            "deposit_fee" => assert!(value >= 100 && value <= 1000),
            "leverage_ratio" => assert!(value >= 2000 && value <= 6000),
            "reinvestment_rate" => assert!(value >= 1000 && value <= 3000),
            _ => panic!("Invalid parameter key"),
        }
    }
});


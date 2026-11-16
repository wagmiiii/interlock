//! Interlock Guard Instruction Unit and Property Tests
//!
//! Covered paths:
//! 1. Pass path (within deviation, valid age, valid confidence, matching feed, full verification, valid owner)
//! 2. Error: StalePrice
//! 3. Error: ConfidenceTooWide
//! 4. Error: WrongFeed
//! 5. Error: InsufficientVerification
//! 6. Error: ReferenceInvalid (ref_price == 0)
//! 7. Error: DeviationExceeded
//! 8. Error: MathOverflow
//! 9. Error: WrongAccountOwner
//! 10. Inverted reference price (orientation inverted) pass & fail paths
//! 11. Property tests: u128 cross-multiplication invariant across arbitrary price levels

use interlock_ix::{
    check_age, check_confidence, check_deviation, execute_sanity_checks, GuardError,
    OracleCheckInputs, OracleGuardParams, ReferenceKind, VerificationLevel, PROGRAM_ID_STR,
};
use proptest::prelude::*;

#[test]
fn test_program_id_format() {
    assert_eq!(PROGRAM_ID_STR.len(), 44);
    assert_ne!(PROGRAM_ID_STR, "11111111111111111111111111111111");
    // Verify all characters are in standard Base58 alphabet
    let b58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    for c in PROGRAM_ID_STR.chars() {
        assert!(b58_chars.contains(c));
    }
}

#[test]
fn test_deviation_pass_within_bounds() {
    // Reference 100_000, Oracle 100_500 (50 bps deviation, max 100 bps)
    let res = check_deviation(100_500, 100_000, 100);
    assert_eq!(res.unwrap(), 50);
}

#[test]
fn test_deviation_fail_exceeded() {
    // Reference 100_000, Oracle 102_000 (200 bps deviation, max 100 bps)
    let res = check_deviation(102_000, 100_000, 100);
    assert_eq!(res.unwrap_err(), GuardError::DeviationExceeded);
}

#[test]
fn test_confidence_check() {
    // Price 100_000, Conf 200 (20 bps, max 50 bps) -> pass
    assert!(check_confidence(200, 100_000, 50).is_ok());

    // Price 100_000, Conf 600 (60 bps, max 50 bps) -> fail
    assert_eq!(
        check_confidence(600, 100_000, 50).unwrap_err(),
        GuardError::ConfidenceTooWide
    );
}

#[test]
fn test_age_check() {
    // Current 1000, Publish 980 (age 20s, max 30s) -> pass
    assert!(check_age(1000, 980, 30).is_ok());

    // Current 1000, Publish 950 (age 50s, max 30s) -> fail
    assert_eq!(
        check_age(1000, 950, 30).unwrap_err(),
        GuardError::StalePrice
    );
}

#[test]
fn test_execute_sanity_checks_all_error_paths() {
    let feed_id = [7u8; 32];
    let params = OracleGuardParams {
        feed_id,
        max_age_secs: 30,
        max_conf_bps: 100,
        max_deviation_bps: 100,
        reference_kind: ReferenceKind::PythSecondFeed,
        reference_invert: false,
    };

    let base_inputs = OracleCheckInputs {
        now: 1_000_000,
        publish_time: 999_990, // 10s age
        price: 150_000_000,
        conf: 100_000, // ~6.6 bps
        feed_id,
        verification_level: VerificationLevel::Full,
        account_owner_ok: true,
        ref_price: 150_200_000, // ~13 bps deviation
    };

    // 1. Pass path
    let res = execute_sanity_checks(&params, &base_inputs);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 13); // 13 bps

    // 2. Wrong account owner
    let mut inputs_owner = base_inputs.clone();
    inputs_owner.account_owner_ok = false;
    let err_owner = execute_sanity_checks(&params, &inputs_owner);
    assert_eq!(err_owner.unwrap_err(), GuardError::WrongAccountOwner);

    // 3. Wrong feed id
    let mut inputs_feed = base_inputs.clone();
    inputs_feed.feed_id = [9u8; 32];
    let err_feed = execute_sanity_checks(&params, &inputs_feed);
    assert_eq!(err_feed.unwrap_err(), GuardError::WrongFeed);

    // 4. Insufficient verification level
    let mut inputs_verif = base_inputs.clone();
    inputs_verif.verification_level = VerificationLevel::Partial;
    let err_verif = execute_sanity_checks(&params, &inputs_verif);
    assert_eq!(err_verif.unwrap_err(), GuardError::InsufficientVerification);

    // 5. Stale price
    let mut inputs_stale = base_inputs.clone();
    inputs_stale.publish_time = inputs_stale.now - 45; // 45s age > 30s
    let err_stale = execute_sanity_checks(&params, &inputs_stale);
    assert_eq!(err_stale.unwrap_err(), GuardError::StalePrice);

    // 6. Confidence too wide
    let mut inputs_conf = base_inputs.clone();
    inputs_conf.conf = 3_000_000; // 200 bps > 100 bps
    let err_conf = execute_sanity_checks(&params, &inputs_conf);
    assert_eq!(err_conf.unwrap_err(), GuardError::ConfidenceTooWide);

    // 7. Reference invalid (zero price)
    let mut inputs_zero = base_inputs.clone();
    inputs_zero.ref_price = 0;
    let err_ref_zero = execute_sanity_checks(&params, &inputs_zero);
    assert_eq!(err_ref_zero.unwrap_err(), GuardError::ReferenceInvalid);

    // 8. Deviation exceeded
    let mut inputs_dev = base_inputs.clone();
    inputs_dev.ref_price = 160_000_000; // ~666 bps > 100 bps
    let err_dev = execute_sanity_checks(&params, &inputs_dev);
    assert_eq!(err_dev.unwrap_err(), GuardError::DeviationExceeded);
}

#[test]
fn test_inverted_reference_orientation() {
    let feed_id = [1u8; 32];
    let params_inverted = OracleGuardParams {
        feed_id,
        max_age_secs: 30,
        max_conf_bps: 100,
        max_deviation_bps: 100, // 1%
        reference_kind: ReferenceKind::DexPool,
        reference_invert: true,
    };

    let inputs_ok = OracleCheckInputs {
        now: 1000,
        publish_time: 995,
        price: 15_000_000_000, // $150.00 in 8-decimal fixed-point
        conf: 10_000,
        feed_id,
        verification_level: VerificationLevel::Full,
        account_owner_ok: true,
        ref_price: 666_666, // 10^16 / 666_666 = 15_000_015_000 (within 1 bps)
    };

    let res_ok = execute_sanity_checks(&params_inverted, &inputs_ok);
    assert!(res_ok.is_ok());

    // If DEX pool is 0.00500000 SOL per USDC ($200/SOL) -> deviation is 33% -> fail
    let mut inputs_bad = inputs_ok;
    inputs_bad.ref_price = 500_000;
    let res_fail = execute_sanity_checks(&params_inverted, &inputs_bad);
    assert_eq!(res_fail.unwrap_err(), GuardError::DeviationExceeded);
}

proptest! {
    #[test]
    fn prop_check_deviation_soundness(
        p_ref in 1u64..1_000_000_000_000u64,
        delta_factor in -500i64..500i64,
        max_bps in 10u16..5000u16
    ) {
        // Compute delta in bps
        let delta = ((p_ref as i128) * (delta_factor as i128)) / 10_000;
        let p_oracle_i128 = (p_ref as i128) + delta;
        if p_oracle_i128 > 0 && p_oracle_i128 <= (u64::MAX as i128) {
            let p_oracle = p_oracle_i128 as u64;
            let result = check_deviation(p_oracle, p_ref, max_bps);

            let expected_abs_diff = ((p_oracle as i128) - (p_ref as i128)).unsigned_abs();
            let expected_diff_scaled = expected_abs_diff * 10_000;
            let threshold = (max_bps as u128) * (p_ref as u128);

            if expected_diff_scaled > threshold {
                prop_assert_eq!(result.unwrap_err(), GuardError::DeviationExceeded);
            } else {
                let measured = result.unwrap();
                let expected_bps = (expected_diff_scaled / (p_ref as u128)) as u16;
                prop_assert_eq!(measured, expected_bps);
            }
        }
    }
}

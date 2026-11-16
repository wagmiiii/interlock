use interlock_ix::{check_age, check_confidence, check_deviation, GuardError};

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

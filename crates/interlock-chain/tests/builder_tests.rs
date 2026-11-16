use interlock_chain::{InstructionType, TransactionComposer, TxCompositionError};
use interlock_ix::{GuardError, OracleGuardParams, ReferenceKind};

fn dummy_params() -> OracleGuardParams {
    OracleGuardParams {
        feed_id: [1u8; 32],
        max_age_secs: 30,
        max_conf_bps: 50,
        max_deviation_bps: 100,
        reference_kind: ReferenceKind::PythSecondFeed,
        reference_invert: false,
    }
}

#[test]
fn test_composer_correct_order() {
    let mut composer = TransactionComposer::new();
    composer
        .add_compute_budget()
        .unwrap()
        .add_oracle_price_update()
        .unwrap()
        .add_guard(dummy_params())
        .unwrap()
        .add_venue_trade("jupiter_open_position")
        .unwrap();

    let ixs = composer.build().unwrap();
    assert_eq!(ixs.len(), 4);
    assert_eq!(ixs[0], InstructionType::ComputeBudget);
    assert_eq!(ixs[1], InstructionType::OraclePriceUpdate);
    assert!(matches!(ixs[2], InstructionType::InterlockGuard(_)));
    assert!(matches!(
        ixs[3],
        InstructionType::VenueTradeInstruction { .. }
    ));
}

#[test]
fn test_composer_refuses_guard_after_trade() {
    let mut composer = TransactionComposer::new();
    composer.add_guard(dummy_params()).unwrap();
    composer.add_venue_trade("jupiter_open").unwrap();

    // Attempting to add guard after trade must fail
    let res = composer.add_guard(dummy_params());
    assert_eq!(
        res.unwrap_err(),
        TxCompositionError::InvalidInstructionOrder
    );
}

#[test]
fn test_composer_refuses_trade_without_guard() {
    let mut composer = TransactionComposer::new();
    // Attempting to add trade before guard must fail
    let res = composer.add_venue_trade("jupiter_open");
    assert_eq!(
        res.unwrap_err(),
        TxCompositionError::MissingGuardInstruction
    );
}

#[test]
fn test_error_diagnostic_mapping() {
    let msg = TransactionComposer::map_error_to_diagnostic(GuardError::DeviationExceeded);
    assert!(msg.contains("deviates excessively"));

    let msg_stale = TransactionComposer::map_error_to_diagnostic(GuardError::StalePrice);
    assert!(msg_stale.contains("stale"));
}

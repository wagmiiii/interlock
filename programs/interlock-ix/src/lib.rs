pub mod checks;
pub mod errors;
pub mod reference;

pub use checks::{check_age, check_confidence, check_deviation};
pub use errors::GuardError;
pub use reference::{ReferenceKind, ReferencePrice};

/// Arguments passed into the assert_oracle_sane instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleGuardParams {
    pub feed_id: [u8; 32],
    pub max_age_secs: u64,
    pub max_conf_bps: u16,
    pub max_deviation_bps: u16,
    pub reference_kind: ReferenceKind,
    pub reference_invert: bool,
}

/// Pure validation helper invoked by the Anchor instruction handler.
pub fn execute_sanity_checks(
    params: &OracleGuardParams,
    now: u64,
    publish_time: u64,
    price: u64,
    conf: u64,
    ref_price: u64,
) -> Result<u16, GuardError> {
    check_age(now, publish_time, params.max_age_secs)?;
    check_confidence(conf, price, params.max_conf_bps)?;
    let measured_bps = check_deviation(price, ref_price, params.max_deviation_bps)?;
    Ok(measured_bps)
}

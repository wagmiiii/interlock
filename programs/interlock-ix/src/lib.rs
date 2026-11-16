pub mod checks;
pub mod errors;
pub mod reference;

pub use checks::{check_age, check_confidence, check_deviation};
pub use errors::GuardError;
pub use reference::{ReferenceKind, ReferencePrice};

/// Canonical program ID for interlock-ix on Solana networks.
pub const PROGRAM_ID_STR: &str = "A2qRStzDzSJw8HnyJN2dudHHLuqeybZGh3YoM7XPFneW";

/// Verification level of a Pyth price update account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    Partial,
    Full,
}

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

/// Inputs observed from accounts for oracle sanity verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleCheckInputs {
    pub now: u64,
    pub publish_time: u64,
    pub price: u64,
    pub conf: u64,
    pub feed_id: [u8; 32],
    pub verification_level: VerificationLevel,
    pub account_owner_ok: bool,
    pub ref_price: u64,
}

/// Pure validation helper invoked by the Anchor instruction handler.
pub fn execute_sanity_checks(
    params: &OracleGuardParams,
    inputs: &OracleCheckInputs,
) -> Result<u16, GuardError> {
    if !inputs.account_owner_ok {
        return Err(GuardError::WrongAccountOwner);
    }
    if inputs.feed_id != params.feed_id {
        return Err(GuardError::WrongFeed);
    }
    if inputs.verification_level != VerificationLevel::Full {
        return Err(GuardError::InsufficientVerification);
    }
    check_age(inputs.now, inputs.publish_time, params.max_age_secs)?;
    check_confidence(inputs.conf, inputs.price, params.max_conf_bps)?;

    let effective_ref_price = if params.reference_invert {
        if inputs.ref_price == 0 {
            return Err(GuardError::ReferenceInvalid);
        }
        let scaled_one = 10_000_000_000_000_000u128; // 10^16 scaling (1.0 with 8 decimals squared)
        let inverted = scaled_one
            .checked_div(inputs.ref_price as u128)
            .ok_or(GuardError::MathOverflow)?;
        u64::try_from(inverted).map_err(|_| GuardError::MathOverflow)?
    } else {
        inputs.ref_price
    };

    let measured_bps =
        check_deviation(inputs.price, effective_ref_price, params.max_deviation_bps)?;
    Ok(measured_bps)
}

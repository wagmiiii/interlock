use crate::errors::GuardError;

/// Maximum permissible future clock drift in seconds (5s).
pub const MAX_FUTURE_CLOCK_DRIFT_SECS: u64 = 5;

/// Verify oracle price staleness.
pub fn check_age(now: u64, publish_time: u64, max_age_secs: u64) -> Result<(), GuardError> {
    if publish_time > now {
        let drift = publish_time
            .checked_sub(now)
            .ok_or(GuardError::MathOverflow)?;
        if drift > MAX_FUTURE_CLOCK_DRIFT_SECS {
            return Err(GuardError::StalePrice);
        }
        return Ok(());
    }
    let age = now
        .checked_sub(publish_time)
        .ok_or(GuardError::MathOverflow)?;
    if age > max_age_secs {
        return Err(GuardError::StalePrice);
    }
    Ok(())
}

/// Verify oracle confidence interval relative to price.
///
/// Formula: fail if `conf * 10_000 > max_conf_bps * p_oracle`
pub fn check_confidence(conf: u64, price: u64, max_conf_bps: u16) -> Result<(), GuardError> {
    let conf_scaled = (conf as u128)
        .checked_mul(10_000)
        .ok_or(GuardError::MathOverflow)?;
    let threshold = (max_conf_bps as u128)
        .checked_mul(price as u128)
        .ok_or(GuardError::MathOverflow)?;

    if conf_scaled > threshold {
        return Err(GuardError::ConfidenceTooWide);
    }
    Ok(())
}

/// Verify deviation between primary oracle and independent reference price.
///
/// Formula: fail if `|p_oracle - p_ref| * 10_000 > max_deviation_bps * p_ref`
pub fn check_deviation(
    p_oracle: u64,
    p_ref: u64,
    max_deviation_bps: u16,
) -> Result<u16, GuardError> {
    if p_ref == 0 {
        return Err(GuardError::ReferenceInvalid);
    }

    let diff = (p_oracle as i128)
        .checked_sub(p_ref as i128)
        .ok_or(GuardError::MathOverflow)?
        .unsigned_abs();

    let diff_scaled = diff.checked_mul(10_000).ok_or(GuardError::MathOverflow)?;

    let threshold = (max_deviation_bps as u128)
        .checked_mul(p_ref as u128)
        .ok_or(GuardError::MathOverflow)?;

    if diff_scaled > threshold {
        return Err(GuardError::DeviationExceeded);
    }

    // Measured deviation in basis points
    let measured_bps = (diff_scaled / (p_ref as u128)) as u16;
    Ok(measured_bps)
}

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone, Copy)]
pub enum GuardError {
    #[error("Oracle price is stale")]
    StalePrice,
    #[error("Confidence interval too wide relative to price")]
    ConfidenceTooWide,
    #[error("Price update account does not match expected feed id")]
    WrongFeed,
    #[error("Price update verification level is insufficient (requires full verification)")]
    InsufficientVerification,
    #[error("Reference price source is invalid or unreadable")]
    ReferenceInvalid,
    #[error("Oracle deviation from reference price exceeded maximum limit")]
    DeviationExceeded,
    #[error("Arithmetic overflow during cross-multiplication or scaling")]
    MathOverflow,
    #[error("Invalid owner for price or reference account")]
    WrongAccountOwner,
}

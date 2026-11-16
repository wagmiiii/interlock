use interlock_ix::{GuardError, OracleGuardParams};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TxCompositionError {
    #[error("Invalid instruction order: Interlock guard must precede venue trade instructions in transaction")]
    InvalidInstructionOrder,
    #[error("Transaction is missing the required Interlock assert_oracle_sane instruction")]
    MissingGuardInstruction,
    #[error("Transaction exceeds maximum recommended instruction count: {0}")]
    InstructionCountExceeded(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionType {
    ComputeBudget,
    OraclePriceUpdate,
    InterlockGuard(OracleGuardParams),
    VenueTradeInstruction { name: String },
}

/// Transaction composition helper ensuring strict safety ordering.
///
/// Order required:
/// `[ComputeBudget] [PriceUpdate] [InterlockGuard] [VenueTradeInstruction]`
#[derive(Debug, Default)]
pub struct TransactionComposer {
    instructions: Vec<InstructionType>,
    has_guard: bool,
    has_venue_trade: bool,
}

impl TransactionComposer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compute budget instruction.
    pub fn add_compute_budget(&mut self) -> Result<&mut Self, TxCompositionError> {
        if self.has_guard || self.has_venue_trade {
            return Err(TxCompositionError::InvalidInstructionOrder);
        }
        self.instructions.push(InstructionType::ComputeBudget);
        Ok(self)
    }

    /// Add an oracle price update instruction (e.g. Pyth pull update).
    pub fn add_oracle_price_update(&mut self) -> Result<&mut Self, TxCompositionError> {
        if self.has_guard || self.has_venue_trade {
            return Err(TxCompositionError::InvalidInstructionOrder);
        }
        self.instructions.push(InstructionType::OraclePriceUpdate);
        Ok(self)
    }

    /// Add the Interlock assert_oracle_sane guard instruction.
    pub fn add_guard(
        &mut self,
        params: OracleGuardParams,
    ) -> Result<&mut Self, TxCompositionError> {
        if self.has_guard || self.has_venue_trade {
            // Guard must NEVER come after the trade instruction, nor be duplicated
            return Err(TxCompositionError::InvalidInstructionOrder);
        }
        self.instructions
            .push(InstructionType::InterlockGuard(params));
        self.has_guard = true;
        Ok(self)
    }

    /// Add a venue trade instruction (e.g. open/increase/swap).
    pub fn add_venue_trade(
        &mut self,
        name: impl Into<String>,
    ) -> Result<&mut Self, TxCompositionError> {
        if !self.has_guard {
            return Err(TxCompositionError::MissingGuardInstruction);
        }
        self.instructions
            .push(InstructionType::VenueTradeInstruction { name: name.into() });
        self.has_venue_trade = true;
        Ok(self)
    }

    /// Validate and build ordered instruction list.
    pub fn build(&self) -> Result<&[InstructionType], TxCompositionError> {
        if !self.has_guard {
            return Err(TxCompositionError::MissingGuardInstruction);
        }
        Ok(&self.instructions)
    }

    /// Map on-chain GuardError to a clear user-facing diagnostic string.
    pub fn map_error_to_diagnostic(err: GuardError) -> &'static str {
        match err {
            GuardError::StalePrice => {
                "Transaction aborted: Oracle price is stale and exceeds max age limit."
            }
            GuardError::ConfidenceTooWide => {
                "Transaction aborted: Oracle confidence interval is too wide (high price uncertainty)."
            }
            GuardError::WrongFeed => {
                "Transaction aborted: Price feed account id does not match target asset feed."
            }
            GuardError::InsufficientVerification => {
                "Transaction aborted: Price update has not passed full cryptographic verification."
            }
            GuardError::ReferenceInvalid => {
                "Transaction aborted: Independent reference price source is unreadable or zero."
            }
            GuardError::DeviationExceeded => {
                "Transaction aborted: Oracle price deviates excessively from independent reference."
            }
            GuardError::MathOverflow => {
                "Transaction aborted: Arithmetic overflow in on-chain price validation math."
            }
            GuardError::WrongAccountOwner => {
                "Transaction aborted: Price or reference account owner does not match expected program id."
            }
        }
    }
}

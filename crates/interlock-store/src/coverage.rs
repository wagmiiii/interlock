use interlock_core::{CoverageReport, MarketId};
use rusqlite::{params, Connection, Result};
use rust_decimal::Decimal;

pub struct CoverageQuery<'a> {
    pub conn: &'a Connection,
}

impl<'a> CoverageQuery<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Calculate daily coverage percentage and report gaps longer than 5 minutes.
    pub fn query_coverage(
        &self,
        market: &MarketId,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<CoverageReport> {
        let mut stmt = self.conn.prepare(
            "SELECT unix_ts FROM snapshots WHERE market = ?1 AND unix_ts BETWEEN ?2 AND ?3 ORDER BY unix_ts ASC",
        )?;
        let timestamps: Vec<i64> = stmt
            .query_map(params![market.0, from_ts, to_ts], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut gaps = Vec::new();
        for window in timestamps.windows(2) {
            let diff = window[1] - window[0];
            if diff > 300 {
                // > 5 minutes
                gaps.push((window[0], window[1]));
            }
        }

        let (effective_start, effective_end) = if from_ts == 0 && to_ts == i64::MAX {
            if let (Some(&first), Some(&last)) = (timestamps.first(), timestamps.last()) {
                (first, last)
            } else {
                (0, 0)
            }
        } else {
            (from_ts, to_ts)
        };

        let total_slots = timestamps.len();
        let duration = effective_end.saturating_sub(effective_start).max(1);
        let expected_samples = ((duration / 60) as usize).max(1);
        let coverage_pct = if total_slots > 0 {
            (Decimal::from(total_slots) / Decimal::from(expected_samples) * Decimal::from(100))
                .min(Decimal::from(100))
        } else {
            Decimal::ZERO
        };

        Ok(CoverageReport {
            market: market.clone(),
            total_slots,
            start_ts: effective_start,
            end_ts: effective_end,
            coverage_pct,
            gaps_over_five_minutes: gaps,
        })
    }
}

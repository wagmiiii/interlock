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
        let mut rows = stmt.query(params![market.0, from_ts, to_ts])?;
        let mut gaps = Vec::new();
        let mut prev_ts: Option<i64> = None;
        let mut first_ts: Option<i64> = None;
        let mut last_ts: Option<i64> = None;
        let mut total_slots: usize = 0;

        while let Some(row) = rows.next()? {
            let ts: i64 = row.get(0)?;
            total_slots += 1;
            if first_ts.is_none() {
                first_ts = Some(ts);
            }
            if let Some(prev) = prev_ts {
                if ts.saturating_sub(prev) > 300 {
                    gaps.push((prev, ts));
                }
            }
            prev_ts = Some(ts);
            last_ts = Some(ts);
        }

        let (effective_start, effective_end) = if from_ts == 0 && to_ts == i64::MAX {
            (first_ts.unwrap_or(0), last_ts.unwrap_or(0))
        } else {
            (from_ts, to_ts)
        };
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

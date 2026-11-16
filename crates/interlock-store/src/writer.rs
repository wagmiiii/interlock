use interlock_core::MarketSnapshot;
use rusqlite::{params, Connection, Result};

pub struct SnapshotWriter<'a> {
    pub conn: &'a mut Connection,
}

impl<'a> SnapshotWriter<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    /// Insert market snapshot in a single transaction.
    pub fn insert_snapshot(&mut self, snapshot: &MarketSnapshot) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            r#"
            INSERT OR IGNORE INTO snapshots (
                slot, unix_ts, market, mark_price, oracle_price, oracle_conf,
                utilization_bps, borrow_rate_hr, open_fee_bps, close_fee_bps, rpc_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                snapshot.slot as i64,
                snapshot.unix_ts,
                snapshot.market.0,
                snapshot.mark_price.to_string(),
                snapshot.oracle_price.to_string(),
                snapshot.oracle_conf.to_string(),
                snapshot.utilization_bps,
                snapshot.venue.borrow_rate_per_hour.to_string(),
                snapshot.venue.open_fee_bps.to_string(),
                snapshot.venue.close_fee_bps.to_string(),
                snapshot.rpc_id,
            ],
        )?;
        tx.commit()
    }
}

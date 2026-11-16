use rusqlite::{Connection, Result};

pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS snapshots (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  slot             INTEGER NOT NULL,
  unix_ts          INTEGER NOT NULL,
  market           TEXT    NOT NULL,
  mark_price       TEXT    NOT NULL,
  oracle_price     TEXT    NOT NULL,
  oracle_conf      TEXT    NOT NULL,
  utilization_bps  INTEGER NOT NULL,
  borrow_rate_hr   TEXT    NOT NULL,
  open_fee_bps     TEXT    NOT NULL,
  close_fee_bps    TEXT    NOT NULL,
  rpc_id           TEXT    NOT NULL,
  UNIQUE (market, slot)
);

CREATE INDEX IF NOT EXISTS idx_snapshots_market_ts ON snapshots (market, unix_ts);

CREATE TABLE IF NOT EXISTS logger_runs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  started_ts  INTEGER NOT NULL,
  stopped_ts  INTEGER,
  note        TEXT
);
"#;

pub fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)
}

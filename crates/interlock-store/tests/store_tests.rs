use interlock_core::{MarketId, MarketSnapshot, VenueParams};
use interlock_store::{initialize_database, CoverageQuery, SnapshotWriter};
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn mock_snapshot(slot: u64, unix_ts: i64) -> MarketSnapshot {
    MarketSnapshot {
        slot,
        unix_ts,
        market: MarketId::from("SOL-PERP"),
        mark_price: dec!(150.0),
        oracle_price: dec!(150.05),
        oracle_conf: dec!(0.10),
        utilization_bps: 6500,
        venue: VenueParams {
            open_fee_bps: dec!(6),
            close_fee_bps: dec!(6),
            borrow_rate_per_hour: dec!(0.00001),
            max_leverage: dec!(20),
            maintenance_margin: dec!(0.02),
            impact: None,
        },
        rpc_id: "orbitflare-primary".to_string(),
    }
}

#[test]
fn test_database_initialization_and_snapshot_writes() {
    let mut conn = Connection::open_in_memory().unwrap();
    initialize_database(&conn).unwrap();

    let mut writer = SnapshotWriter::new(&mut conn);
    let snap1 = mock_snapshot(300_000_000, 1731760000);
    writer.insert_snapshot(&snap1).unwrap();

    // Duplicate insert on (market, slot) must be ignored without failing
    writer.insert_snapshot(&snap1).unwrap();

    let count: i64 = conn
        .query_row("SELECT count(*) FROM snapshots", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_coverage_and_gap_detection() {
    let mut conn = Connection::open_in_memory().unwrap();
    initialize_database(&conn).unwrap();

    let mut writer = SnapshotWriter::new(&mut conn);
    // Insert snapshots: normal interval is 60s
    writer.insert_snapshot(&mock_snapshot(1, 1000)).unwrap();
    writer.insert_snapshot(&mock_snapshot(2, 1060)).unwrap();
    writer.insert_snapshot(&mock_snapshot(3, 1120)).unwrap();
    // 600s gap (> 300s = 5 minutes)
    writer.insert_snapshot(&mock_snapshot(4, 1720)).unwrap();
    writer.insert_snapshot(&mock_snapshot(5, 1780)).unwrap();

    let query = CoverageQuery::new(&conn);
    let report = query
        .query_coverage(&MarketId::from("SOL-PERP"), 1000, 1800)
        .unwrap();

    assert_eq!(report.total_slots, 5);
    // Gap over 5 minutes detected
    assert_eq!(report.gaps_over_five_minutes.len(), 1);
    assert_eq!(report.gaps_over_five_minutes[0], (1120, 1720));
    assert!(report.coverage_pct > Decimal::ZERO);

    // Unbounded window (0, i64::MAX) must calculate coverage based on recorded bounds, not 0.00%
    let unbounded_report = query
        .query_coverage(&MarketId::from("SOL-PERP"), 0, i64::MAX)
        .unwrap();
    assert_eq!(unbounded_report.total_slots, 5);
    assert_eq!(unbounded_report.start_ts, 1000);
    assert_eq!(unbounded_report.end_ts, 1780);
    assert!(unbounded_report.coverage_pct > Decimal::ZERO);
}

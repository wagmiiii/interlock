use interlock_chain::{
    DecoderError, PythFixtureRecord, PythReader, VenueAccountRecord, VenueDecoder,
    VENUE_POOL_DISCRIMINATOR,
};
use interlock_core::MarketId;
use rust_decimal_macros::dec;

#[test]
fn test_decode_venue_fixture() {
    let fixture_str = include_str!("fixtures/venue_account_sol_300000000.json");
    let record: VenueAccountRecord = serde_json::from_str(fixture_str).unwrap();

    let params = VenueDecoder::decode_record(&record).unwrap();
    assert_eq!(params.open_fee_bps, dec!(6));
    assert_eq!(params.close_fee_bps, dec!(6));
    assert_eq!(params.borrow_rate_per_hour, dec!(0.00001000));
    assert_eq!(params.max_leverage, dec!(20));
    assert_eq!(params.maintenance_margin, dec!(0.02));
}

#[test]
fn test_decode_venue_bytes() {
    let mut data = Vec::new();
    data.extend_from_slice(&VENUE_POOL_DISCRIMINATOR); // 8 bytes
    data.extend_from_slice(&6u32.to_le_bytes()); // open fee bps
    data.extend_from_slice(&6u32.to_le_bytes()); // close fee bps
    data.extend_from_slice(&20u32.to_le_bytes()); // max leverage
    data.extend_from_slice(&200u32.to_le_bytes()); // maintenance margin bps
    data.extend_from_slice(&1000u64.to_le_bytes()); // borrow rate scaled

    let params = VenueDecoder::decode_bytes(&MarketId::from("SOL-PERP"), &data).unwrap();
    assert_eq!(params.open_fee_bps, dec!(6));
    assert_eq!(params.close_fee_bps, dec!(6));
    assert_eq!(params.max_leverage, dec!(20));
    assert_eq!(params.maintenance_margin, dec!(0.02));
    assert_eq!(params.borrow_rate_per_hour, dec!(0.00001000));
}

#[test]
fn test_decode_venue_wrong_discriminator() {
    let fake_data = vec![0u8; 40];
    let res = VenueDecoder::decode_bytes(&MarketId::from("SOL-PERP"), &fake_data);
    assert!(matches!(
        res,
        Err(DecoderError::InvalidDiscriminator { .. })
    ));
}

#[test]
fn test_decode_venue_truncated_data() {
    let truncated = vec![168, 204];
    let res = VenueDecoder::decode_bytes(&MarketId::from("SOL-PERP"), &truncated);
    assert!(matches!(res, Err(DecoderError::TruncatedData(..))));
}

#[test]
fn test_decode_pyth_fixture() {
    let fixture_str = include_str!("fixtures/pyth_sol_price_300000000.json");
    let record: PythFixtureRecord = serde_json::from_str(fixture_str).unwrap();

    let pyth_price = PythReader::parse_fixture(&record).unwrap();
    assert_eq!(pyth_price.price, dec!(150.00000000));
    assert_eq!(pyth_price.conf, dec!(0.15000000));
    assert_eq!(pyth_price.expo, -8);
}

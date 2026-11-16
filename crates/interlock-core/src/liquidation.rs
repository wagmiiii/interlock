use crate::types::Side;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Estimate liquidation price and distance percentage from mark price.
///
/// Formula placeholder per ARCHITECTURE.md 6.2:
/// - Long:  `liq_price ~= entry * (1 - 1/leverage + maintenance_margin)`
/// - Short: `liq_price ~= entry * (1 + 1/leverage - maintenance_margin)`
pub fn estimate_liquidation_price(
    side: Side,
    mark_price: Decimal,
    leverage: Decimal,
    maintenance_margin: Decimal,
) -> (Decimal, Decimal) {
    if leverage <= dec!(0) || mark_price <= dec!(0) {
        return (dec!(0), dec!(0));
    }

    let inv_leverage = dec!(1) / leverage;

    let liq_price = match side {
        Side::Long => {
            let factor = dec!(1) - inv_leverage + maintenance_margin;
            (mark_price * factor).max(dec!(0))
        }
        Side::Short => {
            let factor = dec!(1) + inv_leverage - maintenance_margin;
            (mark_price * factor).max(dec!(0))
        }
    };

    let distance_pct = if mark_price > dec!(0) {
        ((liq_price - mark_price).abs() / mark_price) * dec!(100)
    } else {
        dec!(0)
    };

    (liq_price, distance_pct)
}

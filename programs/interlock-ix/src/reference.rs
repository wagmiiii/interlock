#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    /// Secondary Pyth pull-feed
    PythSecondFeed,
    /// On-chain DEX pool spot price (e.g. Raydium/Orca)
    DexPool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePrice {
    pub price: u64,
    pub expo: i32,
}

pub struct RollupId(String);

impl RollupId {
    pub fn new(id: String) -> Self {
        RollupId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for RollupId {
    fn from(s: String) -> Self {
        RollupId(s)
    }
}

impl From<&str> for RollupId {
    fn from(s: &str) -> Self {
        RollupId(s.to_string())
    }
}

impl std::fmt::Display for RollupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for RollupId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RollupId {}

impl std::hash::Hash for RollupId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub const ZKSYNC_ERA: &str = "zksync-era";
pub const POLYGON_ZKEVM: &str = "polygon-zkevm";
pub const SCROLL: &str = "scroll";
pub const ARBITRUM_ONE: &str = "arbitrum-one";
pub const ARBITRUM_NOVA: &str = "arbitrum-nova";
pub const STARKNET: &str = "starknet";
pub const TAIKO: &str = "taiko";
pub const LINEA: &str = "linea";
pub const RISC_ZERO: &str = "risc-zero";
pub const ZKLINK: &str = "zklink";
pub mod helpers;
#[allow(clippy::module_inception)]
pub mod transaction;

use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cluster {
    Devnet,
    Mainnet,
}

impl FromStr for Cluster {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "devnet" => Ok(Cluster::Devnet),
            "mainnet" => Ok(Cluster::Mainnet),
            _ => Err("Invalid cluster. Must be 'mainnet' or 'devnet'".to_string()),
        }
    }
}

impl Cluster {
    pub fn to_network_id(&self) -> i32 {
        match self {
            Cluster::Devnet => 0,
            Cluster::Mainnet => 1,
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cluster::Devnet => write!(f, "devnet"),
            Cluster::Mainnet => write!(f, "mainnet"),
        }
    }
}

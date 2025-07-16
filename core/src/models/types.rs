use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingEventStatus {
    Confirmed,
    Pending,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Sandbox,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferDirection {
    Inflow,
    Outflow,
}

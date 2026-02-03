use serde::{Deserialize, Serialize};

// What the user sends (transaction digest)
#[derive(Debug, Deserialize)]
pub struct ExplainRequest {
    pub digest: String,
}

// What is returned to the user
#[derive(Debug, Serialize)]
pub struct ExplainResponse {
    pub success: bool,
    pub explanation: Option<TransactionExplanation>,
    pub error: Option<String>, //Display error if transaction fails
}

// Explanation of the transaction, including its effects and any relevant details
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TransactionExplanation {
    pub digest: String,
    pub sender: String,
    pub status: String,
    pub gas_used: u64,        //Total gas used in MIST (1 SUI = 1,000,000,000 MIST)
    pub gas_used_sui: String, //Total gas used in SUI, simple and more readable
    pub actions: Vec<String>,
    pub object_changes: Vec<ObjectMod>,
    pub balance_changes: Vec<BalanceChange>,
    pub events: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectMod {
    pub change_type: String,
    pub object_type: String,
    pub object_id: String,
    pub owner: Option<String>,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceChange {
    pub owner: String,
    pub coin_type: String,
    pub amount: i128, //Using signed integer here because there's two considered BalanceChange (Sent, Received)
    pub amount_readable: String,
}

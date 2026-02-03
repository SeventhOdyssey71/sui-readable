use anyhow::{Context, Result};
use sui_json_rpc_types::{
    BalanceChange as SuiBalanceChange, ObjectChange, SuiTransactionBlockDataAPI,
    SuiTransactionBlockEffectsAPI, SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions,
};
use sui_sdk::SuiClientBuilder;

use crate::models::{BalanceChange, ObjectMod as ModelObjectChange, TransactionExplanation};

pub struct SuiClient {
    client: sui_sdk::SuiClient,
}

impl SuiClient {
    //Create a new Sui client connected to mainnet
    pub async fn new() -> Result<Self> {
        //Connect to Sui mainnet RPC endpoint
        let client = SuiClientBuilder::default()
            .build("https://fullnode.mainnet.sui.io:443")
            .await
            .context("Failed to build Sui client")?;

        Ok(Self { client })
    }

    //Fetch and explain a transaction by its digest (hash)
    pub async fn explain_transaction(&self, digest: &str) -> Result<TransactionExplanation> {
        //Parse the digest string into a proper type
        let tx_digest = digest
            .parse()
            .context("Invalid transaction digest format")?;

        //Fetch the transaction with all details
        let tx_response = self
            .client
            .read_api()
            .get_transaction_with_options(
                tx_digest,
                SuiTransactionBlockResponseOptions {
                    show_input: true,
                    show_effects: true,
                    show_events: true,
                    show_object_changes: true,
                    show_balance_changes: true,
                    ..Default::default()
                },
            )
            .await
            .context("Failed to fetch transaction from Sui")?;

        self.parse_transaction(digest, tx_response)
    }

    // Convert the raw Sui response into our human-readable format
    fn parse_transaction(
        &self,
        digest: &str,
        tx: SuiTransactionBlockResponse,
    ) -> Result<TransactionExplanation> {
        let mut explanation = TransactionExplanation {
            digest: digest.to_string(),
            ..Default::default()
        };

        if let Some(tx_data) = &tx.transaction {
            explanation.sender = tx_data.data.sender().to_string();
        }

        if let Some(effects) = &tx.effects {
            // Check if transaction succeeded or failed
            explanation.status = if effects.status().is_ok() {
                "Success".to_string()
            } else {
                format!("Failed : {:?}", effects.status())
            };

            // Calculate total gas used
            let gas_used = effects.gas_cost_summary();
            explanation.gas_used =
                gas_used.computation_cost + gas_used.storage_cost - gas_used.storage_rebate;

            // Convert MIST to SUI (1 SUI = 1,000,000,000 MIST)
            let sui_amount = explanation.gas_used as f64 / 1_000_000_000.0;
            explanation.gas_used_sui = format!("{:.6} SUI", sui_amount);
        }

        if let Some(changes) = &tx.object_changes {
            for change in changes {
                let obj_change = self.parse_object_change(change);
                explanation.actions.push(obj_change.details.clone());
                explanation.object_changes.push(obj_change);
            }
        }

        if let Some(balances) = &tx.balance_changes {
            for balance in balances {
                let bal_change = self.parse_balance_change(balance);
                explanation.balance_changes.push(bal_change);
            }
        }

        if let Some(events) = &tx.events {
            for event in &events.data {
                explanation.events.push(format!(
                    "Event: {} from package {}",
                    self.simplify_type(&event.type_.to_string()),
                    event.package_id
                ));
            }
        }

        explanation.summary = self.generate_summary(&explanation);

        Ok(explanation)
    }

    fn parse_object_change(&self, change: &ObjectChange) -> ModelObjectChange {
        match change {
            ObjectChange::Created {
                object_id,
                object_type,
                owner,
                ..
            } => ModelObjectChange {
                change_type: "Created".to_string(),
                object_type: self.simplify_type(&object_type.to_string()),
                object_id: object_id.to_string(),
                owner: Some(owner.to_string()),
                details: format!(
                    "Created new {} owned by {}",
                    self.simplify_type(&object_type.to_string()),
                    self.shorten_address(&owner.to_string())
                ),
            },
            ObjectChange::Transferred {
                object_id,
                object_type,
                sender,
                recipient,
                ..
            } => ModelObjectChange {
                change_type: "Transferred".to_string(),
                object_type: self.simplify_type(&object_type.to_string()),
                object_id: object_id.to_string(),
                owner: Some(recipient.to_string()),
                details: format!(
                    "Transferred {} from {} to {}",
                    self.simplify_type(&object_type.to_string()),
                    self.shorten_address(&sender.to_string()),
                    self.shorten_address(&recipient.to_string())
                ),
            },
            ObjectChange::Mutated {
                object_id,
                object_type,
                owner,
                ..
            } => ModelObjectChange {
                change_type: "Mutated".to_string(),
                object_type: self.simplify_type(&object_type.to_string()),
                object_id: object_id.to_string(),
                owner: Some(owner.to_string()),
                details: format!(
                    "Modified {} owned by {}",
                    self.simplify_type(&object_type.to_string()),
                    self.shorten_address(&owner.to_string())
                ),
            },
            ObjectChange::Deleted {
                object_id,
                object_type,
                ..
            } => ModelObjectChange {
                change_type: "Deleted".to_string(),
                object_type: self.simplify_type(&object_type.to_string()),
                object_id: object_id.to_string(),
                owner: None,
                details: format!("Deleted {}", self.simplify_type(&object_type.to_string())),
            },
            _ => ModelObjectChange {
                change_type: "Unknown".to_string(),
                object_type: "Unknown".to_string(),
                object_id: "Unknown".to_string(),
                owner: None,
                details: "Unknown object change".to_string(),
            },
        }
    }

    // Convert a SuiBalanceChange into our BalanceChange format
    fn parse_balance_change(&self, balance: &SuiBalanceChange) -> BalanceChange {
        let amount = balance.amount;
        let coin_type = self.simplify_type(&balance.coin_type.to_string());

        // Convert to human-readable format
        let amount_readable = if coin_type.contains("SUI") {
            let sui_amount = amount as f64 / 1_000_000_000.0;
            format!("{:+.6} SUI", sui_amount)
        } else {
            format!("{:+}", amount)
        };

        BalanceChange {
            owner: balance.owner.to_string(),
            coin_type,
            amount,
            amount_readable,
        }
    }

    //Simplify long type names ("0x2::coin::Coin<0x2::sui::SUI>" -> "SUI Coin")
    fn simplify_type(&self, type_str: &str) -> String {
        if type_str.contains("0x2::sui::SUI") {
            return "SUI Coin".to_string();
        }
        if type_str.contains("::coin::Coin") {
            return "Coin".to_string();
        }
        if type_str.contains("::nft::") {
            return "NFT".to_string();
        }

        // Extract just the last part after ::
        type_str.split("::").last().unwrap_or(type_str).to_string()
    }

    //Shorten addresses for readability (0x123...789)
    fn shorten_address(&self, address: &str) -> String {
        if address.len() > 10 {
            format!("{}...{}", &address[..6], &address[address.len() - 4..])
        } else {
            address.to_string()
        }
    }

    // Generate a one-line summary of what happened
    fn generate_summary(&self, explanation: &TransactionExplanation) -> String {
        let action_count = explanation.actions.len();
        let balance_count = explanation.balance_changes.len();

        if action_count == 0 && balance_count == 0 {
            return format!("Transaction executed with {} gas", explanation.gas_used_sui);
        }

        let mut parts = vec![];

        if action_count > 0 {
            parts.push(format!(
                "{} object change{}",
                action_count,
                if action_count == 1 { "" } else { "s" }
            ));
        }

        if balance_count > 0 {
            parts.push(format!(
                "{} balance change{}",
                balance_count,
                if balance_count == 1 { "" } else { "s" }
            ));
        }

        format!("{} • Gas: {}", parts.join(" • "), explanation.gas_used_sui)
    }
}

use std::sync::Arc;

use serde_json;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::{contract::legacy::LegacyContractClass, Felt},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use std::error::Error;
use std::fs::File;

use crate::MAX_FEE;

pub async fn declare_legacy(
    path: &str,
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: &mut Felt,
) -> anyhow::Result<Felt> {
    let contract_artifact: LegacyContractClass = serde_json::from_reader(File::open(path)?)?;

    // Déclaration du contrat sur Starknet
    let result = account
        .declare_legacy(Arc::new(contract_artifact))
        .max_fee(MAX_FEE)
        .nonce(*nonce)
        .send()
        .await?;

    *nonce += Felt::ONE;

    Ok(result.class_hash)
}

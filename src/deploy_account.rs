use starknet::{
    accounts::{AccountFactory, OpenZeppelinAccountFactory},
    core::types::Felt,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use std::error::Error;
use std::sync::Arc;

use crate::MAX_FEE;

pub fn get_address(
    account_factory: &OpenZeppelinAccountFactory<&LocalWallet, &Arc<JsonRpcClient<HttpTransport>>>,
    salt: Felt,
) -> Felt {
    let deploy = account_factory
        .deploy_v1(salt)
        .max_fee(MAX_FEE)
        .nonce(Felt::ZERO);

    deploy.address()
}

/// Déploie un compte Starknet
pub async fn deploy_account(
    account_factory: &OpenZeppelinAccountFactory<&LocalWallet, &Arc<JsonRpcClient<HttpTransport>>>,
    salt: Felt,
) -> Result<Felt, Box<dyn Error>> {
    let deploy = account_factory
        .deploy_v1(salt)
        .max_fee(MAX_FEE)
        .nonce(Felt::ZERO);

    let result = deploy.send().await?;

    Ok(result.contract_address)
}

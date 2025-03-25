use starknet::{
    accounts::SingleOwnerAccount,
    contract::ContractFactory,
    core::{types::Felt, utils::get_udc_deployed_address},
    macros::selector,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use std::error::Error;
use std::sync::Arc;

use crate::MAX_FEE;

/// Déploie un token ERC20 en utilisant un class hash déjà déclaré
pub async fn deploy_erc20(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: &mut Felt,
    class_hash: Felt,
    name: &str,
    symbol: &str,
    decimals: u8,
    initial_supply: Felt,
    recipient: Felt,
    salt: Felt,
) -> anyhow::Result<Felt> {
    let constructor_calldata = vec![
        Felt::from_bytes_be_slice(name.as_bytes()),
        Felt::from_bytes_be_slice(symbol.as_bytes()),
        Felt::from(decimals),
        initial_supply,
        initial_supply,
        recipient,
    ];

    let contract_address = get_udc_deployed_address(
        salt,
        class_hash,
        &starknet::core::utils::UdcUniqueness::NotUnique,
        &constructor_calldata,
    );

    // Créer la factory de contrat
    let contract_factory = ContractFactory::new(class_hash, account);

    // Déployer le contrat
    let deployment = contract_factory
        .deploy_v1(constructor_calldata, salt, false)
        .max_fee(MAX_FEE)
        .nonce(*nonce)
        .send()
        .await?;

    *nonce += Felt::ONE;

    Ok(contract_address)
}

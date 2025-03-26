use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::{Call, Felt},
    macros::selector,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};

use crate::MAX_FEE;

pub async fn transfer(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: &mut Felt,
    contract_address: &Felt,
    amount: &Felt,
    recipient: &Felt,
) -> anyhow::Result<Felt> {
    let calldata = vec![*recipient, *amount, Felt::ZERO];

    let call = Call {
        to: *contract_address,
        selector: selector!("transfer"),
        calldata,
    };

    let result = account
        .execute_v1(vec![call])
        .max_fee(MAX_FEE)
        .nonce(*nonce)
        .send()
        .await?;

    *nonce += Felt::ONE;

    println!("Transfered {} to 0x{:x}", amount, recipient);

    Ok(result.transaction_hash)
}

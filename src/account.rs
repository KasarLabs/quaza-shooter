use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use serde_json;
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    contract::ContractFactory,
    core::{
        types::{
            contract::{legacy::LegacyContractClass, CompiledClass, SierraClass},
            Call, Felt,
        },
        utils::get_udc_deployed_address,
    },
    macros::selector,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{LocalWallet, SigningKey},
};
use std::fs::File;

use crate::{CHAIN_ID, MAX_FEE};

#[derive(Clone)]
pub struct AccountManager {
    account: Arc<SingleOwnerAccount<Arc<JsonRpcClient<HttpTransport>>, LocalWallet>>,
    nonce: Arc<AtomicU64>,
}

impl AccountManager {
    pub fn new(
        provider: Arc<JsonRpcClient<HttpTransport>>,
        private_key: Felt,
        address: &Felt,
        initial_nonce: u64,
    ) -> Self {
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));
        Self {
            account: Arc::new(SingleOwnerAccount::new(
                provider,
                signer,
                *address,
                CHAIN_ID,
                ExecutionEncoding::New,
            )),
            nonce: Arc::new(AtomicU64::new(initial_nonce)),
        }
    }

    fn get_and_increment_nonce(&self) -> Felt {
        let current_nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
        Felt::from(current_nonce)
    }

    pub fn decrement_nonce(&self) {
        let current = self.nonce.load(Ordering::SeqCst);
        if current > 0 {
            self.nonce.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub async fn declare_legacy(&self, path: &str) -> anyhow::Result<Felt> {
        let contract_artifact: LegacyContractClass = serde_json::from_reader(File::open(path)?)?;
        let nonce = self.get_and_increment_nonce();

        let result = self
            .account
            .declare_legacy(Arc::new(contract_artifact))
            .max_fee(MAX_FEE)
            .nonce(nonce.into())
            .send()
            .await?;

        Ok(result.class_hash)
    }

    pub async fn declare_v2(&self, path: &str, compiled_path: &str) -> anyhow::Result<Felt> {
        let contract_artifact: SierraClass = serde_json::from_reader(File::open(path)?)?;
        let compiled_class: CompiledClass = serde_json::from_reader(File::open(compiled_path)?)?;
        let compiled_class_hash = compiled_class.class_hash()?;
        let flattened_class = contract_artifact.flatten()?;

        let nonce = self.get_and_increment_nonce();

        let result = self
            .account
            .declare_v2(Arc::new(flattened_class), compiled_class_hash)
            .max_fee(MAX_FEE)
            .nonce(nonce.into())
            .send()
            .await?;

        Ok(result.class_hash)
    }

    pub async fn execute_v1(&self, calls: Vec<Call>) -> anyhow::Result<Felt> {
        let nonce = self.get_and_increment_nonce();

        let result = self
            .account
            .execute_v1(calls)
            .max_fee(MAX_FEE)
            .nonce(nonce.into())
            .send()
            .await?;

        Ok(result.transaction_hash)
    }

    pub async fn deploy_erc20(
        &self,
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

        let contract_factory = ContractFactory::new(class_hash, &*self.account);

        let nonce = self.get_and_increment_nonce();

        let deployment = contract_factory
            .deploy_v1(constructor_calldata, salt, false)
            .max_fee(MAX_FEE)
            .nonce(nonce.into())
            .send()
            .await?;

        Ok(contract_address)
    }

    pub async fn transfer(
        &self,
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

        let nonce = self.get_and_increment_nonce();

        match self
            .account
            .execute_v1(vec![call])
            .max_fee(MAX_FEE)
            .nonce(nonce.into())
            .send()
            .await
        {
            Ok(result) => Ok(result.transaction_hash),
            Err(e) => {
                self.decrement_nonce();
                Err(e.into())
            }
        }
    }

    pub fn get_account(
        &self,
    ) -> Arc<SingleOwnerAccount<Arc<JsonRpcClient<HttpTransport>>, LocalWallet>> {
        self.account.clone()
    }

    pub fn address(&self) -> Felt {
        self.account.address()
    }

    pub fn nonce(&self) -> Felt {
        Felt::from(self.nonce.load(Ordering::SeqCst))
    }

    /// Définit explicitement une nouvelle valeur de nonce
    pub fn increment_nonce(&self, new_nonce: u64) {
        self.nonce.store(new_nonce, Ordering::SeqCst);
    }
}

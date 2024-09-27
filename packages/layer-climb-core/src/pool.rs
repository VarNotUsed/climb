use std::sync::atomic::AtomicU32;

use crate::signing::SigningClient;
use anyhow::{bail, Result};
use deadpool::managed::{Manager, Metrics, RecycleResult};
use layer_climb_address::*;
use layer_climb_config::ChainConfig;

/// Currently this only works with mnemonic phrases
pub struct SigningClientPoolManager {
    mnemonic: String,
    chain_config: ChainConfig,
    derivation_index: AtomicU32,
}

impl SigningClientPoolManager {
    pub fn new_mnemonic(mnemonic: String, chain_config: ChainConfig) -> Self {
        Self {
            mnemonic,
            chain_config,
            derivation_index: AtomicU32::new(0),
        }
    }
}

impl Manager for SigningClientPoolManager {
    type Type = SigningClient;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<SigningClient> {
        let signer: KeySigner = match &self.chain_config.address_kind {
            layer_climb_config::AddrKind::Cosmos { .. } => {
                let index = self
                    .derivation_index
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                KeySigner::new_mnemonic_str(&self.mnemonic, Some(&cosmos_hub_derivation(index)?))?
            }
            layer_climb_config::AddrKind::Eth => {
                bail!("Eth address kind is not supported (yet)")
            }
        };

        let signing_client = SigningClient::new(self.chain_config.clone(), signer).await?;
        Ok(signing_client)
    }

    async fn recycle(&self, _: &mut SigningClient, _: &Metrics) -> RecycleResult<anyhow::Error> {
        Ok(())
    }
}

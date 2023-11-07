// Copyright (c) 2023 by Alibaba.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use anyhow::*;
use async_trait::async_trait;
use attestation_service::policy_engine::PolicyDigest;
use kbs_types::Tee;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "coco-as")]
mod coco;

#[cfg(feature = "amber-as")]
pub mod amber;

/// Interface for Attestation Services.
///
/// Attestation Service implementations should implement this interface.
#[async_trait]
pub trait Attest: Send + Sync {
    /// Set Attestation Policy
    async fn set_policy(&mut self, _input: as_types::SetPolicyInput) -> Result<()> {
        Err(anyhow!("Set Policy API is unimplemented"))
    }

    async fn remove_policy(&mut self, _policy_id: String) -> Result<()> {
        bail!("Remove Policy API is unimplemented")
    }

    async fn list_policy(&self) -> Result<Vec<PolicyDigest>> {
        bail!("Remove Policy API is unimplemented")
    }

    /// Verify Attestation Evidence
    /// Return Attestation Results Token
    async fn verify(&mut self, tee: Tee, nonce: &str, attestation: &str) -> Result<String>;

    async fn simple_verify(
        &mut self,
        _tee: Tee,
        _evidence: &str,
        _policy_id: Option<String>,
    ) -> Result<String> {
        bail!("Unimplement");
    }
}

/// Attestation Service
#[derive(Clone)]
pub struct AttestationService(pub Arc<Mutex<dyn Attest>>);

impl AttestationService {
    /// Create and initialize AttestionService
    pub async fn new(kbs_config: &Config) -> Result<Self> {
        let attestation_service: Arc<Mutex<dyn Attest>> = {
            cfg_if::cfg_if! {
                if #[cfg(any(feature = "coco-as-builtin", feature = "coco-as-builtin-no-verifier"))] {
                    Arc::new(Mutex::new(coco::builtin::Native::new(&kbs_config.as_config_file_path)?))
                } else if #[cfg(feature = "coco-as-grpc")] {
                    Arc::new(Mutex::new(coco::grpc::Grpc::new(kbs_config).await?))
                } else if #[cfg(feature = "amber-as")] {
                    Arc::new(Mutex::new(amber::Amber::new(&kbs_config.amber)?))
                } else {
                    compile_error!("Please enable at least one of the following features: `coco-as-builtin`, `coco-as-builtin-no-verifier`, `coco-as-grpc` or `amber-as` to continue.");
                }
            }
        };

        Ok(Self(attestation_service))
    }
}

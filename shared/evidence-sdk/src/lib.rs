use std::future::{Future, ready};
use std::sync::{Arc, Mutex};

use contracts::EvidenceManifestV1;
use error_model::{InstitutionalError, InstitutionalResult, OperationContext, SourceErrorInfo};

pub trait EvidenceSink {
    fn record(
        &self,
        manifest: EvidenceManifestV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
    fn recorded(
        &self,
    ) -> impl Future<Output = InstitutionalResult<Vec<EvidenceManifestV1>>> + Send + '_;
}

#[derive(Debug, Default, Clone)]
pub struct MemoryEvidenceSink {
    manifests: Arc<Mutex<Vec<EvidenceManifestV1>>>,
}

impl EvidenceSink for MemoryEvidenceSink {
    fn record(
        &self,
        manifest: EvidenceManifestV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        let result = self
            .manifests
            .lock()
            .map_err(|error| {
                InstitutionalError::persistence(
                    OperationContext::new("shared/evidence-sdk", "record"),
                    "failed to acquire evidence sink lock",
                    SourceErrorInfo::new("std::sync::Mutex", None, error.to_string()),
                )
            })
            .map(|mut manifests| {
                manifests.push(manifest);
            });
        ready(result)
    }

    fn recorded(
        &self,
    ) -> impl Future<Output = InstitutionalResult<Vec<EvidenceManifestV1>>> + Send + '_ {
        let result = self
            .manifests
            .lock()
            .map_err(|error| {
                InstitutionalError::persistence(
                    OperationContext::new("shared/evidence-sdk", "recorded"),
                    "failed to acquire evidence sink lock",
                    SourceErrorInfo::new("std::sync::Mutex", None, error.to_string()),
                )
            })
            .map(|manifests| manifests.clone());
        ready(result)
    }
}

impl MemoryEvidenceSink {
    pub fn len(&self) -> InstitutionalResult<usize> {
        Ok(self
            .manifests
            .lock()
            .map_err(|error| {
                InstitutionalError::persistence(
                    OperationContext::new("shared/evidence-sdk", "len"),
                    "failed to acquire evidence sink lock",
                    SourceErrorInfo::new("std::sync::Mutex", None, error.to_string()),
                )
            })?
            .len())
    }

    pub fn is_empty(&self) -> InstitutionalResult<bool> {
        Ok(self.len()? == 0)
    }
}

use contracts::EvidenceManifestV1;
use error_model::InstitutionalResult;

pub trait EvidenceSink {
    fn record(&mut self, manifest: EvidenceManifestV1) -> InstitutionalResult<()>;
    fn recorded(&self) -> Vec<EvidenceManifestV1>;
}

#[derive(Debug, Default, Clone)]
pub struct MemoryEvidenceSink {
    manifests: Vec<EvidenceManifestV1>,
}

impl EvidenceSink for MemoryEvidenceSink {
    fn record(&mut self, manifest: EvidenceManifestV1) -> InstitutionalResult<()> {
        self.manifests.push(manifest);
        Ok(())
    }

    fn recorded(&self) -> Vec<EvidenceManifestV1> {
        self.manifests.clone()
    }
}

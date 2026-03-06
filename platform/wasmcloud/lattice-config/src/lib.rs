use serde::{Deserialize, Serialize};
use wasmcloud_bindings::{SignedComponentRefV1, WasmComponentBindingV1};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RolloutTargetV1 {
    pub environment: String,
    pub namespace: String,
    pub policy_group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LatticeConfigV1 {
    pub lattice_name: String,
    pub rollout: RolloutTargetV1,
    pub components: Vec<WasmComponentBindingV1>,
}

impl LatticeConfigV1 {
    #[must_use]
    pub fn component_refs(&self) -> Vec<&SignedComponentRefV1> {
        self.components
            .iter()
            .map(|component| &component.component)
            .collect()
    }
}

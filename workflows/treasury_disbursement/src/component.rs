use wasmcloud_bindings::{CapabilityBindingV1, SignedComponentRefV1, WasmComponentBindingV1};

pub const COMPONENT_REPOSITORY: &str = "ghcr.io/shortorigin/treasury-disbursement";
pub const DEFAULT_COMPONENT_TAG: &str = "wasm";
pub const DEFAULT_COMPONENT_DIGEST: &str = "sha256:treasury-disbursement";

#[must_use]
pub fn component_binding() -> WasmComponentBindingV1 {
    component_binding_with_artifact(
        format!("{COMPONENT_REPOSITORY}:{DEFAULT_COMPONENT_TAG}"),
        DEFAULT_COMPONENT_DIGEST,
        "prod",
    )
}

#[must_use]
pub fn component_binding_with_artifact(
    component_ref: impl Into<String>,
    digest: impl Into<String>,
    rollout_environment: impl Into<String>,
) -> WasmComponentBindingV1 {
    WasmComponentBindingV1::workflow(
        "treasury-disbursement",
        SignedComponentRefV1 {
            component_ref: component_ref.into(),
            digest: digest.into(),
            signature_ref: None,
        },
        rollout_environment,
        vec![
            "schemas/wit/v1/platform.wit".to_string(),
            "schemas/contracts/v1/treasury-disbursement-v1.json".to_string(),
        ],
        vec![
            CapabilityBindingV1 {
                provider_id: "wasmcloud:keyvalue".to_string(),
                contract_id: "keyvalue".to_string(),
                link_name: "surrealdb".to_string(),
            },
            CapabilityBindingV1 {
                provider_id: "wasmcloud:httpserver".to_string(),
                contract_id: "http".to_string(),
                link_name: "default".to_string(),
            },
        ],
    )
}

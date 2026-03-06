use serde::{Deserialize, Serialize};

pub const WIT_PACKAGE_V1: &str = "shortorigin:platform";
pub const WIT_VERSION_V1: &str = "1.0.0";
pub const SERVICE_COMPONENT_WORLD_V1: &str = "service-component";
pub const WORKFLOW_COMPONENT_WORLD_V1: &str = "workflow-component";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityBindingV1 {
    pub provider_id: String,
    pub contract_id: String,
    pub link_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedComponentRefV1 {
    pub component_ref: String,
    pub digest: String,
    pub signature_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterfaceBindingV1 {
    pub package: String,
    pub world: String,
    pub version: String,
}

impl InterfaceBindingV1 {
    #[must_use]
    pub fn service_world() -> Self {
        Self {
            package: WIT_PACKAGE_V1.to_string(),
            world: SERVICE_COMPONENT_WORLD_V1.to_string(),
            version: WIT_VERSION_V1.to_string(),
        }
    }

    #[must_use]
    pub fn workflow_world() -> Self {
        Self {
            package: WIT_PACKAGE_V1.to_string(),
            world: WORKFLOW_COMPONENT_WORLD_V1.to_string(),
            version: WIT_VERSION_V1.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadKindV1 {
    Service,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmComponentBindingV1 {
    pub workload_name: String,
    pub workload_kind: WorkloadKindV1,
    pub component: SignedComponentRefV1,
    pub rollout_environment: String,
    pub config_schema_refs: Vec<String>,
    pub interfaces: Vec<InterfaceBindingV1>,
    pub required_capabilities: Vec<CapabilityBindingV1>,
}

impl WasmComponentBindingV1 {
    #[must_use]
    pub fn service(
        workload_name: impl Into<String>,
        component: SignedComponentRefV1,
        rollout_environment: impl Into<String>,
        config_schema_refs: Vec<String>,
        required_capabilities: Vec<CapabilityBindingV1>,
    ) -> Self {
        Self {
            workload_name: workload_name.into(),
            workload_kind: WorkloadKindV1::Service,
            component,
            rollout_environment: rollout_environment.into(),
            config_schema_refs,
            interfaces: vec![InterfaceBindingV1::service_world()],
            required_capabilities,
        }
    }

    #[must_use]
    pub fn workflow(
        workload_name: impl Into<String>,
        component: SignedComponentRefV1,
        rollout_environment: impl Into<String>,
        config_schema_refs: Vec<String>,
        required_capabilities: Vec<CapabilityBindingV1>,
    ) -> Self {
        Self {
            workload_name: workload_name.into(),
            workload_kind: WorkloadKindV1::Workflow,
            component,
            rollout_environment: rollout_environment.into(),
            config_schema_refs,
            interfaces: vec![InterfaceBindingV1::workflow_world()],
            required_capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use wit_parser::Resolve;

    use super::{
        InterfaceBindingV1, SERVICE_COMPONENT_WORLD_V1, WIT_PACKAGE_V1, WIT_VERSION_V1,
        WORKFLOW_COMPONENT_WORLD_V1,
    };

    #[test]
    fn wit_package_and_worlds_match_binding_constants() {
        let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../schemas/wit/v1");
        let mut resolve = Resolve::default();
        let (package_id, _) = resolve.push_dir(&wit_dir).expect("parse wit");
        let package = resolve.packages.get(package_id).expect("package");
        let package_ids = [package_id];

        assert_eq!(package.name.namespace, "shortorigin");
        assert_eq!(package.name.name, "platform");
        assert_eq!(
            package.name.version.as_ref().expect("version").to_string(),
            WIT_VERSION_V1
        );

        let service_world = resolve
            .select_world(&package_ids, Some(SERVICE_COMPONENT_WORLD_V1))
            .expect("service world");
        let workflow_world = resolve
            .select_world(&package_ids, Some(WORKFLOW_COMPONENT_WORLD_V1))
            .expect("workflow world");

        assert_eq!(
            resolve.worlds.get(service_world).expect("service").name,
            SERVICE_COMPONENT_WORLD_V1
        );
        assert_eq!(
            resolve.worlds.get(workflow_world).expect("workflow").name,
            WORKFLOW_COMPONENT_WORLD_V1
        );
        assert_eq!(InterfaceBindingV1::service_world().package, WIT_PACKAGE_V1);
    }
}

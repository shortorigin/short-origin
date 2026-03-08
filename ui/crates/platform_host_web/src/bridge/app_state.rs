use platform_host::{AppStateEnvelope, HostResult};

pub(crate) async fn load_app_state_envelope(
    namespace: &str,
) -> HostResult<Option<AppStateEnvelope>> {
    super::interop::load_app_state_envelope(namespace).await
}

pub(crate) async fn save_app_state_envelope(envelope: &AppStateEnvelope) -> HostResult<()> {
    super::interop::save_app_state_envelope(envelope).await
}

pub(crate) async fn delete_app_state(namespace: &str) -> HostResult<()> {
    super::interop::delete_app_state(namespace).await
}

pub(crate) async fn list_app_state_namespaces() -> HostResult<Vec<String>> {
    super::interop::list_app_state_namespaces().await
}

use platform_host::AppStateEnvelope;

pub(crate) async fn load_app_state_envelope(
    namespace: &str,
) -> Result<Option<AppStateEnvelope>, String> {
    super::interop::load_app_state_envelope(namespace).await
}

pub(crate) async fn save_app_state_envelope(envelope: &AppStateEnvelope) -> Result<(), String> {
    super::interop::save_app_state_envelope(envelope).await
}

pub(crate) async fn delete_app_state(namespace: &str) -> Result<(), String> {
    super::interop::delete_app_state(namespace).await
}

pub(crate) async fn list_app_state_namespaces() -> Result<Vec<String>, String> {
    super::interop::list_app_state_namespaces().await
}

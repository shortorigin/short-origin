pub(crate) async fn cache_put_text(cache_name: &str, key: &str, value: &str) -> Result<(), String> {
    super::interop::cache_put_text(cache_name, key, value).await
}

pub(crate) async fn cache_get_text(cache_name: &str, key: &str) -> Result<Option<String>, String> {
    super::interop::cache_get_text(cache_name, key).await
}

pub(crate) async fn cache_delete(cache_name: &str, key: &str) -> Result<(), String> {
    super::interop::cache_delete(cache_name, key).await
}

pub(crate) async fn load_pref(key: &str) -> Result<Option<String>, String> {
    super::interop::load_pref(key).await
}

pub(crate) async fn save_pref(key: &str, raw_json: &str) -> Result<(), String> {
    super::interop::save_pref(key, raw_json).await
}

pub(crate) async fn delete_pref(key: &str) -> Result<(), String> {
    super::interop::delete_pref(key).await
}

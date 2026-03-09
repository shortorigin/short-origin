use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use contracts::KnowledgeDocumentFormatV1;
use error_model::{InstitutionalError, InstitutionalResult};
use memvid_core::{
    AclEnforcementMode, DocumentProcessor, Memvid, PutOptions, SearchRequest, MEMVID_CORE_VERSION,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleDocument {
    pub document_id: String,
    pub title: String,
    pub uri: String,
    pub content: String,
    pub metadata: BTreeMap<String, String>,
    pub search_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleBuildRequest {
    pub capsule_id: String,
    pub documents: Vec<CapsuleDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleBuildResult {
    pub capsule_id: String,
    pub path: PathBuf,
    pub storage_ref: String,
    pub artifact_hash: String,
    pub version: String,
    pub memvid_version: String,
    pub document_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleHandle {
    pub capsule_id: String,
    pub path: PathBuf,
    pub storage_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleSearchRequest {
    pub query: String,
    pub top_k: usize,
    pub snippet_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapsuleSearchHit {
    pub uri: String,
    pub title: Option<String>,
    pub text: String,
    pub score: Option<f32>,
    pub metadata: BTreeMap<String, String>,
}

pub trait KnowledgeMemoryProvider: Send + Sync {
    fn build_capsule(
        &self,
        request: &CapsuleBuildRequest,
    ) -> InstitutionalResult<CapsuleBuildResult>;
    fn open_capsule(&self, capsule_id: &str) -> InstitutionalResult<CapsuleHandle>;
    fn search_capsule(
        &self,
        capsule_id: &str,
        request: &CapsuleSearchRequest,
    ) -> InstitutionalResult<Vec<CapsuleSearchHit>>;
    fn extract_text(
        &self,
        bytes: &[u8],
        format: KnowledgeDocumentFormatV1,
        mime_type: Option<&str>,
    ) -> InstitutionalResult<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemvidMemoryProvider {
    root_dir: PathBuf,
}

impl MemvidMemoryProvider {
    #[must_use]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    #[must_use]
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn capsule_path(&self, capsule_id: &str) -> PathBuf {
        self.root_dir.join(format!("{capsule_id}.mv2"))
    }
}

impl KnowledgeMemoryProvider for MemvidMemoryProvider {
    fn build_capsule(
        &self,
        request: &CapsuleBuildRequest,
    ) -> InstitutionalResult<CapsuleBuildResult> {
        fs::create_dir_all(&self.root_dir).map_err(|error| {
            InstitutionalError::external(
                "filesystem",
                Some("create memory root".to_string()),
                error.to_string(),
            )
        })?;

        let path = self.capsule_path(&request.capsule_id);
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                InstitutionalError::external(
                    "filesystem",
                    Some("replace capsule".to_string()),
                    error.to_string(),
                )
            })?;
        }

        let mut memvid = Memvid::create(&path).map_err(|error| {
            InstitutionalError::external("memvid", Some("create".to_string()), error.to_string())
        })?;
        for document in &request.documents {
            let mut builder = PutOptions::builder()
                .title(document.title.clone())
                .uri(document.uri.clone())
                .kind("knowledge_source")
                .auto_tag(false)
                .extract_dates(false)
                .extract_triplets(false)
                .enable_embedding(false);
            if let Some(search_text) = &document.search_text {
                builder = builder.search_text(search_text.clone());
            }
            for (key, value) in &document.metadata {
                builder = builder.tag(key.clone(), value.clone());
            }
            memvid
                .put_bytes_with_options(document.content.as_bytes(), builder.build())
                .map_err(|error| {
                    InstitutionalError::external(
                        "memvid",
                        Some("put_bytes".to_string()),
                        error.to_string(),
                    )
                })?;
        }
        memvid.commit().map_err(|error| {
            InstitutionalError::external("memvid", Some("commit".to_string()), error.to_string())
        })?;

        let bytes = fs::read(&path).map_err(|error| {
            InstitutionalError::external(
                "filesystem",
                Some("read capsule".to_string()),
                error.to_string(),
            )
        })?;
        let artifact_hash = hex_digest(&bytes);

        Ok(CapsuleBuildResult {
            capsule_id: request.capsule_id.clone(),
            path: path.clone(),
            storage_ref: format!("memvid:{}", path.display()),
            artifact_hash,
            version: "v1".to_string(),
            memvid_version: MEMVID_CORE_VERSION.to_string(),
            document_count: request.documents.len(),
        })
    }

    fn open_capsule(&self, capsule_id: &str) -> InstitutionalResult<CapsuleHandle> {
        let path = self.capsule_path(capsule_id);
        if !path.exists() {
            return Err(InstitutionalError::NotFound {
                resource: format!("capsule `{capsule_id}`"),
            });
        }
        Ok(CapsuleHandle {
            capsule_id: capsule_id.to_string(),
            storage_ref: format!("memvid:{}", path.display()),
            path,
        })
    }

    fn search_capsule(
        &self,
        capsule_id: &str,
        request: &CapsuleSearchRequest,
    ) -> InstitutionalResult<Vec<CapsuleSearchHit>> {
        let handle = self.open_capsule(capsule_id)?;
        let mut memvid = Memvid::open_read_only(&handle.path).map_err(|error| {
            InstitutionalError::external("memvid", Some("open".to_string()), error.to_string())
        })?;
        let response = memvid
            .search(SearchRequest {
                query: request.query.clone(),
                top_k: request.top_k,
                snippet_chars: request.snippet_chars,
                uri: None,
                scope: None,
                cursor: None,
                as_of_frame: None,
                as_of_ts: None,
                no_sketch: false,
                acl_context: None,
                acl_enforcement_mode: AclEnforcementMode::default(),
            })
            .map_err(|error| {
                InstitutionalError::external(
                    "memvid",
                    Some("search".to_string()),
                    error.to_string(),
                )
            })?;
        Ok(response
            .hits
            .into_iter()
            .map(|hit| CapsuleSearchHit {
                uri: hit.uri,
                title: hit.title,
                text: hit.text,
                score: hit.score,
                metadata: hit
                    .metadata
                    .map_or_else(BTreeMap::new, |metadata| metadata.extra_metadata),
            })
            .collect())
    }

    fn extract_text(
        &self,
        bytes: &[u8],
        format: KnowledgeDocumentFormatV1,
        mime_type: Option<&str>,
    ) -> InstitutionalResult<String> {
        match format {
            KnowledgeDocumentFormatV1::Json => extract_json_text(bytes),
            KnowledgeDocumentFormatV1::Xml => extract_xml_text(bytes),
            KnowledgeDocumentFormatV1::Html => extract_html_text(bytes),
            KnowledgeDocumentFormatV1::Pdf => extract_binary_text(bytes, mime_type),
            KnowledgeDocumentFormatV1::Text => String::from_utf8(bytes.to_vec())
                .map_err(|error| InstitutionalError::parse("knowledge text", error.to_string())),
        }
    }
}

fn extract_json_text(bytes: &[u8]) -> InstitutionalResult<String> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| InstitutionalError::parse("knowledge json", error.to_string()))?;
    let mut lines = Vec::new();
    flatten_json(&value, None, &mut lines);
    Ok(lines.join("\n"))
}

fn extract_xml_text(bytes: &[u8]) -> InstitutionalResult<String> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut output = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(text)) => output.push(
                text.decode()
                    .map_err(|error| InstitutionalError::parse("knowledge xml", error.to_string()))?
                    .into_owned(),
            ),
            Ok(Event::CData(text)) => output.push(
                text.decode()
                    .map_err(|error| InstitutionalError::parse("knowledge xml", error.to_string()))?
                    .into_owned(),
            ),
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(InstitutionalError::parse(
                    "knowledge xml",
                    error.to_string(),
                ));
            }
        }
    }
    Ok(output.join("\n"))
}

fn extract_html_text(bytes: &[u8]) -> InstitutionalResult<String> {
    let html = String::from_utf8(bytes.to_vec())
        .map_err(|error| InstitutionalError::parse("knowledge html", error.to_string()))?;
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(character),
            _ => {}
        }
    }
    Ok(out.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn extract_binary_text(bytes: &[u8], mime_type: Option<&str>) -> InstitutionalResult<String> {
    let document = DocumentProcessor::default()
        .extract_from_bytes(bytes)
        .map_err(|error| {
            InstitutionalError::external(
                "memvid",
                Some(format!("extract {}", mime_type.unwrap_or("binary"))),
                error.to_string(),
            )
        })?;
    document
        .text
        .ok_or_else(|| InstitutionalError::InvariantViolation {
            invariant: "binary document did not yield extractable text".to_string(),
        })
}

fn flatten_json(value: &serde_json::Value, prefix: Option<&str>, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Null => {}
        serde_json::Value::Bool(inner) => {
            out.push(render_json_leaf(prefix, inner.to_string()));
        }
        serde_json::Value::Number(inner) => {
            out.push(render_json_leaf(prefix, inner.to_string()));
        }
        serde_json::Value::String(inner) => {
            out.push(render_json_leaf(prefix, inner.clone()));
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let next = prefix.map_or_else(
                    || format!("[{index}]"),
                    |current| format!("{current}[{index}]"),
                );
                flatten_json(item, Some(&next), out);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                let next = prefix.map_or_else(|| key.clone(), |current| format!("{current}.{key}"));
                flatten_json(item, Some(&next), out);
            }
        }
    }
}

fn render_json_leaf(prefix: Option<&str>, value: String) -> String {
    prefix.map_or(value.clone(), |path| format!("{path}: {value}"))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{
        CapsuleBuildRequest, CapsuleDocument, CapsuleSearchRequest, KnowledgeMemoryProvider,
        MemvidMemoryProvider,
    };

    #[test]
    fn memvid_provider_builds_and_searches_capsules() {
        let dir = tempfile::tempdir().expect("tempdir");
        let provider = MemvidMemoryProvider::new(dir.path());
        let request = CapsuleBuildRequest {
            capsule_id: "capsule-1".to_string(),
            documents: vec![
                CapsuleDocument {
                    document_id: "doc-1".to_string(),
                    title: "IMF balance of payments".to_string(),
                    uri: "mv2://imf/doc-1".to_string(),
                    content: include_str!(
                        "../../../../testing/fixtures/knowledge/imf_external_accounts.txt"
                    )
                    .to_string(),
                    metadata: std::collections::BTreeMap::from([
                        ("source_id".to_string(), "source-imf".to_string()),
                        ("provider".to_string(), "IMF".to_string()),
                    ]),
                    search_text: None,
                },
                CapsuleDocument {
                    document_id: "doc-2".to_string(),
                    title: "BIS liquidity note".to_string(),
                    uri: "mv2://bis/doc-2".to_string(),
                    content: include_str!(
                        "../../../../testing/fixtures/knowledge/bis_liquidity_note.txt"
                    )
                    .to_string(),
                    metadata: std::collections::BTreeMap::from([
                        ("source_id".to_string(), "source-bis".to_string()),
                        ("provider".to_string(), "BIS".to_string()),
                    ]),
                    search_text: None,
                },
                CapsuleDocument {
                    document_id: "doc-3".to_string(),
                    title: "Central bank bulletin".to_string(),
                    uri: "mv2://cb/doc-3".to_string(),
                    content: include_str!(
                        "../../../../testing/fixtures/knowledge/central_bank_bulletin.txt"
                    )
                    .to_string(),
                    metadata: std::collections::BTreeMap::from([
                        ("source_id".to_string(), "source-cb".to_string()),
                        ("provider".to_string(), "CentralBank".to_string()),
                    ]),
                    search_text: None,
                },
            ],
        };

        let build = provider.build_capsule(&request).expect("build capsule");
        assert_eq!(build.document_count, 3);
        assert!(build.path.exists());

        let hits = provider
            .search_capsule(
                "capsule-1",
                &CapsuleSearchRequest {
                    query: "dollar funding stress".to_string(),
                    top_k: 2,
                    snippet_chars: 120,
                },
            )
            .expect("search");
        assert!(!hits.is_empty());
        assert!(hits[0].text.to_lowercase().contains("funding"));
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use contracts::{
    AnalysisCoverageV1, AnalysisImplicationsV1, Classification, ConfidenceV1, DataRegisterEntryV1,
    DirectionalBiasV1, DriverBucketV1, ExecutiveBriefV1, FxDriverAssessmentV1,
    GlobalLiquidityPhaseV1, KnowledgeAppendixV1, KnowledgeCapsuleV1, KnowledgeDocumentFormatV1,
    KnowledgeEdgeV1, KnowledgePublicationRequestV1, KnowledgePublicationStatusV1,
    KnowledgeRelationshipV1, KnowledgeSourceFetchSpecV1, KnowledgeSourceIngestRequestV1,
    KnowledgeSourceKindV1, KnowledgeSourceV1, MacroFinancialAnalysisRequestV1,
    MacroFinancialAnalysisV1, MechanismMapV1, ProbabilityV1, RankedRiskV1, RiskRegisterEntryV1,
    ScenarioCaseV1, ScenarioKindV1, ServiceBoundaryV1, SignalMagnitudeV1, SignalSummaryEntryV1,
    SourceConstraintsV1, WatchlistIndicatorV1,
};
use enforcement::ApprovedMutationContext;
use error_model::{InstitutionalError, InstitutionalResult};
use events::{EventEnvelopeV1, RecordedEventV1};
use identity::ActorRef;
use memory_provider::{
    CapsuleBuildRequest, CapsuleDocument, CapsuleSearchRequest, KnowledgeMemoryProvider,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use surrealdb::Connection;
use surrealdb_access::SurrealRepositoryContext;
use trading_core::{Clock, IdGenerator, SystemClock, SystemIdGenerator};

const SERVICE_NAME: &str = "knowledge-service";
const DOMAIN_NAME: &str = "data_knowledge";
const APPROVED_WORKFLOWS: &[&str] = &["knowledge_publication", "record_retention"];
const OWNED_AGGREGATES: &[&str] = &[
    "knowledge_record",
    "knowledge_source",
    "knowledge_capsule",
    "knowledge_analysis",
    "knowledge_edge",
    "retention_policy",
];
const ANALYSIS_TIMEZONE: &str = "America/Los_Angeles";

#[derive(Clone)]
pub struct KnowledgeService<M>
where
    M: KnowledgeMemoryProvider + Clone,
{
    client: Client,
    memory_provider: M,
    official_document_hosts: BTreeSet<String>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl<M> std::fmt::Debug for KnowledgeService<M>
where
    M: KnowledgeMemoryProvider + Clone,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("KnowledgeService")
            .field("official_document_hosts", &self.official_document_hosts)
            .finish_non_exhaustive()
    }
}

impl<M> KnowledgeService<M>
where
    M: KnowledgeMemoryProvider + Clone,
{
    #[must_use]
    pub fn new(memory_provider: M, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self {
            client: Client::new(),
            memory_provider,
            official_document_hosts: default_official_document_hosts(),
            clock,
            ids,
        }
    }

    #[must_use]
    pub fn with_official_document_host(mut self, host: impl Into<String>) -> Self {
        self.official_document_hosts.insert(host.into());
        self
    }

    pub async fn ingest_sources<C>(
        &self,
        context: &ApprovedMutationContext,
        repositories: &SurrealRepositoryContext<C>,
        request: KnowledgeSourceIngestRequestV1,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>>
    where
        C: Connection,
    {
        context.assert_workflow("knowledge_publication")?;
        context.assert_target_service(SERVICE_NAME)?;
        self.ingest_sources_unchecked(repositories, request).await
    }

    pub async fn publish_capsule<C>(
        &self,
        context: &ApprovedMutationContext,
        repositories: &SurrealRepositoryContext<C>,
        request: KnowledgePublicationRequestV1,
    ) -> InstitutionalResult<KnowledgeCapsuleV1>
    where
        C: Connection,
    {
        context.assert_workflow("knowledge_publication")?;
        context.assert_target_service(SERVICE_NAME)?;
        self.publish_capsule_unchecked(repositories, request).await
    }

    pub async fn generate_analysis<C>(
        &self,
        repositories: &SurrealRepositoryContext<C>,
        request: MacroFinancialAnalysisRequestV1,
    ) -> InstitutionalResult<MacroFinancialAnalysisV1>
    where
        C: Connection,
    {
        let sources = self.resolve_sources(repositories, &request).await?;
        if sources.is_empty() {
            return Err(InstitutionalError::NotFound {
                resource: "macro-financial analysis sources".to_string(),
            });
        }

        let retrieval_context = if let Some(capsule_id) = &request.capsule_id {
            self.memory_provider
                .search_capsule(
                    capsule_id,
                    &CapsuleSearchRequest {
                        query: build_retrieval_query(&request),
                        top_k: 4,
                        snippet_chars: 180,
                    },
                )?
                .into_iter()
                .map(|hit| {
                    let label = hit.title.unwrap_or(hit.uri);
                    format!("{label}: {}", hit.text)
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let data_vintage = request
            .data_vintage
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let required_inputs = required_inputs();
        let dependent_variables = dependent_variables();
        let global_liquidity_phase = determine_global_liquidity_phase(&sources);
        let driver_decomposition =
            build_driver_decomposition(&sources, &request.coverage, global_liquidity_phase);
        let executive_brief = build_executive_brief(
            &request,
            &sources,
            &data_vintage,
            &driver_decomposition,
            &retrieval_context,
            self.clock.now(),
        );
        let data_register = build_data_register(&sources);
        let mechanism_map =
            build_mechanism_map(&sources, global_liquidity_phase, &request.coverage);
        let scenario_matrix =
            build_scenario_matrix(&request, global_liquidity_phase, &driver_decomposition);
        let risk_register = build_risk_register(&driver_decomposition, &sources);
        let knowledge_appendix = build_knowledge_appendix(&request.constraints);

        let mut analysis = MacroFinancialAnalysisV1 {
            analysis_id: request.analysis_id.clone(),
            generated_at: self.clock.now(),
            trace_ref: format!("analysis::{}", request.analysis_id),
            objective: request.objective,
            horizon: request.horizon,
            coverage: request.coverage.clone(),
            data_vintage,
            required_inputs,
            dependent_variables,
            global_liquidity_phase,
            driver_decomposition,
            executive_brief,
            data_register,
            mechanism_map,
            scenario_matrix,
            risk_register,
            knowledge_appendix,
            source_ids: sources
                .iter()
                .map(|source| source.source_id.clone())
                .collect(),
            capsule_id: request.capsule_id.clone(),
            rendered_output: String::new(),
            retrieval_context,
        };
        analysis.rendered_output = render_analysis(&analysis);

        repositories
            .knowledge_analyses()
            .store(analysis.clone())
            .await?;
        repositories
            .recorded_events()
            .append(
                self.ids.next_id(),
                RecordedEventV1 {
                    envelope: EventEnvelopeV1::new(
                        "knowledge.analysis_generated",
                        ActorRef("knowledge-service".to_string()),
                        analysis.analysis_id.clone(),
                        None,
                        request.classification,
                        "schemas/events/data_knowledge/v1/knowledge-analysis-generated-v1.json",
                        sha256_hex(analysis.rendered_output.as_bytes()),
                    ),
                    payload_ref: contracts::PayloadRefV1 {
                        schema_ref: "schemas/contracts/v1/macro-financial-analysis-v1.json"
                            .to_string(),
                        record_id: analysis.analysis_id.clone(),
                    },
                },
            )
            .await?;
        for source in &sources {
            repositories
                .knowledge_edges()
                .store(KnowledgeEdgeV1 {
                    edge_id: self.ids.next_id(),
                    from_id: analysis.analysis_id.clone(),
                    to_id: source.source_id.clone(),
                    relationship: KnowledgeRelationshipV1::Cites,
                    rationale: "Analysis cites a governed primary source.".to_string(),
                })
                .await?;
        }
        if let Some(capsule_id) = &analysis.capsule_id {
            repositories
                .knowledge_edges()
                .store(KnowledgeEdgeV1 {
                    edge_id: self.ids.next_id(),
                    from_id: analysis.analysis_id.clone(),
                    to_id: capsule_id.clone(),
                    relationship: KnowledgeRelationshipV1::Supports,
                    rationale: "Analysis used capsule retrieval context.".to_string(),
                })
                .await?;
        }

        Ok(analysis)
    }

    pub async fn load_analysis<C>(
        &self,
        repositories: &SurrealRepositoryContext<C>,
        analysis_id: &str,
    ) -> InstitutionalResult<Option<MacroFinancialAnalysisV1>>
    where
        C: Connection,
    {
        Ok(repositories
            .knowledge_analyses()
            .load(analysis_id)
            .await?
            .map(|record| record.analysis))
    }

    pub async fn latest_publication_status<C>(
        &self,
        repositories: &SurrealRepositoryContext<C>,
    ) -> InstitutionalResult<Option<KnowledgePublicationStatusV1>>
    where
        C: Connection,
    {
        repositories.knowledge_capsules().latest_status().await
    }

    pub async fn ingest_sources_unchecked<C>(
        &self,
        repositories: &SurrealRepositoryContext<C>,
        request: KnowledgeSourceIngestRequestV1,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>>
    where
        C: Connection,
    {
        let mut sources = Vec::with_capacity(request.sources.len());
        for spec in &request.sources {
            validate_source_url(spec, &self.official_document_hosts)?;
            let response = self.client.get(&spec.url).send().await.map_err(|error| {
                InstitutionalError::external(
                    "reqwest",
                    Some(format!("GET {}", spec.url)),
                    error.to_string(),
                )
            })?;
            if !response.status().is_success() {
                return Err(InstitutionalError::external(
                    "reqwest",
                    Some(format!("GET {}", spec.url)),
                    format!("unexpected status {}", response.status()),
                ));
            }
            let mime_type = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map_or_else(
                    || default_mime_type(spec.expected_format).to_string(),
                    str::to_owned,
                );
            let bytes = response.bytes().await.map_err(|error| {
                InstitutionalError::external(
                    "reqwest",
                    Some(format!("read body {}", spec.url)),
                    error.to_string(),
                )
            })?;
            let source = self.normalize_source(
                &request.ingestion_id,
                request.classification,
                spec,
                bytes.as_ref(),
                &mime_type,
            )?;
            repositories
                .knowledge_sources()
                .store(source.clone())
                .await?;
            repositories
                .recorded_events()
                .append(
                    self.ids.next_id(),
                    RecordedEventV1 {
                        envelope: EventEnvelopeV1::new(
                            "knowledge.source_ingested",
                            ActorRef("knowledge-service".to_string()),
                            request.ingestion_id.clone(),
                            None,
                            request.classification,
                            "schemas/events/data_knowledge/v1/knowledge-source-ingested-v1.json",
                            source.content_digest.clone(),
                        ),
                        payload_ref: contracts::PayloadRefV1 {
                            schema_ref: "schemas/contracts/v1/knowledge-source-ingest-v1.json"
                                .to_string(),
                            record_id: source.source_id.clone(),
                        },
                    },
                )
                .await?;
            sources.push(source);
        }
        Ok(sources)
    }

    pub async fn publish_capsule_unchecked<C>(
        &self,
        repositories: &SurrealRepositoryContext<C>,
        request: KnowledgePublicationRequestV1,
    ) -> InstitutionalResult<KnowledgeCapsuleV1>
    where
        C: Connection,
    {
        let source_records = repositories
            .knowledge_sources()
            .load_many(&request.source_ids)
            .await?;
        if source_records.len() != request.source_ids.len() {
            return Err(InstitutionalError::NotFound {
                resource: "one or more knowledge sources".to_string(),
            });
        }

        let documents = source_records
            .iter()
            .map(|record| CapsuleDocument {
                document_id: record.source.source_id.clone(),
                title: record.source.title.clone(),
                uri: format!("knowledge://source/{}", record.source.source_id),
                content: record.source.content_text.clone(),
                metadata: BTreeMap::from([
                    ("source_id".to_string(), record.source.source_id.clone()),
                    (
                        "provider".to_string(),
                        source_kind_label(record.source.kind).to_string(),
                    ),
                    (
                        "country_area".to_string(),
                        record.source.country_area.clone(),
                    ),
                ]),
                search_text: Some(record.source.content_text.clone()),
            })
            .collect::<Vec<_>>();
        let build = self.memory_provider.build_capsule(&CapsuleBuildRequest {
            capsule_id: request.capsule_id.clone(),
            documents,
        })?;
        let capsule = KnowledgeCapsuleV1 {
            capsule_id: request.capsule_id.clone(),
            publication_id: request.publication_id.clone(),
            title: request.title,
            source_ids: request.source_ids.clone(),
            source_count: source_records.len(),
            storage_ref: build.storage_ref,
            artifact_hash: build.artifact_hash,
            version: build.version,
            memvid_version: build.memvid_version,
            published_at: self.clock.now(),
            classification: request.classification,
            retention_class: request.retention_class,
        };
        repositories
            .knowledge_capsules()
            .store(capsule.clone())
            .await?;
        repositories
            .recorded_events()
            .append(
                self.ids.next_id(),
                RecordedEventV1 {
                    envelope: EventEnvelopeV1::new(
                        "knowledge.capsule_published",
                        ActorRef("knowledge-service".to_string()),
                        request.publication_id.clone(),
                        None,
                        request.classification,
                        "schemas/events/data_knowledge/v1/knowledge-capsule-published-v1.json",
                        capsule.artifact_hash.clone(),
                    ),
                    payload_ref: contracts::PayloadRefV1 {
                        schema_ref: "schemas/contracts/v1/knowledge-publication-v1.json"
                            .to_string(),
                        record_id: capsule.capsule_id.clone(),
                    },
                },
            )
            .await?;
        for source in &source_records {
            repositories
                .knowledge_edges()
                .store(KnowledgeEdgeV1 {
                    edge_id: self.ids.next_id(),
                    from_id: capsule.capsule_id.clone(),
                    to_id: source.source.source_id.clone(),
                    relationship: KnowledgeRelationshipV1::DerivedFrom,
                    rationale: "Capsule compiled from governed source text.".to_string(),
                })
                .await?;
        }
        Ok(capsule)
    }

    async fn resolve_sources<C>(
        &self,
        repositories: &SurrealRepositoryContext<C>,
        request: &MacroFinancialAnalysisRequestV1,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>>
    where
        C: Connection,
    {
        if !request.source_ids.is_empty() {
            return Ok(repositories
                .knowledge_sources()
                .load_many(&request.source_ids)
                .await?
                .into_iter()
                .map(|record| record.source)
                .collect());
        }
        if let Some(capsule_id) = &request.capsule_id {
            if let Some(capsule) = repositories.knowledge_capsules().load(capsule_id).await? {
                return Ok(repositories
                    .knowledge_sources()
                    .load_many(&capsule.capsule.source_ids)
                    .await?
                    .into_iter()
                    .map(|record| record.source)
                    .collect());
            }
        }
        Ok(Vec::new())
    }

    fn normalize_source(
        &self,
        ingestion_id: &str,
        classification: Classification,
        spec: &KnowledgeSourceFetchSpecV1,
        bytes: &[u8],
        mime_type: &str,
    ) -> InstitutionalResult<KnowledgeSourceV1> {
        let source_url = reqwest::Url::parse(&spec.url).map_err(|error| {
            InstitutionalError::parse("knowledge source url", error.to_string())
        })?;
        let source_domain = source_url.host_str().unwrap_or_default().to_string();
        let parsed = parse_source_metadata(spec, bytes)?;
        let content_text =
            self.memory_provider
                .extract_text(bytes, spec.expected_format, Some(mime_type))?;

        Ok(KnowledgeSourceV1 {
            source_id: spec.source_id.clone(),
            ingestion_id: ingestion_id.to_string(),
            kind: spec.kind,
            title: spec.title.clone(),
            country_area: spec.country_area.clone(),
            series_name: spec.series_name.clone(),
            source_url: spec.url.clone(),
            source_domain,
            format: spec.expected_format,
            mime_type: mime_type.to_string(),
            classification,
            acquired_at: self.clock.now(),
            content_digest: sha256_hex(bytes),
            content_text,
            last_observation: parsed.last_observation,
            units: spec.units.clone().or(parsed.units),
            transform: spec.transform.clone().or(parsed.transform),
            release_lag: spec.release_lag.clone().or(parsed.release_lag),
            quality_flags: parsed.quality_flags,
            notes: spec.notes.clone(),
            provider_metadata: parsed.provider_metadata,
        })
    }
}

impl<M> Default for KnowledgeService<M>
where
    M: KnowledgeMemoryProvider + Clone + Default,
{
    fn default() -> Self {
        Self::new(
            M::default(),
            Arc::new(SystemClock),
            Arc::new(SystemIdGenerator),
        )
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        owned_aggregates: OWNED_AGGREGATES
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

#[derive(Debug, Default)]
struct ParsedSourceMetadata {
    last_observation: Option<String>,
    units: Option<String>,
    transform: Option<String>,
    release_lag: Option<String>,
    quality_flags: Vec<contracts::QualityFlagV1>,
    provider_metadata: BTreeMap<String, String>,
}

fn default_official_document_hosts() -> BTreeSet<String> {
    [
        "imf.org",
        "bis.org",
        "worldbank.org",
        "fred.stlouisfed.org",
        "federalreserve.gov",
        "ecb.europa.eu",
        "boj.or.jp",
        "bankofengland.co.uk",
        "treasury.gov",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn validate_source_url(
    spec: &KnowledgeSourceFetchSpecV1,
    official_document_hosts: &BTreeSet<String>,
) -> InstitutionalResult<()> {
    let parsed = reqwest::Url::parse(&spec.url)
        .map_err(|error| InstitutionalError::parse("knowledge source url", error.to_string()))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| InstitutionalError::parse("knowledge source url", "missing host"))?;
    let allowed = match spec.kind {
        KnowledgeSourceKindV1::Imf => host_matches(host, "imf.org"),
        KnowledgeSourceKindV1::Bis => host_matches(host, "bis.org"),
        KnowledgeSourceKindV1::WorldBank => host_matches(host, "worldbank.org"),
        KnowledgeSourceKindV1::Fred => host_matches(host, "fred.stlouisfed.org"),
        KnowledgeSourceKindV1::OfficialDocument => {
            matches!(
                host.rsplit('.').next(),
                Some(segment) if segment.eq_ignore_ascii_case("gov")
            ) || official_document_hosts
                .iter()
                .any(|allowed_host| host_matches(host, allowed_host))
        }
    };
    if allowed
        || official_document_hosts
            .iter()
            .any(|allowed_host| host_matches(host, allowed_host))
    {
        Ok(())
    } else {
        Err(InstitutionalError::PolicyDenied {
            reason: format!("source host `{host}` is not allowed for {:?}", spec.kind),
        })
    }
}

fn host_matches(host: &str, expected: &str) -> bool {
    host == expected || host.ends_with(&format!(".{expected}"))
}

fn default_mime_type(format: KnowledgeDocumentFormatV1) -> &'static str {
    match format {
        KnowledgeDocumentFormatV1::Json => "application/json",
        KnowledgeDocumentFormatV1::Xml => "application/xml",
        KnowledgeDocumentFormatV1::Html => "text/html",
        KnowledgeDocumentFormatV1::Pdf => "application/pdf",
        KnowledgeDocumentFormatV1::Text => "text/plain",
    }
}

fn parse_source_metadata(
    spec: &KnowledgeSourceFetchSpecV1,
    bytes: &[u8],
) -> InstitutionalResult<ParsedSourceMetadata> {
    match spec.kind {
        KnowledgeSourceKindV1::Fred => parse_fred_metadata(bytes),
        KnowledgeSourceKindV1::WorldBank => parse_world_bank_metadata(bytes),
        KnowledgeSourceKindV1::Imf | KnowledgeSourceKindV1::Bis => parse_xml_series_metadata(bytes),
        KnowledgeSourceKindV1::OfficialDocument => Ok(ParsedSourceMetadata {
            quality_flags: vec![contracts::QualityFlagV1::ProxyUsed],
            provider_metadata: BTreeMap::from([(
                "document_type".to_string(),
                "official_document".to_string(),
            )]),
            ..ParsedSourceMetadata::default()
        }),
    }
}

fn parse_fred_metadata(bytes: &[u8]) -> InstitutionalResult<ParsedSourceMetadata> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| InstitutionalError::parse("FRED json", error.to_string()))?;
    let observations = value
        .get("observations")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let last = observations
        .iter()
        .rev()
        .find(|entry| entry.get("value").and_then(serde_json::Value::as_str) != Some("."));
    Ok(ParsedSourceMetadata {
        last_observation: last.and_then(|entry| {
            let date = entry.get("date").and_then(serde_json::Value::as_str)?;
            let value = entry.get("value").and_then(serde_json::Value::as_str)?;
            Some(format!("{date}={value}"))
        }),
        provider_metadata: BTreeMap::from([(
            "observation_count".to_string(),
            observations.len().to_string(),
        )]),
        ..ParsedSourceMetadata::default()
    })
}

fn parse_world_bank_metadata(bytes: &[u8]) -> InstitutionalResult<ParsedSourceMetadata> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| InstitutionalError::parse("World Bank json", error.to_string()))?;
    let Some(series) = value
        .as_array()
        .and_then(|items| items.get(1))
        .and_then(serde_json::Value::as_array)
    else {
        return Ok(ParsedSourceMetadata {
            quality_flags: vec![contracts::QualityFlagV1::Estimated],
            ..ParsedSourceMetadata::default()
        });
    };
    let last = series.iter().find_map(|entry| {
        let date = entry.get("date").and_then(serde_json::Value::as_str)?;
        let value = entry.get("value")?;
        Some(format!("{date}={}", render_json_scalar(value)))
    });
    Ok(ParsedSourceMetadata {
        last_observation: last,
        provider_metadata: BTreeMap::from([(
            "observation_count".to_string(),
            series.len().to_string(),
        )]),
        ..ParsedSourceMetadata::default()
    })
}

fn parse_xml_series_metadata(bytes: &[u8]) -> InstitutionalResult<ParsedSourceMetadata> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut last_observation = None;
    let mut obs_count = 0_usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element)) => {
                let name = String::from_utf8_lossy(element.name().as_ref()).to_ascii_lowercase();
                if name.contains("obs") {
                    let mut time_period = None;
                    let mut obs_value = None;
                    for attr in element.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_ascii_lowercase();
                        let value = attr
                            .decode_and_unescape_value(reader.decoder())
                            .map_err(|error| {
                                InstitutionalError::parse(
                                    "knowledge xml attribute",
                                    error.to_string(),
                                )
                            })?
                            .into_owned();
                        if key.contains("time") || key.contains("date") || key.contains("period") {
                            time_period = Some(value.clone());
                        }
                        if key.contains("value") {
                            obs_value = Some(value);
                        }
                    }
                    if time_period.is_some() || obs_value.is_some() {
                        obs_count += 1;
                        last_observation = Some(format!(
                            "{}={}",
                            time_period.unwrap_or_else(|| "UNKNOWN".to_string()),
                            obs_value.unwrap_or_else(|| "UNKNOWN".to_string())
                        ));
                    }
                }
            }
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
    Ok(ParsedSourceMetadata {
        last_observation,
        provider_metadata: BTreeMap::from([(
            "observation_count".to_string(),
            obs_count.to_string(),
        )]),
        ..ParsedSourceMetadata::default()
    })
}

fn build_retrieval_query(request: &MacroFinancialAnalysisRequestV1) -> String {
    let mut parts = Vec::new();
    parts.push(request.objective.directive_label().to_ascii_lowercase());
    parts.extend(
        request
            .coverage
            .countries
            .iter()
            .map(|value| value.to_ascii_lowercase()),
    );
    parts.extend(
        request
            .coverage
            .currencies
            .iter()
            .map(|value| value.to_ascii_lowercase()),
    );
    parts.extend(
        request
            .coverage
            .fx_pairs
            .iter()
            .map(|value| value.to_ascii_lowercase()),
    );
    parts.join(" ")
}

fn required_inputs() -> Vec<String> {
    vec![
        "FX levels/returns".to_string(),
        "Rates and yields".to_string(),
        "Balance of payments and IIP components".to_string(),
        "Cross-border banking and credit measures".to_string(),
        "Funding and hedging indicators".to_string(),
        "Policy communications and geopolitical timeline".to_string(),
    ]
}

fn dependent_variables() -> Vec<String> {
    vec![
        "FX: bilateral/effective/real effective as applicable".to_string(),
        "Financial conditions: domestic and external".to_string(),
        "Capital flows: gross/net by functional category".to_string(),
        "Liquidity risk: funding stress indicators and backstop capacity".to_string(),
    ]
}

fn determine_global_liquidity_phase(sources: &[KnowledgeSourceV1]) -> GlobalLiquidityPhaseV1 {
    let haystack = sources
        .iter()
        .map(|source| source.content_text.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    if haystack.contains("stress") || haystack.contains("basis") {
        GlobalLiquidityPhaseV1::Stress
    } else if haystack.contains("tight") || haystack.contains("slow") {
        GlobalLiquidityPhaseV1::Tighten
    } else {
        GlobalLiquidityPhaseV1::Ease
    }
}

fn build_driver_decomposition(
    sources: &[KnowledgeSourceV1],
    coverage: &AnalysisCoverageV1,
    global_liquidity_phase: GlobalLiquidityPhaseV1,
) -> Vec<FxDriverAssessmentV1> {
    let corpus = sources
        .iter()
        .map(|source| source.content_text.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    let rate_direction = if corpus.contains("rate") || corpus.contains("policy") {
        DirectionalBiasV1::Positive
    } else {
        DirectionalBiasV1::Stable
    };
    let risk_direction = if global_liquidity_phase == GlobalLiquidityPhaseV1::Stress {
        DirectionalBiasV1::Negative
    } else {
        DirectionalBiasV1::Stable
    };
    let flow_direction = if corpus.contains("inflow") || corpus.contains("surplus") {
        DirectionalBiasV1::Positive
    } else if corpus.contains("outflow") || corpus.contains("reversal") {
        DirectionalBiasV1::Negative
    } else {
        DirectionalBiasV1::Mixed
    };
    let commodity_direction = if coverage
        .asset_classes
        .iter()
        .any(|value| value.to_ascii_lowercase().contains("commodity"))
    {
        DirectionalBiasV1::Mixed
    } else {
        DirectionalBiasV1::Stable
    };
    let funding_direction = if corpus.contains("funding") || corpus.contains("basis") {
        DirectionalBiasV1::Negative
    } else {
        DirectionalBiasV1::Stable
    };
    let geopolitical_direction = if corpus.contains("sanction") || corpus.contains("conflict") {
        DirectionalBiasV1::Negative
    } else {
        DirectionalBiasV1::Stable
    };

    vec![
        FxDriverAssessmentV1 {
            bucket: DriverBucketV1::RateDifferentialsExpectedPolicyPaths,
            direction: rate_direction,
            magnitude: SignalMagnitudeV1::Medium,
            confidence: confidence_from_source_count(sources.len()),
            evidence: "Policy and rate-signaling content were present in the source set."
                .to_string(),
        },
        FxDriverAssessmentV1 {
            bucket: DriverBucketV1::RiskSentimentGlobalFinancialCycleExposure,
            direction: risk_direction,
            magnitude: magnitude_from_liquidity_phase(global_liquidity_phase),
            confidence: confidence_from_source_count(sources.len()),
            evidence: "Global liquidity and stress language shaped the risk-sentiment channel."
                .to_string(),
        },
        FxDriverAssessmentV1 {
            bucket: DriverBucketV1::FlowShocks,
            direction: flow_direction,
            magnitude: SignalMagnitudeV1::Medium,
            confidence: confidence_from_source_count(sources.len()),
            evidence: "Portfolio-flow and current-account cues were used to assess flow pressure."
                .to_string(),
        },
        FxDriverAssessmentV1 {
            bucket: DriverBucketV1::TermsOfTradeCommodityChannel,
            direction: commodity_direction,
            magnitude: SignalMagnitudeV1::Low,
            confidence: ConfidenceV1::Weak,
            evidence: "Commodity channel stayed secondary unless asset-class scope required it."
                .to_string(),
        },
        FxDriverAssessmentV1 {
            bucket: DriverBucketV1::FundingHedgingPremia,
            direction: funding_direction,
            magnitude: magnitude_from_liquidity_phase(global_liquidity_phase),
            confidence: ConfidenceV1::Moderate,
            evidence: "Funding stress and basis language informed hedging-premia assessment."
                .to_string(),
        },
        FxDriverAssessmentV1 {
            bucket: DriverBucketV1::GeopoliticalFractureShocks,
            direction: geopolitical_direction,
            magnitude: SignalMagnitudeV1::Low,
            confidence: ConfidenceV1::Weak,
            evidence: "Geopolitical channel remained contingent on explicit fracture indicators."
                .to_string(),
        },
    ]
}

fn build_executive_brief(
    request: &MacroFinancialAnalysisRequestV1,
    sources: &[KnowledgeSourceV1],
    data_vintage: &str,
    driver_decomposition: &[FxDriverAssessmentV1],
    retrieval_context: &[String],
    as_of: DateTime<Utc>,
) -> ExecutiveBriefV1 {
    let now = as_of.date_naive().to_string();
    let providers = sources
        .iter()
        .map(|source| source_kind_label(source.kind))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let facts = vec![
        format!(
            "Compiled {} governed sources across {} provider families.",
            sources.len(),
            providers.len()
        ),
        format!(
            "Coverage is {} with data_vintage={}.",
            request.coverage.summary(),
            data_vintage
        ),
        format!(
            "Primary providers represented: {}.",
            if providers.is_empty() {
                "MISSING".to_string()
            } else {
                providers.join(", ")
            }
        ),
    ];
    let inferences = vec![
        format!(
            "Global liquidity phase is {} based on funding and stress language in the source set.",
            determine_global_liquidity_phase(sources).directive_label()
        ),
        "Marginal financing is inferred to run through portfolio and bank-credit channels before reserves."
            .to_string(),
        format!(
            "Retrieval context volume is {} snippets, which supports reuse of the capsule as a compiled playbook.",
            retrieval_context.len()
        ),
    ];
    let key_risks = vec![
        RankedRiskV1 {
            risk: "Dollar funding stress".to_string(),
            summary: "Cross-border funding conditions tighten faster than domestic buffers can absorb."
                .to_string(),
            probability: ProbabilityV1::Medium,
        },
        RankedRiskV1 {
            risk: "Flow reversal".to_string(),
            summary: "Portfolio and banking channels reverse before direct investment stabilizes the balance."
                .to_string(),
            probability: ProbabilityV1::Medium,
        },
        RankedRiskV1 {
            risk: "Policy credibility shock".to_string(),
            summary: "Exchange-rate pass-through or communication slippage weakens anchor confidence."
                .to_string(),
            probability: ProbabilityV1::Low,
        },
    ];
    let signal_summary = driver_decomposition
        .iter()
        .take(6)
        .map(|driver| SignalSummaryEntryV1 {
            signal: driver_bucket_label(driver.bucket).to_string(),
            direction: driver.direction,
            magnitude: driver.magnitude,
            confidence: driver.confidence,
            evidence: driver.evidence.clone(),
        })
        .collect();

    ExecutiveBriefV1 {
        as_of_date: now,
        as_of_timezone: ANALYSIS_TIMEZONE.to_string(),
        data_vintage: data_vintage.to_string(),
        objective: request.objective,
        horizon: request.horizon,
        coverage: request.coverage.clone(),
        key_judgments_facts: facts,
        key_judgments_inferences: inferences,
        key_risks,
        signal_summary,
        implications: AnalysisImplicationsV1 {
            policy_evaluation:
                "Preserve optionality around liquidity backstops and intervention signaling."
                    .to_string(),
            investment_strategy:
                "Bias toward hedged positioning until funding and flow signals improve.".to_string(),
            risk_management:
                "Tighten watchlists on basis, reserves, and rollover pressure indicators."
                    .to_string(),
            long_horizon_strategy:
                "Track structural external-balance resilience and sovereign financing mix."
                    .to_string(),
        },
    }
}

fn build_data_register(sources: &[KnowledgeSourceV1]) -> Vec<DataRegisterEntryV1> {
    sources
        .iter()
        .map(|source| DataRegisterEntryV1 {
            series_name: source
                .series_name
                .clone()
                .unwrap_or_else(|| source.title.clone()),
            country_area: source.country_area.clone(),
            source: source_kind_label(source.kind).to_string(),
            frequency: source
                .provider_metadata
                .get("frequency")
                .cloned()
                .unwrap_or_else(|| "MISSING".to_string()),
            last_obs: source
                .last_observation
                .clone()
                .unwrap_or_else(|| "MISSING".to_string()),
            units: source
                .units
                .clone()
                .unwrap_or_else(|| "MISSING".to_string()),
            transform: source
                .transform
                .clone()
                .unwrap_or_else(|| "MISSING".to_string()),
            lag: source
                .release_lag
                .clone()
                .unwrap_or_else(|| "MISSING".to_string()),
            quality_flag: if source.quality_flags.is_empty() {
                String::new()
            } else {
                source
                    .quality_flags
                    .iter()
                    .map(|flag| flag.directive_label())
                    .collect::<Vec<_>>()
                    .join(",")
            },
            notes: if source.notes.is_empty() {
                "MISSING".to_string()
            } else {
                source.notes.join("; ")
            },
        })
        .collect()
}

fn build_mechanism_map(
    sources: &[KnowledgeSourceV1],
    global_liquidity_phase: GlobalLiquidityPhaseV1,
    coverage: &AnalysisCoverageV1,
) -> MechanismMapV1 {
    let country_scope = if coverage.countries.is_empty() {
        "the requested coverage set".to_string()
    } else {
        coverage.countries.join(", ")
    };
    let providers = sources
        .iter()
        .map(|source| source_kind_label(source.kind))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    MechanismMapV1 {
        current_account_narrative: format!(
            "Current account pressure for {country_scope} is assessed from primary sources spanning {providers}."
        ),
        financial_account_funding_mix:
            "Financial account funding is mapped through portfolio, banking, and reserve channels."
                .to_string(),
        reserves_and_backstops:
            "Reserves and institutional backstops remain the final defensive layer in the stress path."
                .to_string(),
        fx_swap_basis_state: format!(
            "FX swap and basis diagnostics point to a {} external funding backdrop.",
            global_liquidity_phase.directive_label()
        ),
        dollar_funding_stress_state: format!(
            "Dollar funding state is classified as {} using BIS-style funding cues and source text.",
            global_liquidity_phase.directive_label()
        ),
        risk_sentiment_linkage:
            "Risk sentiment is translated through the global financial cycle, portfolio flows, and valuation pressure."
                .to_string(),
        spillover_channels:
            "Spillovers are assessed through cross-border banking, bond allocation, and hedging markets."
                .to_string(),
    }
}

fn build_scenario_matrix(
    request: &MacroFinancialAnalysisRequestV1,
    global_liquidity_phase: GlobalLiquidityPhaseV1,
    driver_decomposition: &[FxDriverAssessmentV1],
) -> Vec<ScenarioCaseV1> {
    [
        ScenarioKindV1::Base,
        ScenarioKindV1::Upside,
        ScenarioKindV1::Downside,
        ScenarioKindV1::TailLiquidityEvent,
    ]
    .into_iter()
    .map(|scenario| {
        build_scenario_case(
            scenario,
            request,
            global_liquidity_phase,
            driver_decomposition,
        )
    })
    .collect()
}

fn build_scenario_case(
    scenario: ScenarioKindV1,
    request: &MacroFinancialAnalysisRequestV1,
    global_liquidity_phase: GlobalLiquidityPhaseV1,
    driver_decomposition: &[FxDriverAssessmentV1],
) -> ScenarioCaseV1 {
    let scenario_bias = match scenario {
        ScenarioKindV1::Base => "baseline policy divergence persists without a new shock",
        ScenarioKindV1::Upside => "funding pressure eases and flows stabilize faster than expected",
        ScenarioKindV1::Downside => "portfolio outflows and pass-through concerns intensify",
        ScenarioKindV1::TailLiquidityEvent => {
            "basis blowout and backstop usage signal acute funding stress"
        }
    };
    let fx_outcome = match scenario {
        ScenarioKindV1::Base => "FX remains biased by carry and funding premia.",
        ScenarioKindV1::Upside => "FX appreciation pressure eases as funding channels normalize.",
        ScenarioKindV1::Downside => "FX weakens on tighter external conditions and flow reversal.",
        ScenarioKindV1::TailLiquidityEvent => {
            "FX gaps wider as liquidity overwhelms valuation anchors."
        }
    };
    let watchlist = driver_decomposition
        .iter()
        .take(2)
        .map(|driver| WatchlistIndicatorV1 {
            indicator: driver_bucket_label(driver.bucket).to_string(),
            threshold: match scenario {
                ScenarioKindV1::TailLiquidityEvent => "Move beyond recent stress range".to_string(),
                _ => "Deviate materially from recent trend".to_string(),
            },
            rationale: driver.evidence.clone(),
        })
        .collect();

    ScenarioCaseV1 {
        scenario,
        triggers: format!(
            "{} for {} over {}.",
            scenario_bias,
            request.coverage.summary(),
            request.horizon.directive_label()
        ),
        transmission_path:
            "External accounts -> funding markets -> FX -> domestic financial conditions.".to_string(),
        fx_outcome: fx_outcome.to_string(),
        capital_flows_outcome:
            "Capital flows adjust first through portfolio and banking channels, then reserves.".to_string(),
        liquidity_funding_outcome: format!(
            "Liquidity/funding outcome is benchmarked against a {} baseline.",
            global_liquidity_phase.directive_label()
        ),
        systemic_risk_outcome:
            "Systemic risk rises with leverage, maturity mismatch, and imported funding pressure."
                .to_string(),
        policy_response_space:
            "Policy space depends on reserve adequacy, communication credibility, and macroprudential flexibility."
                .to_string(),
        strategy_implications:
            "Generic strategy bias favors explicit hedging, tighter risk limits, and staged exposure changes."
                .to_string(),
        watchlist,
    }
}

fn build_risk_register(
    driver_decomposition: &[FxDriverAssessmentV1],
    sources: &[KnowledgeSourceV1],
) -> Vec<RiskRegisterEntryV1> {
    let provider_count = sources
        .iter()
        .map(|source| source_kind_label(source.kind))
        .collect::<BTreeSet<_>>()
        .len();
    vec![
        RiskRegisterEntryV1 {
            risk: "Funding basis blowout".to_string(),
            mechanism: "FX hedging premia and swap basis widen as dollar scarcity increases."
                .to_string(),
            early_indicators: "Cross-currency basis, reserve drawdown, and bank-credit tightening."
                .to_string(),
            impact_channels: "FX valuation, domestic liquidity, and rollover costs.".to_string(),
            mitigants_or_hedges:
                "Shorten funding tenor, raise hedge intensity, and monitor backstop eligibility."
                    .to_string(),
            probability: ProbabilityV1::Medium,
            confidence: ConfidenceV1::Moderate,
        },
        RiskRegisterEntryV1 {
            risk: "Flow reversal".to_string(),
            mechanism: "Portfolio reallocations reverse before direct-investment channels adjust."
                .to_string(),
            early_indicators: "Fund-flow data, reserve changes, and sovereign spread widening."
                .to_string(),
            impact_channels: "FX, bond yields, and domestic credit availability.".to_string(),
            mitigants_or_hedges:
                "Stage exposures, keep liquidity buffers, and tighten limit monitoring.".to_string(),
            probability: ProbabilityV1::Medium,
            confidence: confidence_from_source_count(provider_count),
        },
        RiskRegisterEntryV1 {
            risk: "Regime misclassification".to_string(),
            mechanism: "Sparse or lagged data can hide a shift in policy or intervention behavior."
                .to_string(),
            early_indicators: driver_decomposition
                .iter()
                .map(|driver| driver_bucket_label(driver.bucket))
                .collect::<Vec<_>>()
                .join(", "),
            impact_channels: "Incorrect scenario weighting and delayed risk response.".to_string(),
            mitigants_or_hedges:
                "Require fresh official releases and record explicit assumptions when data is missing."
                    .to_string(),
            probability: ProbabilityV1::Low,
            confidence: ConfidenceV1::Moderate,
        },
    ]
}

fn build_knowledge_appendix(constraints: &SourceConstraintsV1) -> KnowledgeAppendixV1 {
    let mut assumptions = vec![
        "1. Official and primary sources dominate the evidence set unless explicitly missing."
            .to_string(),
        "2. Portfolio and banking channels are treated as marginal financers before reserves."
            .to_string(),
        "3. Policy/investment/risk implications remain framework-level and not personalized advice."
            .to_string(),
    ];
    if !constraints.allowed_sources.is_empty() {
        assumptions.push(format!(
            "4. Allowed sources were constrained to {}.",
            constraints.allowed_sources.join(", ")
        ));
    }
    if !constraints.forbidden_sources.is_empty() {
        assumptions.push(format!(
            "5. Forbidden sources were {}.",
            constraints.forbidden_sources.join(", ")
        ));
    }
    KnowledgeAppendixV1 {
        definitions: vec![
            "External accounts cover current account, financial account, and international investment position."
                .to_string(),
            "Functional flow categories include direct, portfolio, other investment, and reserves."
                .to_string(),
            "Reserves and backstops define the last line of defense against external funding stress."
                .to_string(),
        ],
        indicator_dictionary: vec![
            "Liquidity: reserve changes, swap basis, and cross-border bank credit.".to_string(),
            "Flows: portfolio allocations, balance-of-payments detail, and IIP positions."
                .to_string(),
            "FX valuation: bilateral moves, effective rates, and pass-through sensitivity.".to_string(),
            "Risk: sovereign spreads, volatility proxies, and systemic funding indicators."
                .to_string(),
        ],
        playbooks: vec![
            "Risk-on/risk-off: track flow beta, carry support, and reserve intervention."
                .to_string(),
            "EM sudden stop: prioritize marginal financer, rollover risk, and backstop space."
                .to_string(),
            "Reserve drawdown: separate smoothing intervention from regime defense.".to_string(),
            "Basis blowout: monitor swap stress, hedging demand, and nonbank amplification."
                .to_string(),
            "Sovereign stress: map financing needs into bank/nonbank spillovers.".to_string(),
        ],
        common_failure_modes: vec![
            "Data pitfalls: release lags and silent revisions can distort the nowcast.".to_string(),
            "Regime misclassification: intervention patterns can change faster than labels."
                .to_string(),
            "Proxy risk: document-based inference can stand in for missing structured series."
                .to_string(),
        ],
        source_note:
            "Primary-source standards prefer IMF, BIS, World Bank, central banks, finance ministries, and official statistical releases."
                .to_string(),
        assumptions_log: assumptions,
    }
}

fn render_analysis(analysis: &MacroFinancialAnalysisV1) -> String {
    let mut out = Vec::new();
    out.push("[Output 1: Executive Brief]".to_string());
    out.push(format!(
        "AS_OF_DATE: {}",
        analysis.executive_brief.as_of_date
    ));
    out.push(format!(
        "AS_OF_TIMEZONE: {}",
        analysis.executive_brief.as_of_timezone
    ));
    out.push(format!(
        "DATA_VINTAGE: {}",
        analysis.executive_brief.data_vintage
    ));
    out.push(format!(
        "OBJECTIVE: {}",
        analysis.executive_brief.objective.directive_label()
    ));
    out.push(format!(
        "HORIZON: {}",
        analysis.executive_brief.horizon.directive_label()
    ));
    out.push(format!(
        "COVERAGE: {}",
        analysis.executive_brief.coverage.summary()
    ));
    out.push(format!(
        "KEY JUDGMENTS (FACTS): {}",
        join_or_missing(&analysis.executive_brief.key_judgments_facts)
    ));
    out.push(format!(
        "KEY JUDGMENTS (INFERENCES): {}",
        join_or_missing(&analysis.executive_brief.key_judgments_inferences)
    ));
    out.push(format!(
        "KEY RISKS (ranked, with LOW/MEDIUM/HIGH probability): {}",
        if analysis.executive_brief.key_risks.is_empty() {
            "MISSING".to_string()
        } else {
            analysis
                .executive_brief
                .key_risks
                .iter()
                .map(|risk| format!("{} ({})", risk.risk, risk.probability.directive_label()))
                .collect::<Vec<_>>()
                .join("; ")
        }
    ));
    out.push(format!(
        "SIGNAL SUMMARY (top 6 signals; each: direction, magnitude, confidence): {}",
        if analysis.executive_brief.signal_summary.is_empty() {
            "MISSING".to_string()
        } else {
            analysis
                .executive_brief
                .signal_summary
                .iter()
                .map(|signal| {
                    format!(
                        "{} [{} {} {}]",
                        signal.signal,
                        signal.direction.directive_label(),
                        signal.magnitude.directive_label(),
                        signal.confidence.directive_label()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        }
    ));
    out.push("IMPLICATIONS:".to_string());
    out.push(format!(
        "Policy evaluation: {}",
        analysis.executive_brief.implications.policy_evaluation
    ));
    out.push(format!(
        "Investment strategy (generic): {}",
        analysis.executive_brief.implications.investment_strategy
    ));
    out.push(format!(
        "Risk management: {}",
        analysis.executive_brief.implications.risk_management
    ));
    out.push(format!(
        "Long-horizon strategy: {}",
        analysis.executive_brief.implications.long_horizon_strategy
    ));
    out.push(String::new());
    out.push("[Output 2: Data Register]".to_string());
    out.push(
        "Table columns: SERIES_NAME | COUNTRY/AREA | SOURCE | FREQUENCY | LAST_OBS | UNITS | TRANSFORM | LAG | QUALITY_FLAG | NOTES"
            .to_string(),
    );
    for entry in &analysis.data_register {
        out.push(format!(
            "{} | {} | {} | {} | {} | {} | {} | {} | {} | {}",
            entry.series_name,
            entry.country_area,
            entry.source,
            entry.frequency,
            entry.last_obs,
            entry.units,
            entry.transform,
            entry.lag,
            entry.quality_flag,
            entry.notes
        ));
    }
    out.push(String::new());
    out.push("[Output 3: Mechanism Map]".to_string());
    out.push("EXTERNAL ACCOUNTS CORE:".to_string());
    out.push(format!(
        "Current account narrative: {}",
        analysis.mechanism_map.current_account_narrative
    ));
    out.push(format!(
        "Financial account funding mix: {}",
        analysis.mechanism_map.financial_account_funding_mix
    ));
    out.push(format!(
        "Reserves and backstops: {}",
        analysis.mechanism_map.reserves_and_backstops
    ));
    out.push("FUNDING & HEDGING CORE:".to_string());
    out.push(format!(
        "FX swap/basis state: {}",
        analysis.mechanism_map.fx_swap_basis_state
    ));
    out.push(format!(
        "Dollar funding stress state: {}",
        analysis.mechanism_map.dollar_funding_stress_state
    ));
    out.push("GLOBAL CYCLE CORE:".to_string());
    out.push(format!(
        "Risk sentiment linkage: {}",
        analysis.mechanism_map.risk_sentiment_linkage
    ));
    out.push(format!(
        "Spillover channels: {}",
        analysis.mechanism_map.spillover_channels
    ));
    out.push(String::new());
    out.push("[Output 4: Scenario Matrix]".to_string());
    for scenario in &analysis.scenario_matrix {
        out.push(format!("Scenario: {}", scenario.scenario.directive_label()));
        out.push(format!("TRIGGERS: {}", scenario.triggers));
        out.push(format!("TRANSMISSION PATH: {}", scenario.transmission_path));
        out.push(format!("FX OUTCOME: {}", scenario.fx_outcome));
        out.push(format!(
            "CAPITAL FLOWS OUTCOME: {}",
            scenario.capital_flows_outcome
        ));
        out.push(format!(
            "LIQUIDITY/FUNDING OUTCOME: {}",
            scenario.liquidity_funding_outcome
        ));
        out.push(format!(
            "SYSTEMIC RISK OUTCOME: {}",
            scenario.systemic_risk_outcome
        ));
        out.push(format!(
            "POLICY RESPONSE SPACE: {}",
            scenario.policy_response_space
        ));
        out.push(format!(
            "STRATEGY IMPLICATIONS (generic): {}",
            scenario.strategy_implications
        ));
    }
    out.push(String::new());
    out.push("[Output 5: Risk Register]".to_string());
    for risk in &analysis.risk_register {
        out.push(format!("Risk: {}", risk.risk));
        out.push(format!("Mechanism: {}", risk.mechanism));
        out.push(format!("Early indicators: {}", risk.early_indicators));
        out.push(format!("Impact channels: {}", risk.impact_channels));
        out.push(format!(
            "Mitigants / hedges (generic): {}",
            risk.mitigants_or_hedges
        ));
        out.push(format!(
            "Probability (LOW/MEDIUM/HIGH): {}",
            risk.probability.directive_label()
        ));
        out.push(format!(
            "Confidence (WEAK/MODERATE/STRONG): {}",
            risk.confidence.directive_label()
        ));
    }
    out.push(String::new());
    out.push("[Output 6: Knowledge Appendix]".to_string());
    out.push(format!(
        "DEFINITIONS (external accounts, flows, reserves): {}",
        join_or_missing(&analysis.knowledge_appendix.definitions)
    ));
    out.push(format!(
        "INDICATOR DICTIONARY (liquidity, flows, FX valuation, risk): {}",
        join_or_missing(&analysis.knowledge_appendix.indicator_dictionary)
    ));
    out.push(format!(
        "PLAYBOOKS (common regimes: risk-on/risk-off; EM sudden stop; reserve drawdown; basis blowout; sovereign stress): {}",
        join_or_missing(&analysis.knowledge_appendix.playbooks)
    ));
    out.push(format!(
        "COMMON FAILURE MODES (data pitfalls, regime misclassification, proxy risk): {}",
        join_or_missing(&analysis.knowledge_appendix.common_failure_modes)
    ));
    out.push(format!(
        "SOURCE NOTE (what standards were used and why): {}",
        analysis.knowledge_appendix.source_note
    ));
    out.push(format!(
        "ASSUMPTIONS LOG (explicit; numbered; stable): {}",
        join_or_missing(&analysis.knowledge_appendix.assumptions_log)
    ));
    out.join("\n")
}

fn join_or_missing(values: &[String]) -> String {
    if values.is_empty() {
        "MISSING".to_string()
    } else {
        values.join("; ")
    }
}

fn source_kind_label(kind: KnowledgeSourceKindV1) -> &'static str {
    match kind {
        KnowledgeSourceKindV1::Imf => "IMF",
        KnowledgeSourceKindV1::Bis => "BIS",
        KnowledgeSourceKindV1::WorldBank => "WORLD_BANK",
        KnowledgeSourceKindV1::Fred => "FRED",
        KnowledgeSourceKindV1::OfficialDocument => "OFFICIAL_DOCUMENT",
    }
}

fn driver_bucket_label(bucket: DriverBucketV1) -> &'static str {
    match bucket {
        DriverBucketV1::RateDifferentialsExpectedPolicyPaths => {
            "Rate differentials / expected policy paths"
        }
        DriverBucketV1::RiskSentimentGlobalFinancialCycleExposure => {
            "Risk sentiment / global financial cycle exposure"
        }
        DriverBucketV1::FlowShocks => "Flow shocks",
        DriverBucketV1::TermsOfTradeCommodityChannel => "Terms of trade / commodity channel",
        DriverBucketV1::FundingHedgingPremia => "Funding/hedging premia",
        DriverBucketV1::GeopoliticalFractureShocks => "Geopolitical/fracture shocks",
    }
}

fn confidence_from_source_count(source_count: usize) -> ConfidenceV1 {
    match source_count {
        0 | 1 => ConfidenceV1::Weak,
        2 | 3 => ConfidenceV1::Moderate,
        _ => ConfidenceV1::Strong,
    }
}

fn magnitude_from_liquidity_phase(phase: GlobalLiquidityPhaseV1) -> SignalMagnitudeV1 {
    match phase {
        GlobalLiquidityPhaseV1::Ease => SignalMagnitudeV1::Low,
        GlobalLiquidityPhaseV1::Tighten => SignalMagnitudeV1::Medium,
        GlobalLiquidityPhaseV1::Stress => SignalMagnitudeV1::High,
    }
}

fn render_json_scalar(value: &serde_json::Value) -> String {
    if let Some(text) = value.as_str() {
        text.to_string()
    } else if value.is_null() {
        "MISSING".to_string()
    } else {
        value.to_string()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use approval_service::ApprovalService;
    use chrono::{Duration, TimeZone};
    use contracts::{AnalysisHorizonV1, AnalysisObjectiveV1};
    use evidence_service::EvidenceService;
    use memory_provider::MemvidMemoryProvider;
    use policy_service::PolicyService;
    use surrealdb_access::connect_in_memory;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;
    use trading_core::{FixedClock, SequenceIdGenerator};

    use super::*;

    async fn spawn_source_server() -> (SocketAddr, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let address = listener.local_addr().expect("addr");
        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let mut buffer = [0_u8; 4096];
                let read = stream.read(&mut buffer).await.expect("read");
                let request = String::from_utf8_lossy(&buffer[..read]);
                let (content_type, body) = if request.contains("GET /fred") {
                    (
                        "application/json",
                        r#"{"observations":[{"date":"2026-01-01","value":"1.23"},{"date":"2026-02-01","value":"1.27"}]}"#,
                    )
                } else if request.contains("GET /imf") {
                    (
                        "application/xml",
                        r#"<CompactData><DataSet><Series><Obs TIME_PERIOD="2026-02" OBS_VALUE="42.0" /></Series></DataSet></CompactData>"#,
                    )
                } else if request.contains("GET /worldbank") {
                    (
                        "application/json",
                        r#"[{"page":1},[{"date":"2025","value": {"value": "2.1"}}]]"#,
                    )
                } else if request.contains("GET /bulletin") {
                    (
                        "text/html",
                        "<html><body><p>Reserve adequacy remains strong and funding volatility is contained.</p></body></html>",
                    )
                } else {
                    ("text/plain", "not found")
                };
                let status = if request.contains("/missing") {
                    "404 Not Found"
                } else {
                    "200 OK"
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.expect("write");
            }
        });
        (address, handle)
    }

    #[tokio::test]
    async fn knowledge_service_ingests_publishes_and_generates_analysis() {
        let (address, _server) = spawn_source_server().await;
        let repositories = connect_in_memory().await.expect("memory db");
        let dir = tempfile::tempdir().expect("tempdir");
        let service = KnowledgeService::new(
            MemvidMemoryProvider::new(dir.path()),
            Arc::new(FixedClock::new(
                Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0)
                    .single()
                    .expect("time"),
            )),
            Arc::new(SequenceIdGenerator::new("knowledge")),
        )
        .with_official_document_host("127.0.0.1");

        let ingested = service
            .ingest_sources_unchecked(
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-1".to_string(),
                    classification: Classification::Internal,
                    sources: vec![
                        KnowledgeSourceFetchSpecV1 {
                            source_id: "source-fred".to_string(),
                            kind: KnowledgeSourceKindV1::Fred,
                            title: "FRED FX series".to_string(),
                            country_area: "United States".to_string(),
                            url: format!("http://{address}/fred"),
                            series_name: Some("DXY".to_string()),
                            expected_format: KnowledgeDocumentFormatV1::Json,
                            release_lag: Some("T+1d".to_string()),
                            units: Some("index".to_string()),
                            transform: Some("level".to_string()),
                            notes: vec!["Primary source".to_string()],
                        },
                        KnowledgeSourceFetchSpecV1 {
                            source_id: "source-imf".to_string(),
                            kind: KnowledgeSourceKindV1::Imf,
                            title: "IMF BOP".to_string(),
                            country_area: "Japan".to_string(),
                            url: format!("http://{address}/imf"),
                            series_name: Some("BOP".to_string()),
                            expected_format: KnowledgeDocumentFormatV1::Xml,
                            release_lag: Some("T+30d".to_string()),
                            units: Some("USD bn".to_string()),
                            transform: Some("yoy".to_string()),
                            notes: vec!["Primary source".to_string()],
                        },
                        KnowledgeSourceFetchSpecV1 {
                            source_id: "source-wb".to_string(),
                            kind: KnowledgeSourceKindV1::WorldBank,
                            title: "World Bank external debt".to_string(),
                            country_area: "Brazil".to_string(),
                            url: format!("http://{address}/worldbank"),
                            series_name: Some("Debt".to_string()),
                            expected_format: KnowledgeDocumentFormatV1::Json,
                            release_lag: Some("T+90d".to_string()),
                            units: Some("% GDP".to_string()),
                            transform: Some("level".to_string()),
                            notes: vec!["Primary source".to_string()],
                        },
                        KnowledgeSourceFetchSpecV1 {
                            source_id: "source-doc".to_string(),
                            kind: KnowledgeSourceKindV1::OfficialDocument,
                            title: "Central bank bulletin".to_string(),
                            country_area: "Japan".to_string(),
                            url: format!("http://{address}/bulletin"),
                            series_name: None,
                            expected_format: KnowledgeDocumentFormatV1::Html,
                            release_lag: None,
                            units: None,
                            transform: None,
                            notes: vec!["Official bulletin".to_string()],
                        },
                    ],
                },
            )
            .await
            .expect("ingest");
        assert_eq!(ingested.len(), 4);

        let capsule = service
            .publish_capsule_unchecked(
                &repositories,
                KnowledgePublicationRequestV1 {
                    publication_id: "publication-1".to_string(),
                    capsule_id: "capsule-1".to_string(),
                    title: "GMF capsule".to_string(),
                    source_ids: ingested
                        .iter()
                        .map(|source| source.source_id.clone())
                        .collect(),
                    classification: Classification::Internal,
                    retention_class: "institutional_record".to_string(),
                },
            )
            .await
            .expect("publish");
        assert_eq!(capsule.source_count, 4);

        let analysis = service
            .generate_analysis(
                &repositories,
                MacroFinancialAnalysisRequestV1 {
                    analysis_id: "analysis-1".to_string(),
                    objective: AnalysisObjectiveV1::PolicyEval,
                    horizon: AnalysisHorizonV1::Nowcast,
                    coverage: AnalysisCoverageV1 {
                        countries: vec!["Japan".to_string(), "Brazil".to_string()],
                        regions: vec!["Asia".to_string(), "Latin America".to_string()],
                        currencies: vec!["JPY".to_string(), "BRL".to_string()],
                        fx_pairs: vec!["USD/JPY".to_string(), "USD/BRL".to_string()],
                        asset_classes: vec!["rates".to_string(), "credit".to_string()],
                    },
                    data_vintage: Some("2026-03-09".to_string()),
                    source_ids: Vec::new(),
                    capsule_id: Some(capsule.capsule_id.clone()),
                    classification: Classification::Internal,
                    constraints: SourceConstraintsV1::default(),
                },
            )
            .await
            .expect("analysis");

        assert_eq!(analysis.scenario_matrix.len(), 4);
        assert!(analysis
            .rendered_output
            .contains("[Output 1: Executive Brief]"));
        assert!(analysis.rendered_output.contains("TAIL_LIQUIDITY_EVENT"));
        assert!(service
            .latest_publication_status(&repositories)
            .await
            .expect("status")
            .is_some());
        assert!(service
            .load_analysis(&repositories, "analysis-1")
            .await
            .expect("load")
            .is_some());
    }

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source =
            include_str!("../../../enterprise/domains/data_knowledge/service_boundaries.toml");
        let boundary = service_boundary();

        assert_eq!(boundary.service_name, SERVICE_NAME);
        assert_eq!(boundary.domain, DOMAIN_NAME);
        for workflow in APPROVED_WORKFLOWS {
            assert!(source.contains(workflow));
        }
        for aggregate in OWNED_AGGREGATES {
            assert!(source.contains(aggregate));
        }
    }

    #[tokio::test]
    async fn workflow_context_can_gate_ingest_with_engine_authorization() {
        let (address, _server) = spawn_source_server().await;
        let repositories = connect_in_memory().await.expect("memory db");
        let dir = tempfile::tempdir().expect("tempdir");
        let service = KnowledgeService::new(
            MemvidMemoryProvider::new(dir.path()),
            Arc::new(FixedClock::new(Utc::now() + Duration::minutes(1))),
            Arc::new(SequenceIdGenerator::new("knowledge")),
        )
        .with_official_document_host("127.0.0.1");
        let mut engine = orchestrator::WorkflowEngine::new(
            PolicyService::institutional_default(),
            ApprovalService::default(),
            EvidenceService::default(),
        );
        let action = contracts::AgentActionRequestV1 {
            action_id: "action-1".to_string(),
            actor_ref: identity::ActorRef("agent:finance".to_string()),
            objective: "Compile macro-financial sources".to_string(),
            requested_workflow: "knowledge_publication".to_string(),
            impact_tier: contracts::ImpactTier::Tier0,
            classification: Classification::Internal,
            required_approver_roles: Vec::new(),
            policy_refs: vec!["policy.data_knowledge".to_string()],
        };
        let mut approved = None;
        let guarded_request = enforcement::GuardedMutationRequest {
            action_id: action.action_id.clone(),
            workflow_name: "knowledge_publication".to_string(),
            target_service: SERVICE_NAME.to_string(),
            target_aggregate: "knowledge_source".to_string(),
            actor_ref: action.actor_ref.clone(),
            impact_tier: action.impact_tier,
            classification: action.classification,
            policy_refs: action.policy_refs.clone(),
            required_approver_roles: action.required_approver_roles.clone(),
            environment: "prod".to_string(),
            cross_domain: false,
        };
        engine
            .execute_mutation(guarded_request, |context| {
                approved = Some(context.clone());
                Ok(())
            })
            .expect("authorize");
        let context = approved.expect("context");
        let ingested = service
            .ingest_sources(
                &context,
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-ctx".to_string(),
                    classification: Classification::Internal,
                    sources: vec![KnowledgeSourceFetchSpecV1 {
                        source_id: "source-fred".to_string(),
                        kind: KnowledgeSourceKindV1::Fred,
                        title: "FRED FX series".to_string(),
                        country_area: "United States".to_string(),
                        url: format!("http://{address}/fred"),
                        series_name: Some("DXY".to_string()),
                        expected_format: KnowledgeDocumentFormatV1::Json,
                        release_lag: Some("T+1d".to_string()),
                        units: Some("index".to_string()),
                        transform: Some("level".to_string()),
                        notes: vec!["Primary source".to_string()],
                    }],
                },
            )
            .await
            .expect("ingest");
        assert_eq!(ingested.len(), 1);
    }
}

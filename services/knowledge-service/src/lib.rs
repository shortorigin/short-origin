use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[cfg(test)]
use chrono::Utc;
use contracts::{
    AnalysisAssumptionV1, AnalysisCoverageV1, AnalysisImplicationsV1, ClaimEvidenceV1, ClaimKindV1,
    Classification, ConfidenceV1, DataRegisterEntryV1, DirectionalBiasV1, DriverBucketV1,
    ExecutiveBriefV1, ExternalAccountsBalanceSheetMapV1, FxDriverAssessmentV1,
    GlobalLiquidityFundingConditionsV1, GlobalLiquidityPhaseV1, InferenceStepV1,
    KnowledgeAppendixV1, KnowledgeCapsuleV1, KnowledgeDocumentFormatV1, KnowledgeEdgeV1,
    KnowledgeEvidenceUseV1, KnowledgePublicationRequestV1, KnowledgePublicationStatusV1,
    KnowledgeRelationshipV1, KnowledgeSourceFetchSpecV1, KnowledgeSourceIngestRequestV1,
    KnowledgeSourceKindV1, KnowledgeSourceProvenanceV1, KnowledgeSourceV1,
    MacroFinancialAnalysisRequestV1, MacroFinancialAnalysisV1, MechanismMapV1, PipelineStepIdV1,
    PipelineStepTraceV1, PolicyFrictionObservationV1, PolicyRegimeDiagnosisV1, ProbabilityV1,
    ProblemContractV1, RankedRiskV1, RiskRegisterEntryV1, ScenarioCaseV1, ScenarioKindV1,
    ServiceBoundaryV1, SignalMagnitudeV1, SignalSummaryEntryV1, SourceConstraintsV1,
    SourceGovernanceDecisionV1, SovereignSystemicRiskV1, TransmissionChannelV1,
    WatchlistIndicatorV1,
};
use enforcement::ApprovedMutationContext;
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use events::{EventEnvelopeV1, RecordedEventV1};
use governed_storage::KnowledgeStore;
use identity::{ActorRef, EvidenceId, ServiceId, WorkflowId};
use memory_provider::{
    CapsuleBuildRequest, CapsuleDocument, CapsuleSearchRequest, KnowledgeMemoryProvider,
};
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use telemetry::DecisionRef;
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
const ANALYSIS_AS_OF_DATE: &str = "2026-03-09";

fn service_id() -> ServiceId {
    SERVICE_NAME.into()
}

fn knowledge_publication_workflow_id() -> WorkflowId {
    "knowledge_publication".into()
}

fn knowledge_context(operation: &str) -> OperationContext {
    OperationContext::new("services/knowledge-service", operation).with_service_id(service_id())
}

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

    pub async fn ingest_sources<R>(
        &self,
        context: &ApprovedMutationContext,
        repositories: &R,
        request: KnowledgeSourceIngestRequestV1,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>>
    where
        R: KnowledgeStore,
    {
        context.assert_workflow(&knowledge_publication_workflow_id())?;
        context.assert_target_service(&service_id())?;
        self.ingest_sources_unchecked(repositories, request).await
    }

    pub async fn publish_capsule<R>(
        &self,
        context: &ApprovedMutationContext,
        repositories: &R,
        request: KnowledgePublicationRequestV1,
    ) -> InstitutionalResult<KnowledgeCapsuleV1>
    where
        R: KnowledgeStore,
    {
        context.assert_workflow(&knowledge_publication_workflow_id())?;
        context.assert_target_service(&service_id())?;
        self.publish_capsule_unchecked(repositories, request).await
    }

    pub async fn generate_analysis<R>(
        &self,
        repositories: &R,
        request: MacroFinancialAnalysisRequestV1,
    ) -> InstitutionalResult<MacroFinancialAnalysisV1>
    where
        R: KnowledgeStore,
    {
        let sources = self.resolve_sources(repositories, &request).await?;
        let source_governance =
            validate_selected_sources_against_constraints(&sources, &request.constraints)?;
        let evidence_sources = sources
            .iter()
            .filter(|source| source.evidence_use == KnowledgeEvidenceUseV1::Evidence)
            .cloned()
            .collect::<Vec<_>>();
        if evidence_sources.is_empty() && direct_inputs_are_empty(&request.direct_inputs) {
            return Err(InstitutionalError::not_found(
                knowledge_context("generate_analysis"),
                "macro-financial analysis sources or direct inputs",
            ));
        }
        validate_required_output_format(&request.constraints)?;

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
        let problem_contract = build_problem_contract(&request, &evidence_sources);
        let data_register = build_data_register(&evidence_sources, &request.direct_inputs);
        let external_accounts_map =
            build_external_accounts_map(&evidence_sources, &request.direct_inputs);
        let policy_regime_diagnosis =
            build_policy_regime_diagnosis(&evidence_sources, &request.direct_inputs);
        let global_liquidity_funding =
            build_global_liquidity_funding_conditions(&evidence_sources, &request.direct_inputs);
        let driver_decomposition = build_driver_decomposition(
            &evidence_sources,
            &request.direct_inputs,
            &request.coverage,
            global_liquidity_funding.phase,
        );
        let sovereign_systemic_risk = build_sovereign_systemic_risk(
            &evidence_sources,
            &request.direct_inputs,
            &driver_decomposition,
        );
        let assumptions = build_analysis_assumptions(&request.constraints, &source_governance);
        let inference_steps = build_inference_steps(
            &problem_contract,
            &external_accounts_map,
            &policy_regime_diagnosis,
            &global_liquidity_funding,
            &sovereign_systemic_risk,
            &assumptions,
        );
        let scenario_matrix = build_scenario_matrix(
            &request,
            &driver_decomposition,
            &external_accounts_map,
            &policy_regime_diagnosis,
            &global_liquidity_funding,
            &sovereign_systemic_risk,
        );
        let pipeline_trace = build_pipeline_trace(
            &problem_contract,
            &data_register,
            &external_accounts_map,
            &policy_regime_diagnosis,
            &global_liquidity_funding,
            &data_vintage,
            &driver_decomposition,
            &sovereign_systemic_risk,
            &scenario_matrix,
        );
        let executive_brief = build_executive_brief(
            &request,
            &evidence_sources,
            &problem_contract,
            &data_vintage,
            &driver_decomposition,
            &source_governance,
            &inference_steps,
            &retrieval_context,
        );
        let mechanism_map = build_mechanism_map(&external_accounts_map, &global_liquidity_funding);
        let risk_register = build_risk_register(
            &driver_decomposition,
            &sovereign_systemic_risk,
            &evidence_sources,
        );
        let knowledge_appendix = build_knowledge_appendix(&request.constraints, &assumptions);
        let claim_evidence = build_claim_evidence(
            &executive_brief,
            &scenario_matrix,
            &risk_register,
            &knowledge_appendix,
            &evidence_sources,
            &data_register,
            &inference_steps,
        );

        let mut analysis = MacroFinancialAnalysisV1 {
            analysis_id: request.analysis_id.clone(),
            generated_at: self.clock.now(),
            trace_ref: format!("analysis::{}", request.analysis_id),
            objective: request.objective,
            horizon: request.horizon,
            coverage: request.coverage.clone(),
            problem_contract: problem_contract.clone(),
            data_vintage,
            required_inputs: problem_contract.required_inputs.clone(),
            dependent_variables: problem_contract.dependent_variables.clone(),
            global_liquidity_phase: global_liquidity_funding.phase,
            global_liquidity_funding,
            external_accounts_map,
            policy_regime_diagnosis,
            driver_decomposition,
            sovereign_systemic_risk,
            executive_brief,
            data_register,
            mechanism_map,
            scenario_matrix,
            risk_register,
            knowledge_appendix,
            source_governance,
            assumptions,
            inference_steps,
            claim_evidence,
            pipeline_trace,
            source_ids: evidence_sources
                .iter()
                .map(|source| source.source_id.clone())
                .collect(),
            capsule_id: request.capsule_id.clone(),
            rendered_output: String::new(),
            retrieval_context,
        };
        analysis.rendered_output = render_analysis(&analysis);

        repositories.store_analysis(analysis.clone()).await?;
        repositories
            .store_evidence(
                format!("evidence::{}", analysis.analysis_id),
                contracts::EvidenceManifestV1 {
                    evidence_id: EvidenceId::from(format!("evidence::{}", analysis.analysis_id)),
                    producer: SERVICE_NAME.to_string(),
                    artifact_hash: sha256_hex(analysis.rendered_output.as_bytes()),
                    storage_ref: format!("knowledge-store:analysis/{}", analysis.analysis_id),
                    retention_class: "institutional_record".to_string(),
                    classification: request.classification,
                    related_decision_refs: analysis
                        .claim_evidence
                        .iter()
                        .map(|claim| DecisionRef::from(claim.claim_id.clone()))
                        .chain(
                            analysis
                                .inference_steps
                                .iter()
                                .map(|inference| DecisionRef::from(inference.inference_id.clone())),
                        )
                        .collect(),
                },
            )
            .await?;
        repositories
            .append_event(
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
        for source in &evidence_sources {
            repositories
                .store_edge(KnowledgeEdgeV1 {
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
                .store_edge(KnowledgeEdgeV1 {
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

    pub async fn load_analysis<R>(
        &self,
        repositories: &R,
        analysis_id: &str,
    ) -> InstitutionalResult<Option<MacroFinancialAnalysisV1>>
    where
        R: KnowledgeStore,
    {
        repositories.load_analysis(analysis_id).await
    }

    pub async fn latest_publication_status<R>(
        &self,
        repositories: &R,
    ) -> InstitutionalResult<Option<KnowledgePublicationStatusV1>>
    where
        R: KnowledgeStore,
    {
        repositories.latest_publication_status().await
    }

    pub async fn ingest_sources_unchecked<R>(
        &self,
        repositories: &R,
        request: KnowledgeSourceIngestRequestV1,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>>
    where
        R: KnowledgeStore,
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
                &request.constraints,
                spec,
                bytes.as_ref(),
                &mime_type,
            )?;
            repositories.store_source(source.clone()).await?;
            repositories
                .append_event(
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

    pub async fn publish_capsule_unchecked<R>(
        &self,
        repositories: &R,
        request: KnowledgePublicationRequestV1,
    ) -> InstitutionalResult<KnowledgeCapsuleV1>
    where
        R: KnowledgeStore,
    {
        let sources = repositories.load_sources(&request.source_ids).await?;
        if sources.len() != request.source_ids.len() {
            return Err(InstitutionalError::not_found(
                knowledge_context("publish_capsule_unchecked"),
                "one or more knowledge sources",
            ));
        }
        let source_governance =
            validate_selected_sources_against_constraints(&sources, &request.constraints)?;

        let documents = sources
            .iter()
            .map(|source| CapsuleDocument {
                document_id: source.source_id.clone(),
                title: source.title.clone(),
                uri: format!("knowledge://source/{}", source.source_id),
                content: source.content_text.clone(),
                metadata: BTreeMap::from([
                    ("source_id".to_string(), source.source_id.clone()),
                    (
                        "provider".to_string(),
                        source_kind_label(source.kind).to_string(),
                    ),
                    ("country_area".to_string(), source.country_area.clone()),
                ]),
                search_text: Some(source.content_text.clone()),
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
            source_count: sources.len(),
            storage_ref: build.storage_ref,
            artifact_hash: build.artifact_hash,
            version: build.version,
            memvid_version: build.memvid_version,
            published_at: self.clock.now(),
            classification: request.classification,
            retention_class: request.retention_class,
        };
        repositories.store_capsule(capsule.clone()).await?;
        repositories
            .append_event(
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
        for source in &sources {
            repositories
                .store_edge(KnowledgeEdgeV1 {
                    edge_id: self.ids.next_id(),
                    from_id: capsule.capsule_id.clone(),
                    to_id: source.source_id.clone(),
                    relationship: KnowledgeRelationshipV1::DerivedFrom,
                    rationale: source_governance
                        .iter()
                        .find(|decision| decision.source_id == source.source_id)
                        .map_or_else(
                            || "Capsule compiled from governed source text.".to_string(),
                            |decision| decision.reasons.join(" | "),
                        ),
                })
                .await?;
        }
        Ok(capsule)
    }

    async fn resolve_sources<R>(
        &self,
        repositories: &R,
        request: &MacroFinancialAnalysisRequestV1,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>>
    where
        R: KnowledgeStore,
    {
        if !request.source_ids.is_empty() {
            let sources = repositories.load_sources(&request.source_ids).await?;
            if sources.len() != request.source_ids.len() {
                return Err(InstitutionalError::not_found(
                    knowledge_context("resolve_sources"),
                    "one or more requested knowledge sources",
                ));
            }
            return Ok(sources);
        }
        if let Some(capsule_id) = &request.capsule_id
            && let Some(capsule) = repositories.load_capsule(capsule_id).await?
        {
            let sources = repositories.load_sources(&capsule.source_ids).await?;
            if sources.len() != capsule.source_ids.len() {
                return Err(InstitutionalError::not_found(
                    knowledge_context("resolve_sources"),
                    "one or more capsule knowledge sources",
                ));
            }
            return Ok(sources);
        }
        Ok(Vec::new())
    }

    fn normalize_source(
        &self,
        ingestion_id: &str,
        classification: Classification,
        constraints: &SourceConstraintsV1,
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
        let governance = evaluate_source_governance(
            spec.source_id.as_str(),
            spec.title.as_str(),
            spec.url.as_str(),
            source_domain.as_str(),
            spec.kind,
            constraints,
        )?;

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
            provenance_tier: governance.provenance_tier,
            evidence_use: governance.evidence_use,
            last_observation: parsed.last_observation,
            units: spec.units.clone().or(parsed.units),
            transform: spec.transform.clone().or(parsed.transform),
            release_lag: spec.release_lag.clone().or(parsed.release_lag),
            quality_flags: parsed.quality_flags,
            notes: spec.notes.clone(),
            governance_notes: governance.reasons.clone(),
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
    if is_social_media_host(host) {
        return Err(InstitutionalError::policy_denied(
            knowledge_context("validate_source_url"),
            format!("social media host `{host}` is not admissible evidence"),
        ));
    }
    let allowed = match spec.kind {
        KnowledgeSourceKindV1::Imf => host_matches(host, "imf.org"),
        KnowledgeSourceKindV1::Bis => host_matches(host, "bis.org"),
        KnowledgeSourceKindV1::Fsb => host_matches(host, "fsb.org"),
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
        KnowledgeSourceKindV1::ResearchPaper => {
            host.rsplit('.')
                .next()
                .is_some_and(|segment| segment.eq_ignore_ascii_case("edu"))
                || host_matches(host, "nber.org")
                || host_matches(host, "imf.org")
                || host_matches(host, "bis.org")
                || host_matches(host, "worldbank.org")
                || host_matches(host, "fsb.org")
                || official_document_hosts
                    .iter()
                    .any(|allowed_host| host_matches(host, allowed_host))
        }
        KnowledgeSourceKindV1::SecondaryContext => !is_social_media_host(host),
    };
    if allowed
        || official_document_hosts
            .iter()
            .any(|allowed_host| host_matches(host, allowed_host))
    {
        Ok(())
    } else {
        Err(InstitutionalError::policy_denied(
            knowledge_context("validate_source_url"),
            format!("source host `{host}` is not allowed for {:?}", spec.kind),
        ))
    }
}

fn is_social_media_host(host: &str) -> bool {
    [
        "x.com",
        "twitter.com",
        "facebook.com",
        "instagram.com",
        "linkedin.com",
        "tiktok.com",
        "youtube.com",
        "reddit.com",
    ]
    .iter()
    .any(|candidate| host_matches(host, candidate))
}

fn evaluate_source_governance(
    source_id: &str,
    title: &str,
    source_url: &str,
    source_domain: &str,
    kind: KnowledgeSourceKindV1,
    constraints: &SourceConstraintsV1,
) -> InstitutionalResult<SourceGovernanceDecisionV1> {
    let provenance_tier = match kind {
        KnowledgeSourceKindV1::SecondaryContext => KnowledgeSourceProvenanceV1::Secondary,
        _ => KnowledgeSourceProvenanceV1::Primary,
    };
    let evidence_use = match provenance_tier {
        KnowledgeSourceProvenanceV1::Primary => KnowledgeEvidenceUseV1::Evidence,
        KnowledgeSourceProvenanceV1::Secondary => KnowledgeEvidenceUseV1::ContextOnly,
    };
    let mut reasons = vec![format!(
        "Source classified as {} with {} use.",
        provenance_tier.directive_label(),
        evidence_use.directive_label()
    )];

    let allowed = constraints.allowed_sources.is_empty()
        || constraints.allowed_sources.iter().any(|constraint| {
            source_matches_constraint(
                constraint,
                source_id,
                title,
                source_url,
                source_domain,
                kind,
                provenance_tier,
            )
        });
    if !allowed {
        return Err(InstitutionalError::policy_denied(
            knowledge_context("validate_source_selection"),
            format!("source `{source_id}` is outside the allowed source set"),
        ));
    }
    if constraints.forbidden_sources.iter().any(|constraint| {
        source_matches_constraint(
            constraint,
            source_id,
            title,
            source_url,
            source_domain,
            kind,
            provenance_tier,
        )
    }) {
        return Err(InstitutionalError::policy_denied(
            knowledge_context("validate_source_selection"),
            format!("source `{source_id}` matched a forbidden source rule"),
        ));
    }

    if provenance_tier == KnowledgeSourceProvenanceV1::Secondary {
        reasons.push(
            "Secondary sources are context-only and cannot support evidence claims.".to_string(),
        );
    }

    Ok(SourceGovernanceDecisionV1 {
        source_id: source_id.to_string(),
        source_domain: source_domain.to_string(),
        provenance_tier,
        evidence_use,
        accepted: true,
        reasons,
    })
}

fn source_matches_constraint(
    constraint: &str,
    source_id: &str,
    title: &str,
    source_url: &str,
    source_domain: &str,
    kind: KnowledgeSourceKindV1,
    provenance_tier: KnowledgeSourceProvenanceV1,
) -> bool {
    let needle = constraint.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return false;
    }

    [
        source_id.to_ascii_lowercase(),
        title.to_ascii_lowercase(),
        source_url.to_ascii_lowercase(),
        source_domain.to_ascii_lowercase(),
        format!("{kind:?}").to_ascii_lowercase(),
        provenance_tier.directive_label().to_ascii_lowercase(),
    ]
    .iter()
    .any(|candidate| candidate.contains(&needle))
}

fn validate_selected_sources_against_constraints(
    sources: &[KnowledgeSourceV1],
    constraints: &SourceConstraintsV1,
) -> InstitutionalResult<Vec<SourceGovernanceDecisionV1>> {
    sources
        .iter()
        .map(|source| {
            evaluate_source_governance(
                source.source_id.as_str(),
                source.title.as_str(),
                source.source_url.as_str(),
                source.source_domain.as_str(),
                source.kind,
                constraints,
            )
        })
        .collect()
}

fn validate_required_output_format(constraints: &SourceConstraintsV1) -> InstitutionalResult<()> {
    if let Some(format) = &constraints.required_output_format {
        let normalized = format.trim().to_ascii_lowercase();
        if !normalized.is_empty()
            && normalized != "default"
            && normalized != "strict"
            && normalized != "strict_template"
        {
            return Err(InstitutionalError::validation(
                knowledge_context("validate_required_output_format"),
                format!("required output format `{format}` is not supported"),
            ));
        }
    }
    Ok(())
}

fn direct_inputs_are_empty(inputs: &contracts::MacroFinancialDirectInputsV1) -> bool {
    inputs.fx_levels_returns.is_empty()
        && inputs.rates_yields.is_empty()
        && inputs.inflation_growth_terms_trade_fiscal.is_empty()
        && inputs.balance_of_payments_iip.is_empty()
        && inputs.cross_border_banking_credit.is_empty()
        && inputs
            .portfolio_flow_positions_reserve_composition
            .is_empty()
        && inputs.funding_hedging_indicators.is_empty()
        && inputs.market_stress_proxies.is_empty()
        && inputs.policy_communications.is_empty()
        && inputs.geopolitical_timeline.is_empty()
        && inputs.inline_documents.is_empty()
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
        KnowledgeSourceKindV1::Imf | KnowledgeSourceKindV1::Bis | KnowledgeSourceKindV1::Fsb => {
            parse_xml_series_metadata(bytes)
        }
        KnowledgeSourceKindV1::OfficialDocument
        | KnowledgeSourceKindV1::ResearchPaper
        | KnowledgeSourceKindV1::SecondaryContext => Ok(ParsedSourceMetadata {
            quality_flags: vec![contracts::QualityFlagV1::ProxyUsed],
            provider_metadata: BTreeMap::from([(
                "document_type".to_string(),
                match spec.kind {
                    KnowledgeSourceKindV1::OfficialDocument => "official_document",
                    KnowledgeSourceKindV1::ResearchPaper => "research_paper",
                    KnowledgeSourceKindV1::SecondaryContext => "secondary_context",
                    _ => "document",
                }
                .to_string(),
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

fn required_input_catalog() -> [(&'static str, &'static str); 9] {
    [
        (
            "FX levels/returns",
            "MISSING: provide FX levels/returns for the covered currencies or FX pairs.",
        ),
        (
            "Rates and yields",
            "MISSING: provide rates/yields or policy-path data for the covered jurisdictions.",
        ),
        (
            "Inflation, growth, terms of trade, and fiscal metrics",
            "MISSING: provide inflation, growth, terms-of-trade, and fiscal metrics for the covered jurisdictions.",
        ),
        (
            "Balance of payments and IIP components",
            "MISSING: provide balance of payments and IIP components.",
        ),
        (
            "Cross-border banking and credit measures",
            "MISSING: provide cross-border banking/credit measures or equivalent official series.",
        ),
        (
            "Portfolio flow/position measures; reserve composition",
            "MISSING: provide portfolio flow/position measures and reserve composition or reserve-change series.",
        ),
        (
            "Funding and hedging indicators",
            "MISSING: provide funding/hedging indicators such as FX swaps, basis, or on/offshore spreads.",
        ),
        (
            "Market stress proxies",
            "MISSING: provide market stress proxies such as volatility, credit spreads, or funding-stress markers.",
        ),
        (
            "Policy communications and geopolitical timeline",
            "MISSING: provide policy communications and geopolitical/event timeline inputs.",
        ),
    ]
}

fn infer_present_inputs(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> BTreeSet<String> {
    let mut present = BTreeSet::new();
    if !inputs.fx_levels_returns.is_empty() {
        present.insert("FX levels/returns".to_string());
    }
    if !inputs.rates_yields.is_empty() {
        present.insert("Rates and yields".to_string());
    }
    if !inputs.inflation_growth_terms_trade_fiscal.is_empty() {
        present.insert("Inflation, growth, terms of trade, and fiscal metrics".to_string());
    }
    if !inputs.balance_of_payments_iip.is_empty() {
        present.insert("Balance of payments and IIP components".to_string());
    }
    if !inputs.cross_border_banking_credit.is_empty() {
        present.insert("Cross-border banking and credit measures".to_string());
    }
    if !inputs
        .portfolio_flow_positions_reserve_composition
        .is_empty()
    {
        present.insert("Portfolio flow/position measures; reserve composition".to_string());
    }
    if !inputs.funding_hedging_indicators.is_empty() {
        present.insert("Funding and hedging indicators".to_string());
    }
    if !inputs.market_stress_proxies.is_empty() {
        present.insert("Market stress proxies".to_string());
    }
    if !inputs.policy_communications.is_empty() || !inputs.geopolitical_timeline.is_empty() {
        present.insert("Policy communications and geopolitical timeline".to_string());
    }
    for source in sources {
        let haystack = format!(
            "{} {} {}",
            source.title,
            source.series_name.clone().unwrap_or_default(),
            source.content_text
        )
        .to_ascii_lowercase();
        if source.kind == KnowledgeSourceKindV1::Fred || haystack.contains("fx") {
            present.insert("FX levels/returns".to_string());
        }
        if haystack.contains("rate") || haystack.contains("yield") || haystack.contains("policy") {
            present.insert("Rates and yields".to_string());
        }
        if matches!(
            source.kind,
            KnowledgeSourceKindV1::Imf | KnowledgeSourceKindV1::WorldBank
        ) || haystack.contains("current account")
            || haystack.contains("iip")
        {
            present.insert("Balance of payments and IIP components".to_string());
        }
        if source.kind == KnowledgeSourceKindV1::Bis || haystack.contains("cross-border") {
            present.insert("Cross-border banking and credit measures".to_string());
        }
        if haystack.contains("portfolio")
            || haystack.contains("reserve")
            || haystack.contains("capital flow")
        {
            present.insert("Portfolio flow/position measures; reserve composition".to_string());
        }
        if haystack.contains("funding") || haystack.contains("basis") || haystack.contains("swap") {
            present.insert("Funding and hedging indicators".to_string());
        }
        if haystack.contains("stress") || haystack.contains("spread") || haystack.contains("vol") {
            present.insert("Market stress proxies".to_string());
        }
        if haystack.contains("statement")
            || haystack.contains("minutes")
            || haystack.contains("sanction")
            || haystack.contains("tariff")
            || haystack.contains("conflict")
        {
            present.insert("Policy communications and geopolitical timeline".to_string());
        }
    }
    present
}

fn combined_corpus(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> String {
    let mut corpus = sources
        .iter()
        .map(|source| source.content_text.clone())
        .collect::<Vec<_>>();
    corpus.extend(
        inputs
            .inline_documents
            .iter()
            .map(|document| document.text.clone()),
    );
    corpus.extend(
        inputs
            .policy_communications
            .iter()
            .map(|item| format!("{} {}", item.title, item.summary)),
    );
    corpus.extend(
        inputs
            .geopolitical_timeline
            .iter()
            .map(|item| item.summary.clone()),
    );
    corpus.join("\n").to_ascii_lowercase()
}

fn build_problem_contract(
    request: &MacroFinancialAnalysisRequestV1,
    sources: &[KnowledgeSourceV1],
) -> ProblemContractV1 {
    let present_inputs = infer_present_inputs(sources, &request.direct_inputs);
    let missing_inputs = required_input_catalog()
        .iter()
        .filter(|(label, _)| !present_inputs.contains(*label))
        .map(|(_, detail)| (*detail).to_string())
        .collect::<Vec<_>>();

    ProblemContractV1 {
        objective: request.objective,
        horizon: request.horizon,
        target_countries: request.coverage.countries.clone(),
        target_regions: request.coverage.regions.clone(),
        target_currencies: request.coverage.currencies.clone(),
        target_fx_pairs: request.coverage.fx_pairs.clone(),
        asset_classes: request.coverage.asset_classes.clone(),
        dependent_variables: dependent_variables(),
        required_inputs: required_inputs(),
        missing_inputs,
    }
}

fn build_external_accounts_map(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> ExternalAccountsBalanceSheetMapV1 {
    let corpus = combined_corpus(sources, inputs);
    let mut missing_inputs = Vec::new();
    if inputs.balance_of_payments_iip.is_empty()
        && !corpus.contains("current account")
        && !corpus.contains("iip")
    {
        missing_inputs.push(
            "MISSING: provide BPM/IIP detail for current account, financial account, and external debt structure."
                .to_string(),
        );
    }
    ExternalAccountsBalanceSheetMapV1 {
        current_account_pressures: if missing_inputs.is_empty() {
            "Current account pressure is assessed from BPM/IIP-consistent balance-of-payments evidence."
                .to_string()
        } else {
            missing_inputs[0].clone()
        },
        financial_account_decomposition:
            "Financial account decomposition addresses direct, portfolio, other investment, and reserves."
                .to_string(),
        external_debt_structure: if inputs.balance_of_payments_iip.is_empty() {
            "MISSING: provide external debt structure by currency and maturity.".to_string()
        } else {
            "External debt structure is inferred from provided balance-sheet and external-account inputs."
                .to_string()
        },
        currency_mismatch_indicators: if inputs.balance_of_payments_iip.is_empty() {
            "MISSING: provide currency mismatch indicators or foreign-currency liability splits."
                .to_string()
        } else {
            "Currency mismatch indicators are tracked through liability currency composition and reserve buffers."
                .to_string()
        },
        marginal_financer:
            "Portfolio and banking channels are treated as marginal financers before reserves."
                .to_string(),
        flow_reversal_vulnerability:
            "Portfolio and banking flows are expected to reverse before direct investment in stress."
                .to_string(),
        missing_inputs,
    }
}

fn build_policy_regime_diagnosis(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> PolicyRegimeDiagnosisV1 {
    let corpus = combined_corpus(sources, inputs);
    let mut missing_inputs = Vec::new();
    if inputs.policy_communications.is_empty()
        && !corpus.contains("statement")
        && !corpus.contains("minutes")
        && !corpus.contains("intervention")
    {
        missing_inputs.push(
            "MISSING: provide central-bank statements, minutes, or intervention evidence."
                .to_string(),
        );
    }
    PolicyRegimeDiagnosisV1 {
        monetary_policy_regime: if corpus.contains("inflation") || corpus.contains("policy") {
            "Inflation-sensitive monetary-policy regime signals are present.".to_string()
        } else {
            "MISSING: provide policy communications to classify the monetary regime precisely."
                .to_string()
        },
        credibility_signals:
            "Credibility is read through communication consistency, inflation language, and reserve usage."
                .to_string(),
        exchange_rate_regime: if corpus.contains("intervention") || corpus.contains("reserve") {
            "Managed or intervention-aware exchange-rate behavior is relevant.".to_string()
        } else {
            "MISSING: provide exchange-rate intervention or regime evidence.".to_string()
        },
        intervention_pattern:
            "Intervention pattern is inferred from reserve references, official documents, and communications."
                .to_string(),
        frictions: vec![
            PolicyFrictionObservationV1 {
                friction: "Shallow/illiquid FX markets".to_string(),
                observable_indicators: vec![
                    "FX volatility".to_string(),
                    "Bid/ask widening".to_string(),
                    "Basis dislocation".to_string(),
                ],
                confidence: ConfidenceV1::Moderate,
            },
            PolicyFrictionObservationV1 {
                friction: "Unhedged FX exposures on balance sheets".to_string(),
                observable_indicators: vec![
                    "Foreign-currency liabilities".to_string(),
                    "Hedging demand".to_string(),
                ],
                confidence: ConfidenceV1::Weak,
            },
            PolicyFrictionObservationV1 {
                friction: "Inflation expectations de-anchoring / exchange rate pass-through risk"
                    .to_string(),
                observable_indicators: vec![
                    "Inflation expectations".to_string(),
                    "Policy guidance".to_string(),
                    "FX pass-through commentary".to_string(),
                ],
                confidence: ConfidenceV1::Moderate,
            },
        ],
        missing_inputs,
    }
}

fn build_global_liquidity_funding_conditions(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> GlobalLiquidityFundingConditionsV1 {
    let phase = determine_global_liquidity_phase(sources, inputs);
    let dominant_transmission_channel = if !inputs.cross_border_banking_credit.is_empty()
        || sources
            .iter()
            .any(|source| source.kind == KnowledgeSourceKindV1::Bis)
    {
        TransmissionChannelV1::CrossBorderBankCredit
    } else if !inputs
        .portfolio_flow_positions_reserve_composition
        .is_empty()
    {
        TransmissionChannelV1::BondMarketGlobalPortfolio
    } else {
        TransmissionChannelV1::DerivativesFundingStress
    };
    let mut missing_inputs = Vec::new();
    if inputs.funding_hedging_indicators.is_empty()
        && inputs.market_stress_proxies.is_empty()
        && !combined_corpus(sources, inputs).contains("basis")
    {
        missing_inputs.push(
            "MISSING: provide basis, swap, or stress-proxy inputs to validate funding conditions."
                .to_string(),
        );
    }
    GlobalLiquidityFundingConditionsV1 {
        phase,
        dominant_transmission_channel,
        dollar_funding_stress_state: format!(
            "Dollar funding stress is classified as {}.",
            phase.directive_label()
        ),
        backstop_availability:
            "Backstop availability is inferred from reserves, official facilities, and institutional flexibility."
                .to_string(),
        missing_inputs,
    }
}

fn build_sovereign_systemic_risk(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
    driver_decomposition: &[FxDriverAssessmentV1],
) -> SovereignSystemicRiskV1 {
    let mut missing_inputs = Vec::new();
    if inputs.inflation_growth_terms_trade_fiscal.is_empty()
        && !combined_corpus(sources, inputs).contains("debt")
    {
        missing_inputs.push(
            "MISSING: provide fiscal metrics, debt sustainability inputs, or financing-need indicators."
                .to_string(),
        );
    }
    SovereignSystemicRiskV1 {
        debt_sustainability_state: if missing_inputs.is_empty() {
            "Debt sustainability is qualitatively monitored through fiscal, spread, and reserve signals."
                .to_string()
        } else {
            missing_inputs[0].clone()
        },
        gross_financing_needs:
            "Gross financing needs are inferred from rollover pressure, spreads, and reserve/funding commentary."
                .to_string(),
        rollover_risk: "Rollover risk rises when flow shocks and funding premia deteriorate together."
            .to_string(),
        sovereign_bank_nonbank_nexus:
            "Sovereign-bank/nonbank nexus risk is tracked through spread widening and funding spillovers."
                .to_string(),
        key_amplifiers: vec![
            "Leverage".to_string(),
            "Liquidity mismatch".to_string(),
            "Short-term funding dependence".to_string(),
        ],
        cross_border_spillovers: format!(
            "Cross-border spillovers propagate through {} and {}.",
            driver_bucket_label(driver_decomposition[1].bucket),
            driver_bucket_label(driver_decomposition[4].bucket)
        ),
        missing_inputs,
    }
}

fn build_analysis_assumptions(
    constraints: &SourceConstraintsV1,
    source_governance: &[SourceGovernanceDecisionV1],
) -> Vec<AnalysisAssumptionV1> {
    let mut assumptions = vec![
        AnalysisAssumptionV1 {
            assumption_id: "A1".to_string(),
            text: "Official and primary sources dominate evidence claims unless explicitly missing."
                .to_string(),
            stable: true,
        },
        AnalysisAssumptionV1 {
            assumption_id: "A2".to_string(),
            text: "Portfolio and banking channels are treated as marginal financers before reserves."
                .to_string(),
            stable: true,
        },
        AnalysisAssumptionV1 {
            assumption_id: "A3".to_string(),
            text: "Recommendations remain generic and framework-level rather than personalized advice."
                .to_string(),
            stable: true,
        },
    ];
    if !constraints.allowed_sources.is_empty() {
        assumptions.push(AnalysisAssumptionV1 {
            assumption_id: "A4".to_string(),
            text: format!(
                "Allowed source constraints were {}.",
                constraints.allowed_sources.join(", ")
            ),
            stable: true,
        });
    }
    if !constraints.forbidden_sources.is_empty() {
        assumptions.push(AnalysisAssumptionV1 {
            assumption_id: "A5".to_string(),
            text: format!(
                "Forbidden source constraints were {}.",
                constraints.forbidden_sources.join(", ")
            ),
            stable: true,
        });
    }
    if source_governance
        .iter()
        .any(|decision| decision.provenance_tier == KnowledgeSourceProvenanceV1::Secondary)
    {
        assumptions.push(AnalysisAssumptionV1 {
            assumption_id: "A6".to_string(),
            text: "Secondary sources are context-only and cannot anchor evidence claims."
                .to_string(),
            stable: true,
        });
    }
    assumptions
}

fn build_inference_steps(
    problem_contract: &ProblemContractV1,
    external_accounts_map: &ExternalAccountsBalanceSheetMapV1,
    policy_regime_diagnosis: &PolicyRegimeDiagnosisV1,
    global_liquidity_funding: &GlobalLiquidityFundingConditionsV1,
    sovereign_systemic_risk: &SovereignSystemicRiskV1,
    assumptions: &[AnalysisAssumptionV1],
) -> Vec<InferenceStepV1> {
    let assumption_ids = assumptions
        .iter()
        .map(|assumption| assumption.assumption_id.clone())
        .collect::<Vec<_>>();
    vec![
        InferenceStepV1 {
            inference_id: "INF-01".to_string(),
            label: "Marginal financer".to_string(),
            assumption_ids: assumption_ids.clone(),
            inputs_used: vec![
                problem_contract.objective.directive_label().to_string(),
                external_accounts_map
                    .financial_account_decomposition
                    .clone(),
            ],
            resulting_judgment: external_accounts_map.marginal_financer.clone(),
        },
        InferenceStepV1 {
            inference_id: "INF-02".to_string(),
            label: "Global liquidity phase".to_string(),
            assumption_ids: assumption_ids.clone(),
            inputs_used: vec![
                global_liquidity_funding.phase.directive_label().to_string(),
                global_liquidity_funding
                    .dominant_transmission_channel
                    .directive_label()
                    .to_string(),
            ],
            resulting_judgment: global_liquidity_funding.dollar_funding_stress_state.clone(),
        },
        InferenceStepV1 {
            inference_id: "INF-03".to_string(),
            label: "Policy regime and credibility".to_string(),
            assumption_ids: assumption_ids.clone(),
            inputs_used: vec![
                policy_regime_diagnosis.monetary_policy_regime.clone(),
                policy_regime_diagnosis.exchange_rate_regime.clone(),
            ],
            resulting_judgment: policy_regime_diagnosis.credibility_signals.clone(),
        },
        InferenceStepV1 {
            inference_id: "INF-04".to_string(),
            label: "Sovereign/systemic translation".to_string(),
            assumption_ids,
            inputs_used: vec![
                sovereign_systemic_risk.debt_sustainability_state.clone(),
                sovereign_systemic_risk.rollover_risk.clone(),
            ],
            resulting_judgment: sovereign_systemic_risk.cross_border_spillovers.clone(),
        },
    ]
}

fn build_pipeline_trace(
    problem_contract: &ProblemContractV1,
    data_register: &[DataRegisterEntryV1],
    external_accounts_map: &ExternalAccountsBalanceSheetMapV1,
    policy_regime_diagnosis: &PolicyRegimeDiagnosisV1,
    global_liquidity_funding: &GlobalLiquidityFundingConditionsV1,
    data_vintage: &str,
    driver_decomposition: &[FxDriverAssessmentV1],
    sovereign_systemic_risk: &SovereignSystemicRiskV1,
    scenario_matrix: &[ScenarioCaseV1],
) -> Vec<PipelineStepTraceV1> {
    vec![
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepA,
            ordinal: 1,
            summary: format!(
                "Problem contract compiled with {} missing input groups.",
                problem_contract.missing_inputs.len()
            ),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepB,
            ordinal: 2,
            summary: format!(
                "Data register contains {} entries with DATA_VINTAGE={data_vintage}.",
                data_register.len()
            ),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepC,
            ordinal: 3,
            summary: external_accounts_map.marginal_financer.clone(),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepD,
            ordinal: 4,
            summary: policy_regime_diagnosis.exchange_rate_regime.clone(),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepE,
            ordinal: 5,
            summary: format!(
                "{} via {}.",
                global_liquidity_funding.phase.directive_label(),
                global_liquidity_funding
                    .dominant_transmission_channel
                    .directive_label()
            ),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepF,
            ordinal: 6,
            summary: format!(
                "FX decomposition populated {} buckets.",
                driver_decomposition.len()
            ),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepG,
            ordinal: 7,
            summary: sovereign_systemic_risk.rollover_risk.clone(),
        },
        PipelineStepTraceV1 {
            step: PipelineStepIdV1::StepH,
            ordinal: 8,
            summary: format!(
                "Scenario matrix built with {} scenarios.",
                scenario_matrix.len()
            ),
        },
    ]
}

fn build_claim_evidence(
    executive_brief: &ExecutiveBriefV1,
    scenario_matrix: &[ScenarioCaseV1],
    risk_register: &[RiskRegisterEntryV1],
    knowledge_appendix: &KnowledgeAppendixV1,
    sources: &[KnowledgeSourceV1],
    data_register: &[DataRegisterEntryV1],
    inference_steps: &[InferenceStepV1],
) -> Vec<ClaimEvidenceV1> {
    let source_ids = sources
        .iter()
        .filter(|source| source.evidence_use == KnowledgeEvidenceUseV1::Evidence)
        .map(|source| source.source_id.clone())
        .collect::<Vec<_>>();
    let direct_input_refs = if source_ids.is_empty() {
        data_register
            .iter()
            .map(|entry| {
                format!(
                    "data-register::{}",
                    sha256_hex(
                        format!(
                            "{}|{}|{}",
                            entry.series_name, entry.country_area, entry.source
                        )
                        .as_bytes()
                    )
                )
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let evidence_refs = if source_ids.is_empty() {
        direct_input_refs.clone()
    } else {
        source_ids.clone()
    };
    let inference_ids = inference_steps
        .iter()
        .map(|step| step.inference_id.clone())
        .collect::<Vec<_>>();
    let mut claims = Vec::new();

    for (index, statement) in executive_brief.key_judgments_facts.iter().enumerate() {
        claims.push(ClaimEvidenceV1 {
            claim_id: format!("claim-fact-{}", index + 1),
            output_section: "Executive Brief".to_string(),
            claim_kind: ClaimKindV1::Fact,
            statement: statement.clone(),
            source_ids: evidence_refs.clone(),
            inference_ids: Vec::new(),
        });
    }
    for (index, statement) in executive_brief.key_judgments_inferences.iter().enumerate() {
        claims.push(ClaimEvidenceV1 {
            claim_id: format!("claim-inference-{}", index + 1),
            output_section: "Executive Brief".to_string(),
            claim_kind: ClaimKindV1::Inference,
            statement: statement.clone(),
            source_ids: evidence_refs.clone(),
            inference_ids: inference_ids.clone(),
        });
    }
    for (index, scenario) in scenario_matrix.iter().enumerate() {
        claims.push(ClaimEvidenceV1 {
            claim_id: format!("claim-scenario-{}", index + 1),
            output_section: "Scenario Matrix".to_string(),
            claim_kind: ClaimKindV1::Inference,
            statement: format!(
                "{} | {} | {}",
                scenario.triggers, scenario.fx_outcome, scenario.systemic_risk_outcome
            ),
            source_ids: evidence_refs.clone(),
            inference_ids: inference_ids.clone(),
        });
    }
    for (index, risk) in risk_register.iter().enumerate() {
        claims.push(ClaimEvidenceV1 {
            claim_id: format!("claim-risk-{}", index + 1),
            output_section: "Risk Register".to_string(),
            claim_kind: ClaimKindV1::Inference,
            statement: format!("{} | {}", risk.risk, risk.mechanism),
            source_ids: evidence_refs.clone(),
            inference_ids: inference_ids.clone(),
        });
    }
    claims.push(ClaimEvidenceV1 {
        claim_id: "claim-recommendation-1".to_string(),
        output_section: "Executive Brief".to_string(),
        claim_kind: ClaimKindV1::Recommendation,
        statement: executive_brief.implications.risk_management.clone(),
        source_ids: evidence_refs.clone(),
        inference_ids: inference_ids.clone(),
    });
    claims.push(ClaimEvidenceV1 {
        claim_id: "claim-appendix-source-note".to_string(),
        output_section: "Knowledge Appendix".to_string(),
        claim_kind: ClaimKindV1::Fact,
        statement: knowledge_appendix.source_note.clone(),
        source_ids: evidence_refs,
        inference_ids,
    });
    claims
}

fn build_retrieval_query(request: &MacroFinancialAnalysisRequestV1) -> String {
    let mut parts = BTreeSet::new();
    parts.insert(request.objective.directive_label().to_ascii_lowercase());
    parts.insert(request.horizon.directive_label().to_ascii_lowercase());
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
            .regions
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
    parts.extend(
        request
            .direct_inputs
            .inline_documents
            .iter()
            .map(|document| document.title.to_ascii_lowercase()),
    );
    parts.extend(
        request
            .direct_inputs
            .policy_communications
            .iter()
            .map(|item| item.title.to_ascii_lowercase()),
    );
    parts.extend(
        request
            .direct_inputs
            .geopolitical_timeline
            .iter()
            .map(|item| item.summary.to_ascii_lowercase()),
    );
    parts.into_iter().collect::<Vec<_>>().join(" ")
}

fn required_inputs() -> Vec<String> {
    required_input_catalog()
        .into_iter()
        .map(|(label, _)| label.to_string())
        .collect()
}

fn dependent_variables() -> Vec<String> {
    vec![
        "FX: bilateral/effective/real effective as applicable".to_string(),
        "Financial conditions: domestic and external".to_string(),
        "Capital flows: gross/net by functional category".to_string(),
        "Liquidity risk: funding stress indicators and backstop capacity".to_string(),
    ]
}

fn determine_global_liquidity_phase(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> GlobalLiquidityPhaseV1 {
    let haystack = combined_corpus(sources, inputs);
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
    inputs: &contracts::MacroFinancialDirectInputsV1,
    coverage: &AnalysisCoverageV1,
    global_liquidity_phase: GlobalLiquidityPhaseV1,
) -> Vec<FxDriverAssessmentV1> {
    let corpus = combined_corpus(sources, inputs);
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
    problem_contract: &ProblemContractV1,
    data_vintage: &str,
    driver_decomposition: &[FxDriverAssessmentV1],
    source_governance: &[SourceGovernanceDecisionV1],
    inference_steps: &[InferenceStepV1],
    retrieval_context: &[String],
) -> ExecutiveBriefV1 {
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
        format!(
            "Problem contract identified {} precise missing input requirements.",
            problem_contract.missing_inputs.len()
        ),
        format!(
            "Capsule retrieval returned {} supporting snippets.",
            retrieval_context.len()
        ),
    ];
    let inferences = vec![
        format!(
            "Global liquidity phase is {} based on funding and stress language in the source set.",
            determine_global_liquidity_phase(sources, &request.direct_inputs).directive_label()
        ),
        format!(
            "{} governance decisions were accepted for evidence/context use.",
            source_governance
                .iter()
                .filter(|decision| decision.accepted)
                .count()
        ),
        format!(
            "{} inference steps separate model-based judgments from facts and recommendations.",
            inference_steps.len()
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
        as_of_date: ANALYSIS_AS_OF_DATE.to_string(),
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

fn build_data_register(
    sources: &[KnowledgeSourceV1],
    inputs: &contracts::MacroFinancialDirectInputsV1,
) -> Vec<DataRegisterEntryV1> {
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
        .chain(
            inputs
                .fx_levels_returns
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_FX")),
        )
        .chain(
            inputs
                .rates_yields
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_RATES")),
        )
        .chain(
            inputs
                .inflation_growth_terms_trade_fiscal
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_MACRO")),
        )
        .chain(
            inputs
                .balance_of_payments_iip
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_BOP_IIP")),
        )
        .chain(
            inputs
                .cross_border_banking_credit
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_BANK_CREDIT")),
        )
        .chain(
            inputs
                .portfolio_flow_positions_reserve_composition
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_FLOWS_RESERVES")),
        )
        .chain(
            inputs
                .funding_hedging_indicators
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_FUNDING")),
        )
        .chain(
            inputs
                .market_stress_proxies
                .iter()
                .map(|series| direct_input_series_entry(series, "DIRECT_INPUT_STRESS")),
        )
        .chain(
            inputs
                .policy_communications
                .iter()
                .map(direct_input_policy_entry),
        )
        .chain(
            inputs
                .geopolitical_timeline
                .iter()
                .map(direct_input_geopolitical_entry),
        )
        .chain(
            inputs
                .inline_documents
                .iter()
                .map(direct_input_document_entry),
        )
        .collect()
}

fn direct_input_series_entry(
    series: &contracts::AnalysisSeriesInputV1,
    source: &str,
) -> DataRegisterEntryV1 {
    DataRegisterEntryV1 {
        series_name: series.series_name.clone(),
        country_area: series.country_area.clone(),
        source: format!("{source}:{}", series.source_label),
        frequency: series.frequency.clone().unwrap_or_else(|| {
            "MISSING: provide frequency for this direct-input series.".to_string()
        }),
        last_obs: series
            .last_observation
            .clone()
            .or_else(|| {
                series
                    .observations
                    .last()
                    .map(|observation| format!("{}={}", observation.timestamp, observation.value))
            })
            .unwrap_or_else(|| {
                "MISSING: provide a last observation or at least one time-stamped observation."
                    .to_string()
            }),
        units: series
            .units
            .clone()
            .unwrap_or_else(|| "MISSING: provide units for this direct-input series.".to_string()),
        transform: series
            .transform
            .clone()
            .unwrap_or_else(|| "MISSING: provide transform or state RAW.".to_string()),
        lag: "USER_PROVIDED".to_string(),
        quality_flag: String::new(),
        notes: "Direct input supplied with analysis request.".to_string(),
    }
}

fn direct_input_policy_entry(item: &contracts::PolicyCommunicationInputV1) -> DataRegisterEntryV1 {
    DataRegisterEntryV1 {
        series_name: item.title.clone(),
        country_area: item.issuer.clone(),
        source: "DIRECT_INPUT_POLICY_COMMUNICATION".to_string(),
        frequency: "EVENT".to_string(),
        last_obs: item.issued_at.clone(),
        units: "TEXT".to_string(),
        transform: "SUMMARY".to_string(),
        lag: "USER_PROVIDED".to_string(),
        quality_flag: contracts::QualityFlagV1::ProxyUsed
            .directive_label()
            .to_string(),
        notes: item.summary.clone(),
    }
}

fn direct_input_geopolitical_entry(
    item: &contracts::GeopoliticalEventInputV1,
) -> DataRegisterEntryV1 {
    DataRegisterEntryV1 {
        series_name: item.event_id.clone(),
        country_area: if item.jurisdictions.is_empty() {
            "MISSING: provide jurisdictions linked to the event.".to_string()
        } else {
            item.jurisdictions.join(", ")
        },
        source: "DIRECT_INPUT_GEOPOLITICAL_EVENT".to_string(),
        frequency: "EVENT".to_string(),
        last_obs: item.event_date.clone(),
        units: "TEXT".to_string(),
        transform: "SUMMARY".to_string(),
        lag: "USER_PROVIDED".to_string(),
        quality_flag: contracts::QualityFlagV1::ProxyUsed
            .directive_label()
            .to_string(),
        notes: item.summary.clone(),
    }
}

fn direct_input_document_entry(item: &contracts::AnalysisTextInputV1) -> DataRegisterEntryV1 {
    DataRegisterEntryV1 {
        series_name: item.title.clone(),
        country_area: item.country_area.clone().unwrap_or_else(|| {
            "MISSING: provide country/area context for this document.".to_string()
        }),
        source: format!("DIRECT_INPUT_DOCUMENT:{}", item.source_label),
        frequency: "DOCUMENT".to_string(),
        last_obs: "MISSING: document inputs require an external date if timing matters."
            .to_string(),
        units: "TEXT".to_string(),
        transform: "RAW_TEXT".to_string(),
        lag: "USER_PROVIDED".to_string(),
        quality_flag: contracts::QualityFlagV1::ProxyUsed
            .directive_label()
            .to_string(),
        notes: format!("Input id={}", item.input_id),
    }
}

fn build_mechanism_map(
    external_accounts_map: &ExternalAccountsBalanceSheetMapV1,
    global_liquidity_funding: &GlobalLiquidityFundingConditionsV1,
) -> MechanismMapV1 {
    MechanismMapV1 {
        current_account_narrative: external_accounts_map.current_account_pressures.clone(),
        financial_account_funding_mix: format!(
            "{} Marginal financer: {}. Flow-reversal vulnerability: {}.",
            external_accounts_map.financial_account_decomposition,
            external_accounts_map.marginal_financer,
            external_accounts_map.flow_reversal_vulnerability
        ),
        reserves_and_backstops:
            global_liquidity_funding.backstop_availability.clone(),
        fx_swap_basis_state: format!(
            "Dominant transmission channel: {}.",
            global_liquidity_funding
                .dominant_transmission_channel
                .directive_label()
        ),
        dollar_funding_stress_state: global_liquidity_funding.dollar_funding_stress_state.clone(),
        risk_sentiment_linkage:
            "Risk sentiment is translated through the global financial cycle, funding premia, and portfolio adjustment."
                .to_string(),
        spillover_channels:
            "Spillovers are assessed through cross-border banking, bond allocation, derivatives funding, and sovereign financing links."
                .to_string(),
    }
}

fn build_scenario_matrix(
    request: &MacroFinancialAnalysisRequestV1,
    driver_decomposition: &[FxDriverAssessmentV1],
    external_accounts_map: &ExternalAccountsBalanceSheetMapV1,
    policy_regime_diagnosis: &PolicyRegimeDiagnosisV1,
    global_liquidity_funding: &GlobalLiquidityFundingConditionsV1,
    sovereign_systemic_risk: &SovereignSystemicRiskV1,
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
            driver_decomposition,
            external_accounts_map,
            policy_regime_diagnosis,
            global_liquidity_funding,
            sovereign_systemic_risk,
        )
    })
    .collect()
}

fn build_scenario_case(
    scenario: ScenarioKindV1,
    request: &MacroFinancialAnalysisRequestV1,
    driver_decomposition: &[FxDriverAssessmentV1],
    external_accounts_map: &ExternalAccountsBalanceSheetMapV1,
    policy_regime_diagnosis: &PolicyRegimeDiagnosisV1,
    global_liquidity_funding: &GlobalLiquidityFundingConditionsV1,
    sovereign_systemic_risk: &SovereignSystemicRiskV1,
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
    let watchlist: Vec<WatchlistIndicatorV1> = driver_decomposition
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
            "{} for {} over {}. Watchlist: {}",
            scenario_bias,
            request.coverage.summary(),
            request.horizon.directive_label(),
            watchlist
                .iter()
                .map(|item| format!("{} [{}]", item.indicator, item.threshold))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        transmission_path: format!(
            "External accounts ({}) -> funding markets ({}) -> FX -> domestic conditions shaped by {}.",
            external_accounts_map.marginal_financer,
            global_liquidity_funding
                .dominant_transmission_channel
                .directive_label(),
            policy_regime_diagnosis.exchange_rate_regime
        ),
        fx_outcome: fx_outcome.to_string(),
        capital_flows_outcome: external_accounts_map.flow_reversal_vulnerability.clone(),
        liquidity_funding_outcome: format!(
            "Liquidity/funding outcome is benchmarked against a {} baseline.",
            global_liquidity_funding.phase.directive_label()
        ),
        systemic_risk_outcome: sovereign_systemic_risk.cross_border_spillovers.clone(),
        policy_response_space:
            format!(
                "Policy space depends on reserve adequacy, communication credibility, and macroprudential flexibility. {}",
                policy_regime_diagnosis.credibility_signals
            ),
        strategy_implications:
            "Generic strategy bias favors explicit hedging, tighter risk limits, and staged exposure changes."
                .to_string(),
        watchlist,
    }
}

fn build_risk_register(
    driver_decomposition: &[FxDriverAssessmentV1],
    sovereign_systemic_risk: &SovereignSystemicRiskV1,
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
            impact_channels: format!(
                "FX, bond yields, domestic credit availability, and {}.",
                sovereign_systemic_risk.sovereign_bank_nonbank_nexus
            ),
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

fn build_knowledge_appendix(
    constraints: &SourceConstraintsV1,
    assumptions: &[AnalysisAssumptionV1],
) -> KnowledgeAppendixV1 {
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
        source_note: if constraints.allowed_sources.is_empty()
            && constraints.forbidden_sources.is_empty()
        {
            "Primary-source standards prefer IMF, BIS, FSB, World Bank, central banks, finance ministries/statistical agencies, and approved flagship or peer-reviewed institutional research; secondary sources remain context only."
                .to_string()
        } else {
            format!(
                "Primary-source standards prefer IMF, BIS, FSB, World Bank, central banks, finance ministries/statistical agencies, and approved flagship or peer-reviewed institutional research. Allowed={}; Forbidden={}.",
                if constraints.allowed_sources.is_empty() {
                    "UNSPECIFIED".to_string()
                } else {
                    constraints.allowed_sources.join(", ")
                },
                if constraints.forbidden_sources.is_empty() {
                    "UNSPECIFIED".to_string()
                } else {
                    constraints.forbidden_sources.join(", ")
                }
            )
        },
        assumptions_log: assumptions
            .iter()
            .map(|assumption| {
                format!(
                    "{}. {} [{}]",
                    assumption.assumption_id,
                    assumption.text,
                    if assumption.stable { "stable" } else { "unstable" }
                )
            })
            .collect(),
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
        KnowledgeSourceKindV1::Fsb => "FSB",
        KnowledgeSourceKindV1::WorldBank => "WORLD_BANK",
        KnowledgeSourceKindV1::Fred => "FRED",
        KnowledgeSourceKindV1::OfficialDocument => "OFFICIAL_DOCUMENT",
        KnowledgeSourceKindV1::ResearchPaper => "RESEARCH_PAPER",
        KnowledgeSourceKindV1::SecondaryContext => "SECONDARY",
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
    mod contract_parity {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/contract_parity.rs"
        ));
    }

    use std::net::SocketAddr;

    use approval_service::ApprovalService;
    use chrono::{Duration, TimeZone};
    use contract_parity::assert_service_boundary_matches_catalog;
    use contracts::{AnalysisHorizonV1, AnalysisObjectiveV1};
    use evidence_service::EvidenceService;
    use governed_storage::connect_in_memory;
    use memory_provider::MemvidMemoryProvider;
    use policy_service::PolicyService;
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

    fn sample_direct_inputs() -> contracts::MacroFinancialDirectInputsV1 {
        contracts::MacroFinancialDirectInputsV1 {
            fx_levels_returns: vec![contracts::AnalysisSeriesInputV1 {
                series_name: "USDJPY".to_string(),
                country_area: "Japan".to_string(),
                source_label: "USER_FX".to_string(),
                frequency: Some("Daily".to_string()),
                last_observation: Some("2026-03-09=149.20".to_string()),
                units: Some("spot".to_string()),
                transform: Some("level".to_string()),
                observations: vec![contracts::AnalysisObservationV1 {
                    timestamp: "2026-03-09".to_string(),
                    value: "149.20".to_string(),
                }],
            }],
            rates_yields: vec![contracts::AnalysisSeriesInputV1 {
                series_name: "JPY_2Y".to_string(),
                country_area: "Japan".to_string(),
                source_label: "USER_RATES".to_string(),
                frequency: Some("Daily".to_string()),
                last_observation: Some("2026-03-09=0.85".to_string()),
                units: Some("%".to_string()),
                transform: Some("level".to_string()),
                observations: vec![contracts::AnalysisObservationV1 {
                    timestamp: "2026-03-09".to_string(),
                    value: "0.85".to_string(),
                }],
            }],
            inflation_growth_terms_trade_fiscal: vec![contracts::AnalysisSeriesInputV1 {
                series_name: "CPI".to_string(),
                country_area: "Japan".to_string(),
                source_label: "USER_MACRO".to_string(),
                frequency: Some("Monthly".to_string()),
                last_observation: Some("2026-02=2.4".to_string()),
                units: Some("%".to_string()),
                transform: Some("yoy".to_string()),
                observations: vec![contracts::AnalysisObservationV1 {
                    timestamp: "2026-02".to_string(),
                    value: "2.4".to_string(),
                }],
            }],
            funding_hedging_indicators: vec![contracts::AnalysisSeriesInputV1 {
                series_name: "USDJPY_BASIS".to_string(),
                country_area: "Japan".to_string(),
                source_label: "USER_FUNDING".to_string(),
                frequency: Some("Daily".to_string()),
                last_observation: Some("2026-03-09=-18".to_string()),
                units: Some("bp".to_string()),
                transform: Some("level".to_string()),
                observations: vec![contracts::AnalysisObservationV1 {
                    timestamp: "2026-03-09".to_string(),
                    value: "-18".to_string(),
                }],
            }],
            market_stress_proxies: vec![contracts::AnalysisSeriesInputV1 {
                series_name: "CREDIT_SPREAD".to_string(),
                country_area: "Global".to_string(),
                source_label: "USER_STRESS".to_string(),
                frequency: Some("Daily".to_string()),
                last_observation: Some("2026-03-09=145".to_string()),
                units: Some("bp".to_string()),
                transform: Some("level".to_string()),
                observations: vec![contracts::AnalysisObservationV1 {
                    timestamp: "2026-03-09".to_string(),
                    value: "145".to_string(),
                }],
            }],
            policy_communications: vec![contracts::PolicyCommunicationInputV1 {
                communication_id: "pc-1".to_string(),
                issuer: "Bank of Japan".to_string(),
                title: "Policy statement".to_string(),
                issued_at: "2026-03-08".to_string(),
                summary: "Policy statement highlights inflation and reserve flexibility."
                    .to_string(),
            }],
            geopolitical_timeline: vec![contracts::GeopoliticalEventInputV1 {
                event_id: "geo-1".to_string(),
                event_date: "2026-03-05".to_string(),
                summary: "Tariff escalation raises external funding stress.".to_string(),
                jurisdictions: vec!["Japan".to_string(), "United States".to_string()],
            }],
            inline_documents: vec![contracts::AnalysisTextInputV1 {
                input_id: "doc-1".to_string(),
                title: "Desk note".to_string(),
                country_area: Some("Japan".to_string()),
                source_label: "USER_NOTE".to_string(),
                text: "Cross-border funding conditions are tighter and basis stress is rising."
                    .to_string(),
            }],
            ..contracts::MacroFinancialDirectInputsV1::default()
        }
    }

    fn assert_strict_output_headings(rendered_output: &str) {
        let headings = rendered_output
            .lines()
            .filter(|line| line.starts_with("[Output "))
            .collect::<Vec<_>>();
        assert_eq!(
            headings,
            vec![
                "[Output 1: Executive Brief]",
                "[Output 2: Data Register]",
                "[Output 3: Mechanism Map]",
                "[Output 4: Scenario Matrix]",
                "[Output 5: Risk Register]",
                "[Output 6: Knowledge Appendix]",
            ]
        );
        assert!(rendered_output.contains("AS_OF_DATE: 2026-03-09"));
        assert!(rendered_output.contains("AS_OF_TIMEZONE: America/Los_Angeles"));
        assert!(rendered_output.contains("Scenario: BASE"));
        assert!(rendered_output.contains("Scenario: UPSIDE"));
        assert!(rendered_output.contains("Scenario: DOWNSIDE"));
        assert!(rendered_output.contains("Scenario: TAIL_LIQUIDITY_EVENT"));
        assert!(!rendered_output.contains("WATCHLIST:"));
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
                    constraints: SourceConstraintsV1::default(),
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
                    constraints: SourceConstraintsV1::default(),
                },
            )
            .await
            .expect("publish");
        assert_eq!(capsule.source_count, 4);

        let analysis = Box::pin(service.generate_analysis(
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
                direct_inputs: contracts::MacroFinancialDirectInputsV1::default(),
                classification: Classification::Internal,
                constraints: SourceConstraintsV1::default(),
            },
        ))
        .await
        .expect("analysis");

        assert_eq!(analysis.scenario_matrix.len(), 4);
        assert!(
            analysis
                .rendered_output
                .contains("[Output 1: Executive Brief]")
        );
        assert!(analysis.rendered_output.contains("TAIL_LIQUIDITY_EVENT"));
        assert_strict_output_headings(&analysis.rendered_output);
        assert!(
            !analysis
                .policy_regime_diagnosis
                .monetary_policy_regime
                .is_empty()
        );
        assert!(!analysis.policy_regime_diagnosis.frictions.is_empty());
        assert!(
            analysis
                .policy_regime_diagnosis
                .frictions
                .iter()
                .all(|friction| !friction.observable_indicators.is_empty())
        );
        assert!(
            !analysis
                .sovereign_systemic_risk
                .debt_sustainability_state
                .is_empty()
        );
        assert!(
            !analysis
                .sovereign_systemic_risk
                .cross_border_spillovers
                .is_empty()
        );
        assert!(
            analysis
                .source_governance
                .iter()
                .all(|decision| decision.accepted)
        );
        assert_eq!(
            analysis
                .inference_steps
                .iter()
                .map(|step| step.inference_id.as_str())
                .collect::<Vec<_>>(),
            vec!["INF-01", "INF-02", "INF-03", "INF-04"]
        );
        assert!(
            analysis
                .claim_evidence
                .iter()
                .any(|claim| claim.claim_kind == ClaimKindV1::Fact)
        );
        assert!(
            analysis
                .claim_evidence
                .iter()
                .any(|claim| claim.claim_kind == ClaimKindV1::Inference)
        );
        assert!(
            analysis
                .claim_evidence
                .iter()
                .any(|claim| claim.claim_kind == ClaimKindV1::Recommendation)
        );
        assert_eq!(
            analysis
                .pipeline_trace
                .iter()
                .map(|trace| trace.step)
                .collect::<Vec<_>>(),
            vec![
                PipelineStepIdV1::StepA,
                PipelineStepIdV1::StepB,
                PipelineStepIdV1::StepC,
                PipelineStepIdV1::StepD,
                PipelineStepIdV1::StepE,
                PipelineStepIdV1::StepF,
                PipelineStepIdV1::StepG,
                PipelineStepIdV1::StepH,
            ]
        );
        assert!(
            analysis
                .claim_evidence
                .iter()
                .all(|claim| !claim.source_ids.is_empty() || !claim.inference_ids.is_empty())
        );
        assert!(
            analysis
                .scenario_matrix
                .iter()
                .all(|scenario| scenario.triggers.contains("Watchlist:"))
        );
        assert!(
            service
                .latest_publication_status(&repositories)
                .await
                .expect("status")
                .is_some()
        );
        assert!(
            service
                .load_analysis(&repositories, "analysis-1")
                .await
                .expect("load")
                .is_some()
        );
    }

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source =
            include_str!("../../../enterprise/domains/data_knowledge/service_boundaries.toml");
        let boundary = service_boundary();

        assert_service_boundary_matches_catalog(&boundary, DOMAIN_NAME, source);
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
            action_id: "action-1".into(),
            actor_ref: identity::ActorRef("agent:finance".to_string()),
            objective: "Compile macro-financial sources".to_string(),
            requested_workflow: "knowledge_publication".into(),
            impact_tier: contracts::ImpactTier::Tier0,
            classification: Classification::Internal,
            required_approver_roles: Vec::new(),
            policy_refs: vec!["policy.data_knowledge".to_string()],
        };
        let mut approved = None;
        let guarded_request = enforcement::GuardedMutationRequest {
            action_id: action.action_id.clone(),
            workflow_name: "knowledge_publication".into(),
            target_service: SERVICE_NAME.into(),
            target_aggregate: "knowledge_source".into(),
            actor_ref: action.actor_ref.clone(),
            impact_tier: action.impact_tier,
            classification: action.classification,
            policy_refs: action.policy_refs.clone(),
            required_approver_roles: action.required_approver_roles.clone(),
            environment: "prod".into(),
            cross_domain: false,
        };
        engine
            .execute_mutation(guarded_request, |context| {
                approved = Some(context.clone());
                Ok(())
            })
            .await
            .expect("authorize");
        let context = approved.expect("context");
        let ingested = service
            .ingest_sources(
                &context,
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-ctx".to_string(),
                    classification: Classification::Internal,
                    constraints: SourceConstraintsV1::default(),
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

    #[tokio::test]
    async fn direct_input_only_analysis_is_supported_and_traceable() {
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
        );

        let analysis = Box::pin(service.generate_analysis(
            &repositories,
            MacroFinancialAnalysisRequestV1 {
                analysis_id: "analysis-direct".to_string(),
                objective: AnalysisObjectiveV1::RiskMgmt,
                horizon: AnalysisHorizonV1::OneToThreeMonths,
                coverage: AnalysisCoverageV1 {
                    countries: vec!["Japan".to_string()],
                    regions: vec!["Asia".to_string()],
                    currencies: vec!["JPY".to_string()],
                    fx_pairs: vec!["USD/JPY".to_string()],
                    asset_classes: vec!["rates".to_string()],
                },
                data_vintage: None,
                source_ids: Vec::new(),
                capsule_id: None,
                direct_inputs: sample_direct_inputs(),
                classification: Classification::Internal,
                constraints: SourceConstraintsV1::default(),
            },
        ))
        .await
        .expect("analysis");

        assert_eq!(analysis.executive_brief.as_of_date, ANALYSIS_AS_OF_DATE);
        assert_eq!(analysis.executive_brief.as_of_timezone, ANALYSIS_TIMEZONE);
        assert_eq!(analysis.executive_brief.data_vintage, "UNKNOWN");
        assert!(
            analysis
                .problem_contract
                .missing_inputs
                .contains(&"MISSING: provide balance of payments and IIP components.".to_string())
        );
        assert!(analysis.problem_contract.missing_inputs.contains(
            &"MISSING: provide cross-border banking/credit measures or equivalent official series."
                .to_string()
        ));
        assert!(
            analysis
                .data_register
                .iter()
                .any(|entry| entry.source.starts_with("DIRECT_INPUT"))
        );
        assert!(
            analysis
                .claim_evidence
                .iter()
                .all(|claim| !claim.source_ids.is_empty() || !claim.inference_ids.is_empty())
        );
        assert_eq!(analysis.pipeline_trace.len(), 8);
        assert!(analysis.source_governance.is_empty());
        assert_strict_output_headings(&analysis.rendered_output);
        assert_eq!(
            analysis
                .inference_steps
                .iter()
                .map(|step| step.inference_id.as_str())
                .collect::<Vec<_>>(),
            vec!["INF-01", "INF-02", "INF-03", "INF-04"]
        );
        assert!(
            analysis
                .claim_evidence
                .iter()
                .any(|claim| claim.claim_kind == ClaimKindV1::Recommendation)
        );
    }

    #[tokio::test]
    async fn governance_rules_block_forbidden_publication_and_secondary_only_analysis() {
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

        let forbidden_ingest = service
            .ingest_sources_unchecked(
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-forbidden".to_string(),
                    classification: Classification::Internal,
                    constraints: SourceConstraintsV1 {
                        allowed_sources: Vec::new(),
                        forbidden_sources: vec!["source-fred".to_string()],
                        required_output_format: None,
                    },
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
            .expect_err("forbidden source should fail");
        assert!(matches!(
            forbidden_ingest,
            InstitutionalError::PolicyDenied { .. }
        ));

        let allowlisted_primary = service
            .ingest_sources_unchecked(
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-allow".to_string(),
                    classification: Classification::Internal,
                    constraints: SourceConstraintsV1 {
                        allowed_sources: vec!["imf".to_string()],
                        forbidden_sources: Vec::new(),
                        required_output_format: None,
                    },
                    sources: vec![KnowledgeSourceFetchSpecV1 {
                        source_id: "source-imf-allow".to_string(),
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
                    }],
                },
            )
            .await
            .expect("allowlisted primary ingest");
        assert_eq!(allowlisted_primary.len(), 1);

        let secondary = service
            .ingest_sources_unchecked(
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-secondary".to_string(),
                    classification: Classification::Internal,
                    constraints: SourceConstraintsV1::default(),
                    sources: vec![KnowledgeSourceFetchSpecV1 {
                        source_id: "source-secondary".to_string(),
                        kind: KnowledgeSourceKindV1::SecondaryContext,
                        title: "Desk context note".to_string(),
                        country_area: "Japan".to_string(),
                        url: format!("http://{address}/bulletin"),
                        series_name: None,
                        expected_format: KnowledgeDocumentFormatV1::Html,
                        release_lag: None,
                        units: None,
                        transform: None,
                        notes: vec!["Secondary context".to_string()],
                    }],
                },
            )
            .await
            .expect("secondary ingest");
        assert_eq!(
            secondary[0].provenance_tier,
            KnowledgeSourceProvenanceV1::Secondary
        );
        assert_eq!(
            secondary[0].evidence_use,
            KnowledgeEvidenceUseV1::ContextOnly
        );

        let secondary_only_error = Box::pin(service.generate_analysis(
            &repositories,
            MacroFinancialAnalysisRequestV1 {
                analysis_id: "analysis-secondary".to_string(),
                objective: AnalysisObjectiveV1::PolicyEval,
                horizon: AnalysisHorizonV1::Nowcast,
                coverage: AnalysisCoverageV1 {
                    countries: vec!["Japan".to_string()],
                    regions: Vec::new(),
                    currencies: vec!["JPY".to_string()],
                    fx_pairs: vec!["USD/JPY".to_string()],
                    asset_classes: vec!["rates".to_string()],
                },
                data_vintage: Some("2026-03-09".to_string()),
                source_ids: vec!["source-secondary".to_string()],
                capsule_id: None,
                direct_inputs: contracts::MacroFinancialDirectInputsV1::default(),
                classification: Classification::Internal,
                constraints: SourceConstraintsV1::default(),
            },
        ))
        .await
        .expect_err("secondary-only analysis should fail");
        assert!(matches!(
            secondary_only_error,
            InstitutionalError::NotFound { .. }
        ));

        let primary = service
            .ingest_sources_unchecked(
                &repositories,
                KnowledgeSourceIngestRequestV1 {
                    ingestion_id: "ingest-primary".to_string(),
                    classification: Classification::Internal,
                    constraints: SourceConstraintsV1::default(),
                    sources: vec![KnowledgeSourceFetchSpecV1 {
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
                    }],
                },
            )
            .await
            .expect("primary ingest");

        let publish_error = service
            .publish_capsule_unchecked(
                &repositories,
                KnowledgePublicationRequestV1 {
                    publication_id: "publication-blocked".to_string(),
                    capsule_id: "capsule-blocked".to_string(),
                    title: "Blocked capsule".to_string(),
                    source_ids: vec![primary[0].source_id.clone()],
                    classification: Classification::Internal,
                    retention_class: "institutional_record".to_string(),
                    constraints: SourceConstraintsV1 {
                        allowed_sources: Vec::new(),
                        forbidden_sources: vec!["source-imf".to_string()],
                        required_output_format: None,
                    },
                },
            )
            .await
            .expect_err("publication should enforce forbidden sources");
        assert!(matches!(
            publish_error,
            InstitutionalError::PolicyDenied { .. }
        ));
    }

    #[test]
    fn source_policy_rejects_social_media_hosts() {
        let error = validate_source_url(
            &KnowledgeSourceFetchSpecV1 {
                source_id: "source-social".to_string(),
                kind: KnowledgeSourceKindV1::SecondaryContext,
                title: "Social thread".to_string(),
                country_area: "Global".to_string(),
                url: "https://x.com/example/status/1".to_string(),
                series_name: None,
                expected_format: KnowledgeDocumentFormatV1::Html,
                release_lag: None,
                units: None,
                transform: None,
                notes: Vec::new(),
            },
            &default_official_document_hosts(),
        )
        .expect_err("social hosts must be rejected");
        assert!(matches!(error, InstitutionalError::PolicyDenied { .. }));
    }
}

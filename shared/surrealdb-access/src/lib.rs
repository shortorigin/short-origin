use contracts::{
    EvidenceManifestV1, KnowledgeCapsuleV1, KnowledgeEdgeV1, KnowledgePublicationStatusV1,
    KnowledgeSourceV1, MacroFinancialAnalysisV1, TreasuryDisbursementRecordedV1,
};
use error_model::{InstitutionalError, InstitutionalResult};
use events::RecordedEventV1;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::{Connection, Surreal};
use surrealdb_model::{
    EventRecordV1, EvidenceManifestRecordV1, KnowledgeAnalysisRecordV1, KnowledgeCapsuleRecordV1,
    KnowledgeEdgeRecordV1, KnowledgeSourceRecordV1, TreasuryDisbursementRecordV1,
    WorkflowExecutionRecordV1,
};

pub const DEFAULT_NAMESPACE: &str = "short_origin";
pub const DEFAULT_DATABASE: &str = "institutional";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableCatalog {
    pub workflow_execution: &'static str,
    pub evidence_manifest: &'static str,
    pub recorded_event: &'static str,
    pub treasury_disbursement: &'static str,
    pub knowledge_source: &'static str,
    pub knowledge_capsule: &'static str,
    pub knowledge_analysis: &'static str,
    pub knowledge_edge: &'static str,
}

pub const TABLES: TableCatalog = TableCatalog {
    workflow_execution: "workflow_execution",
    evidence_manifest: "evidence_manifest",
    recorded_event: "recorded_event",
    treasury_disbursement: "treasury_disbursement",
    knowledge_source: "knowledge_source",
    knowledge_capsule: "knowledge_capsule",
    knowledge_analysis: "knowledge_analysis",
    knowledge_edge: "knowledge_edge",
};

pub struct SurrealRepositoryContext<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> Clone for SurrealRepositoryContext<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl<C> SurrealRepositoryContext<C>
where
    C: Connection,
{
    #[must_use]
    pub fn new(db: Surreal<C>) -> Self {
        Self { db }
    }

    pub async fn use_namespace(&self, namespace: &str, database: &str) -> InstitutionalResult<()> {
        self.db
            .use_ns(namespace)
            .use_db(database)
            .await
            .map_err(surreal_error)?;
        Ok(())
    }

    #[must_use]
    pub fn workflow_executions(&self) -> WorkflowExecutionRepository<C> {
        WorkflowExecutionRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn evidence_manifests(&self) -> EvidenceManifestRepository<C> {
        EvidenceManifestRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn recorded_events(&self) -> RecordedEventRepository<C> {
        RecordedEventRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn treasury_disbursements(&self) -> TreasuryDisbursementRepository<C> {
        TreasuryDisbursementRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_sources(&self) -> KnowledgeSourceRepository<C> {
        KnowledgeSourceRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_capsules(&self) -> KnowledgeCapsuleRepository<C> {
        KnowledgeCapsuleRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_analyses(&self) -> KnowledgeAnalysisRepository<C> {
        KnowledgeAnalysisRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_edges(&self) -> KnowledgeEdgeRepository<C> {
        KnowledgeEdgeRepository {
            db: self.db.clone(),
        }
    }
}

pub async fn connect_in_memory() -> InstitutionalResult<SurrealRepositoryContext<Db>> {
    let db = Surreal::new::<Mem>(()).await.map_err(surreal_error)?;
    let context = SurrealRepositoryContext::new(db);
    context
        .use_namespace(DEFAULT_NAMESPACE, DEFAULT_DATABASE)
        .await?;
    Ok(context)
}

pub struct WorkflowExecutionRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> WorkflowExecutionRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        record: WorkflowExecutionRecordV1,
    ) -> InstitutionalResult<WorkflowExecutionRecordV1> {
        create_record(
            &self.db,
            TABLES.workflow_execution,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<WorkflowExecutionRecordV1>> {
        select_record(&self.db, TABLES.workflow_execution, id).await
    }
}

pub struct EvidenceManifestRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> EvidenceManifestRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        id: impl Into<String>,
        evidence: EvidenceManifestV1,
    ) -> InstitutionalResult<EvidenceManifestRecordV1> {
        let record = EvidenceManifestRecordV1 {
            id: id.into(),
            evidence,
        };
        create_record(
            &self.db,
            TABLES.evidence_manifest,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<EvidenceManifestRecordV1>> {
        select_record(&self.db, TABLES.evidence_manifest, id).await
    }
}

pub struct RecordedEventRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> RecordedEventRepository<C>
where
    C: Connection,
{
    pub async fn append(
        &self,
        id: impl Into<String>,
        event: RecordedEventV1,
    ) -> InstitutionalResult<EventRecordV1> {
        let record = EventRecordV1 {
            id: id.into(),
            event,
        };
        create_record(&self.db, TABLES.recorded_event, record.id.clone(), record).await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<EventRecordV1>> {
        select_record(&self.db, TABLES.recorded_event, id).await
    }
}

pub struct TreasuryDisbursementRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> TreasuryDisbursementRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        id: impl Into<String>,
        disbursement: TreasuryDisbursementRecordedV1,
    ) -> InstitutionalResult<TreasuryDisbursementRecordV1> {
        let record = TreasuryDisbursementRecordV1 {
            id: id.into(),
            disbursement,
        };
        create_record(
            &self.db,
            TABLES.treasury_disbursement,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(
        &self,
        id: &str,
    ) -> InstitutionalResult<Option<TreasuryDisbursementRecordV1>> {
        select_record(&self.db, TABLES.treasury_disbursement, id).await
    }
}

pub struct KnowledgeSourceRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeSourceRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        source: KnowledgeSourceV1,
    ) -> InstitutionalResult<KnowledgeSourceRecordV1> {
        let record = KnowledgeSourceRecordV1 {
            id: source.source_id.clone(),
            source,
        };
        create_record(&self.db, TABLES.knowledge_source, record.id.clone(), record).await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeSourceRecordV1>> {
        select_record(&self.db, TABLES.knowledge_source, id).await
    }

    pub async fn load_many(
        &self,
        ids: &[String],
    ) -> InstitutionalResult<Vec<KnowledgeSourceRecordV1>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(record) = self.load(id).await? {
                out.push(record);
            }
        }
        Ok(out)
    }
}

pub struct KnowledgeCapsuleRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeCapsuleRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        capsule: KnowledgeCapsuleV1,
    ) -> InstitutionalResult<KnowledgeCapsuleRecordV1> {
        let record = KnowledgeCapsuleRecordV1 {
            id: capsule.capsule_id.clone(),
            capsule,
        };
        create_record(
            &self.db,
            TABLES.knowledge_capsule,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeCapsuleRecordV1>> {
        select_record(&self.db, TABLES.knowledge_capsule, id).await
    }

    pub async fn latest_status(&self) -> InstitutionalResult<Option<KnowledgePublicationStatusV1>> {
        let mut response = self
            .db
            .query(
                "SELECT *, type::string(id) AS id FROM knowledge_capsule ORDER BY capsule.published_at DESC LIMIT 1;",
            )
            .await
            .map_err(surreal_error)?;
        let latest: Option<KnowledgeCapsuleRecordV1> = response.take(0).map_err(surreal_error)?;
        Ok(latest
            .as_ref()
            .map(|record| KnowledgePublicationStatusV1::from_capsule(&record.capsule)))
    }
}

pub struct KnowledgeAnalysisRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeAnalysisRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        analysis: MacroFinancialAnalysisV1,
    ) -> InstitutionalResult<KnowledgeAnalysisRecordV1> {
        let record = KnowledgeAnalysisRecordV1 {
            id: analysis.analysis_id.clone(),
            analysis,
        };
        create_record(
            &self.db,
            TABLES.knowledge_analysis,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeAnalysisRecordV1>> {
        select_record(&self.db, TABLES.knowledge_analysis, id).await
    }
}

pub struct KnowledgeEdgeRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeEdgeRepository<C>
where
    C: Connection,
{
    pub async fn store(&self, edge: KnowledgeEdgeV1) -> InstitutionalResult<KnowledgeEdgeRecordV1> {
        let record = KnowledgeEdgeRecordV1 {
            id: edge.edge_id.clone(),
            edge,
        };
        create_record(&self.db, TABLES.knowledge_edge, record.id.clone(), record).await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeEdgeRecordV1>> {
        select_record(&self.db, TABLES.knowledge_edge, id).await
    }

    pub async fn load_many(
        &self,
        ids: &[String],
    ) -> InstitutionalResult<Vec<KnowledgeEdgeRecordV1>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(record) = self.load(id).await? {
                out.push(record);
            }
        }
        Ok(out)
    }
}

async fn create_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: String,
    record: T,
) -> InstitutionalResult<T>
where
    C: Connection,
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    let content = serde_json::to_value(record.clone()).map_err(|error| {
        InstitutionalError::external("surrealdb", Some("create".to_string()), error.to_string())
    })?;
    let content = match content {
        serde_json::Value::Object(mut map) => {
            map.remove("id");
            serde_json::Value::Object(map)
        }
        other => other,
    };
    db.query("UPSERT type::thing($table, $id) CONTENT $content;")
        .bind(("table", table.to_string()))
        .bind(("id", id))
        .bind(("content", content))
        .await
        .map_err(surreal_error)?;
    Ok(record)
}

async fn select_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: &str,
) -> InstitutionalResult<Option<T>>
where
    C: Connection,
    T: serde::de::DeserializeOwned,
{
    let mut response = db
        .query("SELECT *, type::string(id) AS id FROM ONLY type::thing($table, $id);")
        .bind(("table", table.to_string()))
        .bind(("id", id.to_string()))
        .await
        .map_err(surreal_error)?;
    response.take(0).map_err(surreal_error)
}

fn surreal_error(error: surrealdb::Error) -> InstitutionalError {
    InstitutionalError::external("surrealdb", None, error.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use contracts::{
        AnalysisAssumptionV1, AnalysisCoverageV1, AnalysisHorizonV1, AnalysisImplicationsV1,
        AnalysisObjectiveV1, ClaimEvidenceV1, ClaimKindV1, Classification, ConfidenceV1,
        DataRegisterEntryV1, DirectionalBiasV1, DriverBucketV1, EvidenceManifestV1,
        ExecutiveBriefV1, ExternalAccountsBalanceSheetMapV1, FxDriverAssessmentV1,
        GlobalLiquidityFundingConditionsV1, GlobalLiquidityPhaseV1, InferenceStepV1,
        KnowledgeAppendixV1, KnowledgeCapsuleV1, KnowledgeDocumentFormatV1, KnowledgeEdgeV1,
        KnowledgeEvidenceUseV1, KnowledgeRelationshipV1, KnowledgeSourceKindV1,
        KnowledgeSourceProvenanceV1, KnowledgeSourceV1, MacroFinancialAnalysisV1, MechanismMapV1,
        PipelineStepIdV1, PipelineStepTraceV1, PolicyFrictionObservationV1,
        PolicyRegimeDiagnosisV1, ProbabilityV1, ProblemContractV1, RankedRiskV1,
        RiskRegisterEntryV1, ScenarioCaseV1, ScenarioKindV1, SignalMagnitudeV1,
        SignalSummaryEntryV1, SourceGovernanceDecisionV1, SovereignSystemicRiskV1,
        TransmissionChannelV1, TreasuryDisbursementRecordedV1, WatchlistIndicatorV1,
    };
    use events::EventEnvelopeV1;
    use identity::ActorRef;

    use super::*;

    #[tokio::test]
    async fn repositories_round_trip_core_records() {
        let context = connect_in_memory().await.expect("memory db");

        let workflow = context
            .workflow_executions()
            .store(WorkflowExecutionRecordV1 {
                id: "wf-1".to_string(),
                workflow_name: "treasury_disbursement".to_string(),
                trace_ref: "trace-1".to_string(),
            })
            .await
            .expect("store workflow");
        assert_eq!(workflow.workflow_name, "treasury_disbursement");

        let evidence = context
            .evidence_manifests()
            .store(
                "evidence-1",
                EvidenceManifestV1 {
                    evidence_id: "evidence-1".to_string(),
                    producer: "tests".to_string(),
                    artifact_hash: "abc".to_string(),
                    storage_ref: "surrealdb:evidence/evidence-1".to_string(),
                    retention_class: "standard".to_string(),
                    classification: Classification::Internal,
                    related_decision_refs: vec!["decision-1".to_string()],
                },
            )
            .await
            .expect("store evidence");
        assert_eq!(evidence.evidence.producer, "tests");

        let event = context
            .recorded_events()
            .append(
                "event-1",
                RecordedEventV1 {
                    envelope: EventEnvelopeV1::new(
                        "workflow.started",
                        ActorRef("ops:user-1".to_string()),
                        "corr-1",
                        None,
                        Classification::Internal,
                        "schemas/events/v1/workflow-started",
                        "deadbeef",
                    ),
                    payload_ref: contracts::PayloadRefV1 {
                        schema_ref: "schemas/contracts/v1/workflow-execution".to_string(),
                        record_id: "wf-1".to_string(),
                    },
                },
            )
            .await
            .expect("append event");
        assert_eq!(event.id, "event-1");

        let disbursement = context
            .treasury_disbursements()
            .store(
                "disbursement-1",
                TreasuryDisbursementRecordedV1 {
                    disbursement_id: "disbursement-1".to_string(),
                    workflow_execution_id: "wf-1".to_string(),
                    ledger_ref: "ledger:primary".to_string(),
                    amount_minor: 5000,
                    currency: "USD".to_string(),
                    beneficiary: "Vendor".to_string(),
                    approved_by_roles: Vec::new(),
                },
            )
            .await
            .expect("store disbursement");
        assert_eq!(disbursement.disbursement.currency, "USD");

        let loaded = context
            .workflow_executions()
            .load("wf-1")
            .await
            .expect("load workflow")
            .expect("workflow present");
        assert_eq!(loaded.trace_ref, "trace-1");

        let source = context
            .knowledge_sources()
            .store(KnowledgeSourceV1 {
                source_id: "source-1".to_string(),
                ingestion_id: "ingest-1".to_string(),
                kind: KnowledgeSourceKindV1::Imf,
                title: "IMF External Accounts".to_string(),
                country_area: "Japan".to_string(),
                series_name: Some("BOP".to_string()),
                source_url: "https://www.imf.org/example".to_string(),
                source_domain: "www.imf.org".to_string(),
                format: KnowledgeDocumentFormatV1::Json,
                mime_type: "application/json".to_string(),
                classification: Classification::Internal,
                acquired_at: Utc::now(),
                content_digest: "digest-1".to_string(),
                content_text: "Current account surplus remains positive.".to_string(),
                provenance_tier: KnowledgeSourceProvenanceV1::Primary,
                evidence_use: KnowledgeEvidenceUseV1::Evidence,
                last_observation: Some("2026-02".to_string()),
                units: Some("USD bn".to_string()),
                transform: Some("yoy".to_string()),
                release_lag: Some("T+30d".to_string()),
                quality_flags: Vec::new(),
                notes: vec!["Primary source".to_string()],
                governance_notes: vec!["Primary IMF source".to_string()],
                provider_metadata: std::collections::BTreeMap::new(),
            })
            .await
            .expect("store source");
        assert_eq!(source.id, "source-1");

        let capsule = context
            .knowledge_capsules()
            .store(KnowledgeCapsuleV1 {
                capsule_id: "capsule-1".to_string(),
                publication_id: "publication-1".to_string(),
                title: "Macro capsule".to_string(),
                source_ids: vec!["source-1".to_string()],
                source_count: 1,
                storage_ref: "memvid:capsule-1".to_string(),
                artifact_hash: "capsule-hash".to_string(),
                version: "v1".to_string(),
                memvid_version: "2.0.138".to_string(),
                published_at: Utc::now(),
                classification: Classification::Internal,
                retention_class: "institutional_record".to_string(),
            })
            .await
            .expect("store capsule");
        assert_eq!(capsule.capsule.storage_ref, "memvid:capsule-1");
        assert!(context
            .knowledge_capsules()
            .latest_status()
            .await
            .expect("latest status")
            .is_some());

        let analysis = context
            .knowledge_analyses()
            .store(MacroFinancialAnalysisV1 {
                analysis_id: "analysis-1".to_string(),
                generated_at: Utc::now(),
                trace_ref: "trace-1".to_string(),
                objective: AnalysisObjectiveV1::PolicyEval,
                horizon: AnalysisHorizonV1::Nowcast,
                coverage: AnalysisCoverageV1 {
                    countries: vec!["Japan".to_string()],
                    regions: Vec::new(),
                    currencies: vec!["JPY".to_string()],
                    fx_pairs: vec!["USD/JPY".to_string()],
                    asset_classes: vec!["rates".to_string()],
                },
                problem_contract: ProblemContractV1 {
                    objective: AnalysisObjectiveV1::PolicyEval,
                    horizon: AnalysisHorizonV1::Nowcast,
                    target_countries: vec!["Japan".to_string()],
                    target_regions: Vec::new(),
                    target_currencies: vec!["JPY".to_string()],
                    target_fx_pairs: vec!["USD/JPY".to_string()],
                    asset_classes: vec!["rates".to_string()],
                    dependent_variables: vec!["FX bilateral".to_string()],
                    required_inputs: vec!["FX levels".to_string()],
                    missing_inputs: vec![
                        "MISSING: provide balance of payments and IIP components.".to_string(),
                    ],
                },
                data_vintage: "2026-03-09".to_string(),
                required_inputs: vec!["FX levels".to_string()],
                dependent_variables: vec!["FX bilateral".to_string()],
                global_liquidity_phase: GlobalLiquidityPhaseV1::Tighten,
                global_liquidity_funding: GlobalLiquidityFundingConditionsV1 {
                    phase: GlobalLiquidityPhaseV1::Tighten,
                    dominant_transmission_channel: TransmissionChannelV1::CrossBorderBankCredit,
                    dollar_funding_stress_state: "Contained.".to_string(),
                    backstop_availability: "Adequate reserves.".to_string(),
                    missing_inputs: Vec::new(),
                },
                external_accounts_map: ExternalAccountsBalanceSheetMapV1 {
                    current_account_pressures: "Surplus persists.".to_string(),
                    financial_account_decomposition: "Portfolio flows dominate.".to_string(),
                    external_debt_structure: "Mostly local currency debt.".to_string(),
                    currency_mismatch_indicators: "Contained mismatch.".to_string(),
                    marginal_financer: "Portfolio investors".to_string(),
                    flow_reversal_vulnerability: "Portfolio reverses first".to_string(),
                    missing_inputs: Vec::new(),
                },
                policy_regime_diagnosis: PolicyRegimeDiagnosisV1 {
                    monetary_policy_regime: "Inflation targeting".to_string(),
                    credibility_signals: "Credibility remains intact.".to_string(),
                    exchange_rate_regime: "Managed float".to_string(),
                    intervention_pattern: "Smoothing intervention".to_string(),
                    frictions: vec![PolicyFrictionObservationV1 {
                        friction: "FX illiquidity".to_string(),
                        observable_indicators: vec!["volatility".to_string()],
                        confidence: ConfidenceV1::Moderate,
                    }],
                    missing_inputs: Vec::new(),
                },
                driver_decomposition: vec![FxDriverAssessmentV1 {
                    bucket: DriverBucketV1::RateDifferentialsExpectedPolicyPaths,
                    direction: DirectionalBiasV1::Positive,
                    magnitude: SignalMagnitudeV1::Medium,
                    confidence: ConfidenceV1::Moderate,
                    evidence: "Policy spread widened.".to_string(),
                }],
                sovereign_systemic_risk: SovereignSystemicRiskV1 {
                    debt_sustainability_state: "Stable".to_string(),
                    gross_financing_needs: "Manageable".to_string(),
                    rollover_risk: "Contained".to_string(),
                    sovereign_bank_nonbank_nexus: "Present but stable".to_string(),
                    key_amplifiers: vec!["Leverage".to_string()],
                    cross_border_spillovers: "Portfolio and swaps.".to_string(),
                    missing_inputs: Vec::new(),
                },
                executive_brief: ExecutiveBriefV1 {
                    as_of_date: "2026-03-09".to_string(),
                    as_of_timezone: "America/Los_Angeles".to_string(),
                    data_vintage: "2026-03-09".to_string(),
                    objective: AnalysisObjectiveV1::PolicyEval,
                    horizon: AnalysisHorizonV1::Nowcast,
                    coverage: AnalysisCoverageV1 {
                        countries: vec!["Japan".to_string()],
                        regions: Vec::new(),
                        currencies: vec!["JPY".to_string()],
                        fx_pairs: vec!["USD/JPY".to_string()],
                        asset_classes: vec!["rates".to_string()],
                    },
                    key_judgments_facts: vec!["FACT".to_string()],
                    key_judgments_inferences: vec!["INFERENCE".to_string()],
                    key_risks: vec![RankedRiskV1 {
                        risk: "Funding stress".to_string(),
                        summary: "Dollar funding availability tightens.".to_string(),
                        probability: ProbabilityV1::Medium,
                    }],
                    signal_summary: vec![SignalSummaryEntryV1 {
                        signal: "Rate differentials".to_string(),
                        direction: DirectionalBiasV1::Positive,
                        magnitude: SignalMagnitudeV1::Medium,
                        confidence: ConfidenceV1::Moderate,
                        evidence: "Policy spread widened.".to_string(),
                    }],
                    implications: AnalysisImplicationsV1 {
                        policy_evaluation: "Maintain monitoring.".to_string(),
                        investment_strategy: "Prefer hedged exposure.".to_string(),
                        risk_management: "Tighten liquidity limits.".to_string(),
                        long_horizon_strategy: "Track reserve adequacy.".to_string(),
                    },
                },
                data_register: vec![DataRegisterEntryV1 {
                    series_name: "BOP".to_string(),
                    country_area: "Japan".to_string(),
                    source: "IMF".to_string(),
                    frequency: "Monthly".to_string(),
                    last_obs: "2026-02".to_string(),
                    units: "USD bn".to_string(),
                    transform: "yoy".to_string(),
                    lag: "T+30d".to_string(),
                    quality_flag: String::new(),
                    notes: "Primary".to_string(),
                }],
                mechanism_map: MechanismMapV1 {
                    current_account_narrative: "Surplus persists.".to_string(),
                    financial_account_funding_mix: "Portfolio flows dominate.".to_string(),
                    reserves_and_backstops: "Adequate reserves.".to_string(),
                    fx_swap_basis_state: "Basis mildly negative.".to_string(),
                    dollar_funding_stress_state: "Contained.".to_string(),
                    risk_sentiment_linkage: "High beta to global risk.".to_string(),
                    spillover_channels: "Portfolio and swaps.".to_string(),
                },
                scenario_matrix: vec![ScenarioCaseV1 {
                    scenario: ScenarioKindV1::Base,
                    triggers: "Stable policy path".to_string(),
                    transmission_path: "Accounts to funding to FX".to_string(),
                    fx_outcome: "Range-bound".to_string(),
                    capital_flows_outcome: "Steady".to_string(),
                    liquidity_funding_outcome: "Stable".to_string(),
                    systemic_risk_outcome: "Contained".to_string(),
                    policy_response_space: "Moderate".to_string(),
                    strategy_implications: "Keep hedges on.".to_string(),
                    watchlist: vec![WatchlistIndicatorV1 {
                        indicator: "Basis".to_string(),
                        threshold: "< -20bp".to_string(),
                        rationale: "Funding stress".to_string(),
                    }],
                }],
                risk_register: vec![RiskRegisterEntryV1 {
                    risk: "Funding stress".to_string(),
                    mechanism: "Basis widening".to_string(),
                    early_indicators: "Cross-currency basis".to_string(),
                    impact_channels: "FX and liquidity".to_string(),
                    mitigants_or_hedges: "Shorten tenor".to_string(),
                    probability: ProbabilityV1::Medium,
                    confidence: ConfidenceV1::Moderate,
                }],
                knowledge_appendix: KnowledgeAppendixV1 {
                    definitions: vec!["External accounts".to_string()],
                    indicator_dictionary: vec!["Basis".to_string()],
                    playbooks: vec!["Sudden stop".to_string()],
                    common_failure_modes: vec!["Proxy drift".to_string()],
                    source_note: "Primary sources only".to_string(),
                    assumptions_log: vec!["1. Stable policy backdrop.".to_string()],
                },
                source_governance: vec![SourceGovernanceDecisionV1 {
                    source_id: "source-1".to_string(),
                    source_domain: "www.imf.org".to_string(),
                    provenance_tier: KnowledgeSourceProvenanceV1::Primary,
                    evidence_use: KnowledgeEvidenceUseV1::Evidence,
                    accepted: true,
                    reasons: vec!["Primary IMF source".to_string()],
                }],
                assumptions: vec![AnalysisAssumptionV1 {
                    assumption_id: "A1".to_string(),
                    text: "Primary sources dominate evidence.".to_string(),
                    stable: true,
                }],
                inference_steps: vec![InferenceStepV1 {
                    inference_id: "INF-01".to_string(),
                    label: "Funding inference".to_string(),
                    assumption_ids: vec!["A1".to_string()],
                    inputs_used: vec!["BOP".to_string()],
                    resulting_judgment: "Funding remains stable".to_string(),
                }],
                claim_evidence: vec![ClaimEvidenceV1 {
                    claim_id: "claim-1".to_string(),
                    output_section: "Executive Brief".to_string(),
                    claim_kind: ClaimKindV1::Fact,
                    statement: "FACT".to_string(),
                    source_ids: vec!["source-1".to_string()],
                    inference_ids: Vec::new(),
                }],
                pipeline_trace: vec![PipelineStepTraceV1 {
                    step: PipelineStepIdV1::StepA,
                    ordinal: 1,
                    summary: "Problem contract complete".to_string(),
                }],
                source_ids: vec!["source-1".to_string()],
                capsule_id: Some("capsule-1".to_string()),
                rendered_output: "analysis".to_string(),
                retrieval_context: vec!["Surplus persists.".to_string()],
            })
            .await
            .expect("store analysis");
        assert_eq!(analysis.analysis.analysis_id, "analysis-1");

        let edge = context
            .knowledge_edges()
            .store(KnowledgeEdgeV1 {
                edge_id: "edge-1".to_string(),
                from_id: "analysis-1".to_string(),
                to_id: "source-1".to_string(),
                relationship: KnowledgeRelationshipV1::Cites,
                rationale: "Analysis cites source".to_string(),
            })
            .await
            .expect("store edge");
        assert_eq!(edge.edge.from_id, "analysis-1");
    }
}

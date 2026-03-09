use std::collections::BTreeMap;

use futures::future::{self, BoxFuture};
use futures::stream::{self, BoxStream};

use contracts::{
    AgentActionRequestV1, KnowledgePublicationRequestV1, KnowledgePublicationStatusV1,
    KnowledgeSourceIngestRequestV1, MacroFinancialAnalysisRequestV1, MacroFinancialAnalysisV1,
    MarketDataBatchV1, PromotionRecommendationV1, QuantStrategyPromotionRequestV1,
    ServiceBoundaryV1, WorkflowBoundaryV1,
};
use error_model::{InstitutionalError, InstitutionalResult};
use events::EventEnvelopeV1;
use lattice_config::LatticeConfigV1;
use serde::{Deserialize, Serialize};

pub type TransportFuture<T> = BoxFuture<'static, InstitutionalResult<T>>;
pub type EventSubscription = BoxStream<'static, EventEnvelopeV1>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleasedUiAppV1 {
    pub app_id: String,
    pub display_name: String,
    pub desktop_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiDashboardSnapshotV1 {
    pub client_name: String,
    pub services: Vec<ServiceBoundaryV1>,
    pub workflows: Vec<WorkflowBoundaryV1>,
    pub lattice: Option<LatticeConfigV1>,
    pub release_apps: Vec<ReleasedUiAppV1>,
    pub connected_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "command_type", content = "payload", rename_all = "snake_case")]
pub enum PlatformCommandV1 {
    DispatchAgentAction(AgentActionRequestV1),
    PreparePromotion {
        action: AgentActionRequestV1,
        request: QuantStrategyPromotionRequestV1,
    },
    RegisterMarketDataBatch(MarketDataBatchV1),
    SubmitPromotionRecommendation(PromotionRecommendationV1),
    IngestKnowledgeSources(KnowledgeSourceIngestRequestV1),
    PublishKnowledgeCapsule(KnowledgePublicationRequestV1),
    SubmitMacroFinancialAnalysis(MacroFinancialAnalysisRequestV1),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformCommandAckV1 {
    pub command_id: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "query_type", content = "payload", rename_all = "snake_case")]
pub enum PlatformQueryV1 {
    Dashboard,
    SupportedWorkflows,
    RecentEvents { limit: usize },
    GetMacroFinancialAnalysis { analysis_id: String },
    GetLatestKnowledgePublicationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "result_type", content = "payload", rename_all = "snake_case")]
pub enum PlatformQueryResultV1 {
    Dashboard(UiDashboardSnapshotV1),
    SupportedWorkflows(Vec<WorkflowBoundaryV1>),
    RecentEvents(Vec<EventEnvelopeV1>),
    MacroFinancialAnalysis(Box<Option<MacroFinancialAnalysisV1>>),
    KnowledgePublicationStatus(Option<KnowledgePublicationStatusV1>),
}

pub trait InstitutionalPlatformTransport: Send + Sync {
    fn execute_command(&self, command: PlatformCommandV1) -> TransportFuture<PlatformCommandAckV1>;
    fn execute_query(&self, query: PlatformQueryV1) -> TransportFuture<PlatformQueryResultV1>;
    fn subscribe_events(&self) -> EventSubscription;
}

#[derive(Debug, Clone, Default)]
pub struct NoopPlatformTransport;

impl InstitutionalPlatformTransport for NoopPlatformTransport {
    fn execute_command(
        &self,
        _command: PlatformCommandV1,
    ) -> TransportFuture<PlatformCommandAckV1> {
        Box::pin(future::ready(Err(InstitutionalError::PolicyDenied {
            reason: "no platform transport configured".to_string(),
        })))
    }

    fn execute_query(&self, _query: PlatformQueryV1) -> TransportFuture<PlatformQueryResultV1> {
        Box::pin(future::ready(Err(InstitutionalError::PolicyDenied {
            reason: "no platform transport configured".to_string(),
        })))
    }

    fn subscribe_events(&self) -> EventSubscription {
        Box::pin(stream::empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalPlatformClientV1 {
    pub client_name: String,
    pub supported_services: Vec<ServiceBoundaryV1>,
    pub supported_workflows: Vec<WorkflowBoundaryV1>,
    pub lattice_config: Option<LatticeConfigV1>,
}

impl InstitutionalPlatformClientV1 {
    #[must_use]
    pub fn prepare_action(&self, action: AgentActionRequestV1) -> AgentActionRequestV1 {
        action
    }

    #[must_use]
    pub fn receive_event(&self, envelope: EventEnvelopeV1) -> EventEnvelopeV1 {
        envelope
    }

    #[must_use]
    pub fn prepare_quant_strategy_promotion(
        &self,
        action: AgentActionRequestV1,
        request: QuantStrategyPromotionRequestV1,
    ) -> (AgentActionRequestV1, QuantStrategyPromotionRequestV1) {
        (action, request)
    }

    #[must_use]
    pub fn register_market_data_batch(&self, batch: MarketDataBatchV1) -> MarketDataBatchV1 {
        batch
    }

    #[must_use]
    pub fn submit_promotion_recommendation(
        &self,
        recommendation: PromotionRecommendationV1,
    ) -> PromotionRecommendationV1 {
        recommendation
    }

    #[must_use]
    pub fn ingest_knowledge_sources(
        &self,
        request: KnowledgeSourceIngestRequestV1,
    ) -> KnowledgeSourceIngestRequestV1 {
        request
    }

    #[must_use]
    pub fn publish_knowledge_capsule(
        &self,
        request: KnowledgePublicationRequestV1,
    ) -> KnowledgePublicationRequestV1 {
        request
    }

    #[must_use]
    pub fn submit_macro_financial_analysis(
        &self,
        request: MacroFinancialAnalysisRequestV1,
    ) -> MacroFinancialAnalysisRequestV1 {
        request
    }

    #[must_use]
    pub fn dashboard_snapshot(
        &self,
        release_apps: Vec<ReleasedUiAppV1>,
        connected_cache: bool,
    ) -> UiDashboardSnapshotV1 {
        UiDashboardSnapshotV1 {
            client_name: self.client_name.clone(),
            services: self.supported_services.clone(),
            workflows: self.supported_workflows.clone(),
            lattice: self.lattice_config.clone(),
            release_apps,
            connected_cache,
        }
    }
}

#[derive(Clone)]
pub struct InstitutionalPlatformRuntimeClient<T>
where
    T: InstitutionalPlatformTransport,
{
    manifest: InstitutionalPlatformClientV1,
    transport: T,
}

impl<T> InstitutionalPlatformRuntimeClient<T>
where
    T: InstitutionalPlatformTransport,
{
    #[must_use]
    pub fn new(manifest: InstitutionalPlatformClientV1, transport: T) -> Self {
        Self {
            manifest,
            transport,
        }
    }

    #[must_use]
    pub fn manifest(&self) -> &InstitutionalPlatformClientV1 {
        &self.manifest
    }

    pub async fn execute_command(
        &self,
        command: PlatformCommandV1,
    ) -> InstitutionalResult<PlatformCommandAckV1> {
        self.transport.execute_command(command).await
    }

    pub async fn query_dashboard(&self) -> InstitutionalResult<UiDashboardSnapshotV1> {
        match self
            .transport
            .execute_query(PlatformQueryV1::Dashboard)
            .await?
        {
            PlatformQueryResultV1::Dashboard(snapshot) => Ok(snapshot),
            _ => Err(InstitutionalError::InvariantViolation {
                invariant: "dashboard query returned non-dashboard payload".to_string(),
            }),
        }
    }

    pub async fn query_supported_workflows(&self) -> InstitutionalResult<Vec<WorkflowBoundaryV1>> {
        match self
            .transport
            .execute_query(PlatformQueryV1::SupportedWorkflows)
            .await?
        {
            PlatformQueryResultV1::SupportedWorkflows(workflows) => Ok(workflows),
            _ => Err(InstitutionalError::InvariantViolation {
                invariant: "workflow query returned non-workflow payload".to_string(),
            }),
        }
    }

    pub async fn query_macro_financial_analysis(
        &self,
        analysis_id: impl Into<String>,
    ) -> InstitutionalResult<Option<MacroFinancialAnalysisV1>> {
        match self
            .transport
            .execute_query(PlatformQueryV1::GetMacroFinancialAnalysis {
                analysis_id: analysis_id.into(),
            })
            .await?
        {
            PlatformQueryResultV1::MacroFinancialAnalysis(analysis) => Ok(*analysis),
            _ => Err(InstitutionalError::InvariantViolation {
                invariant: "macro analysis query returned non-analysis payload".to_string(),
            }),
        }
    }

    pub async fn query_latest_knowledge_publication_status(
        &self,
    ) -> InstitutionalResult<Option<KnowledgePublicationStatusV1>> {
        match self
            .transport
            .execute_query(PlatformQueryV1::GetLatestKnowledgePublicationStatus)
            .await?
        {
            PlatformQueryResultV1::KnowledgePublicationStatus(status) => Ok(status),
            _ => Err(InstitutionalError::InvariantViolation {
                invariant: "knowledge status query returned non-status payload".to_string(),
            }),
        }
    }

    #[must_use]
    pub fn subscribe_events(&self) -> EventSubscription {
        self.transport.subscribe_events()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryPlatformTransport {
    dashboard: UiDashboardSnapshotV1,
    recent_events: Vec<EventEnvelopeV1>,
    analyses: BTreeMap<String, MacroFinancialAnalysisV1>,
    latest_publication_status: Option<KnowledgePublicationStatusV1>,
}

impl MemoryPlatformTransport {
    #[must_use]
    pub fn new(dashboard: UiDashboardSnapshotV1, recent_events: Vec<EventEnvelopeV1>) -> Self {
        Self {
            dashboard,
            recent_events,
            analyses: BTreeMap::new(),
            latest_publication_status: None,
        }
    }

    #[must_use]
    pub fn with_analysis(mut self, analysis: MacroFinancialAnalysisV1) -> Self {
        self.analyses.insert(analysis.analysis_id.clone(), analysis);
        self
    }

    #[must_use]
    pub fn with_latest_publication_status(mut self, status: KnowledgePublicationStatusV1) -> Self {
        self.latest_publication_status = Some(status);
        self
    }
}

impl InstitutionalPlatformTransport for MemoryPlatformTransport {
    fn execute_command(
        &self,
        _command: PlatformCommandV1,
    ) -> TransportFuture<PlatformCommandAckV1> {
        Box::pin(future::ready(Ok(PlatformCommandAckV1 {
            command_id: "memory-ack".to_string(),
            accepted: true,
        })))
    }

    fn execute_query(&self, query: PlatformQueryV1) -> TransportFuture<PlatformQueryResultV1> {
        let result = match query {
            PlatformQueryV1::Dashboard => PlatformQueryResultV1::Dashboard(self.dashboard.clone()),
            PlatformQueryV1::SupportedWorkflows => {
                PlatformQueryResultV1::SupportedWorkflows(self.dashboard.workflows.clone())
            }
            PlatformQueryV1::RecentEvents { limit } => PlatformQueryResultV1::RecentEvents(
                self.recent_events.iter().take(limit).cloned().collect(),
            ),
            PlatformQueryV1::GetMacroFinancialAnalysis { analysis_id } => {
                PlatformQueryResultV1::MacroFinancialAnalysis(Box::new(
                    self.analyses.get(&analysis_id).cloned(),
                ))
            }
            PlatformQueryV1::GetLatestKnowledgePublicationStatus => {
                PlatformQueryResultV1::KnowledgePublicationStatus(
                    self.latest_publication_status.clone(),
                )
            }
        };
        Box::pin(future::ready(Ok(result)))
    }

    fn subscribe_events(&self) -> EventSubscription {
        Box::pin(stream::iter(self.recent_events.clone()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contracts::{
        AnalysisCoverageV1, AnalysisHorizonV1, AnalysisImplicationsV1, AnalysisObjectiveV1,
        Classification, ConfidenceV1, DataRegisterEntryV1, DirectionalBiasV1, DriverBucketV1,
        ExecutiveBriefV1, FxDriverAssessmentV1, GlobalLiquidityPhaseV1, KnowledgeAppendixV1,
        KnowledgePublicationStatusV1, MechanismMapV1, ProbabilityV1, RankedRiskV1,
        RiskRegisterEntryV1, ScenarioCaseV1, ScenarioKindV1, SignalMagnitudeV1,
        SignalSummaryEntryV1, WatchlistIndicatorV1,
    };
    use futures::StreamExt;
    use identity::ActorRef;

    use super::*;

    #[tokio::test]
    async fn memory_transport_serves_dashboard_and_events() {
        let manifest = InstitutionalPlatformClientV1 {
            client_name: "ui-shell".to_string(),
            supported_services: Vec::new(),
            supported_workflows: Vec::new(),
            lattice_config: None,
        };
        let dashboard = manifest.dashboard_snapshot(
            vec![ReleasedUiAppV1 {
                app_id: "system.control-center".to_string(),
                display_name: "Control Center".to_string(),
                desktop_enabled: true,
            }],
            true,
        );
        let event = EventEnvelopeV1::new(
            "shell.started",
            ActorRef("ui-shell".to_string()),
            "corr-1",
            None,
            Classification::Internal,
            "schemas/events/v1/shell-started",
            "abc",
        );
        let analysis = MacroFinancialAnalysisV1 {
            analysis_id: "analysis-1".to_string(),
            generated_at: Utc
                .with_ymd_and_hms(2026, 3, 9, 12, 0, 0)
                .single()
                .expect("time"),
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
            data_vintage: "2026-03-09".to_string(),
            required_inputs: vec!["FX levels".to_string()],
            dependent_variables: vec!["FX bilateral".to_string()],
            global_liquidity_phase: GlobalLiquidityPhaseV1::Tighten,
            driver_decomposition: vec![FxDriverAssessmentV1 {
                bucket: DriverBucketV1::FlowShocks,
                direction: DirectionalBiasV1::Positive,
                magnitude: SignalMagnitudeV1::Medium,
                confidence: ConfidenceV1::Moderate,
                evidence: "Flows stabilized.".to_string(),
            }],
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
                    summary: "Moderate".to_string(),
                    probability: ProbabilityV1::Medium,
                }],
                signal_summary: vec![SignalSummaryEntryV1 {
                    signal: "Flows".to_string(),
                    direction: DirectionalBiasV1::Positive,
                    magnitude: SignalMagnitudeV1::Medium,
                    confidence: ConfidenceV1::Moderate,
                    evidence: "Flows stabilized.".to_string(),
                }],
                implications: AnalysisImplicationsV1 {
                    policy_evaluation: "Maintain flexibility.".to_string(),
                    investment_strategy: "Keep hedges.".to_string(),
                    risk_management: "Watch funding.".to_string(),
                    long_horizon_strategy: "Track buffers.".to_string(),
                },
            },
            data_register: vec![DataRegisterEntryV1 {
                series_name: "DXY".to_string(),
                country_area: "United States".to_string(),
                source: "FRED".to_string(),
                frequency: "Daily".to_string(),
                last_obs: "2026-03-09".to_string(),
                units: "index".to_string(),
                transform: "level".to_string(),
                lag: "T+1d".to_string(),
                quality_flag: String::new(),
                notes: "Primary".to_string(),
            }],
            mechanism_map: MechanismMapV1 {
                current_account_narrative: "Stable".to_string(),
                financial_account_funding_mix: "Portfolio".to_string(),
                reserves_and_backstops: "Adequate".to_string(),
                fx_swap_basis_state: "Contained".to_string(),
                dollar_funding_stress_state: "Moderate".to_string(),
                risk_sentiment_linkage: "Global cycle".to_string(),
                spillover_channels: "Flows".to_string(),
            },
            scenario_matrix: vec![ScenarioCaseV1 {
                scenario: ScenarioKindV1::Base,
                triggers: "Baseline".to_string(),
                transmission_path: "Accounts -> funding -> FX".to_string(),
                fx_outcome: "Range bound".to_string(),
                capital_flows_outcome: "Steady".to_string(),
                liquidity_funding_outcome: "Stable".to_string(),
                systemic_risk_outcome: "Contained".to_string(),
                policy_response_space: "Moderate".to_string(),
                strategy_implications: "Keep hedges".to_string(),
                watchlist: vec![WatchlistIndicatorV1 {
                    indicator: "Basis".to_string(),
                    threshold: "< -20bp".to_string(),
                    rationale: "Funding".to_string(),
                }],
            }],
            risk_register: vec![RiskRegisterEntryV1 {
                risk: "Funding stress".to_string(),
                mechanism: "Basis".to_string(),
                early_indicators: "Basis".to_string(),
                impact_channels: "FX".to_string(),
                mitigants_or_hedges: "Hedge".to_string(),
                probability: ProbabilityV1::Medium,
                confidence: ConfidenceV1::Moderate,
            }],
            knowledge_appendix: KnowledgeAppendixV1 {
                definitions: vec!["External accounts".to_string()],
                indicator_dictionary: vec!["Basis".to_string()],
                playbooks: vec!["Sudden stop".to_string()],
                common_failure_modes: vec!["Proxy drift".to_string()],
                source_note: "Primary".to_string(),
                assumptions_log: vec!["1. Stable".to_string()],
            },
            source_ids: vec!["source-1".to_string()],
            capsule_id: Some("capsule-1".to_string()),
            rendered_output: "rendered".to_string(),
            retrieval_context: vec!["context".to_string()],
        };
        let transport = MemoryPlatformTransport::new(dashboard.clone(), vec![event.clone()])
            .with_analysis(analysis.clone())
            .with_latest_publication_status(KnowledgePublicationStatusV1 {
                publication_id: "publication-1".to_string(),
                capsule_id: "capsule-1".to_string(),
                published_at: analysis.generated_at,
                source_count: 1,
                storage_ref: "memvid:/tmp/capsule-1.mv2".to_string(),
                artifact_hash: "hash".to_string(),
                version: "v1".to_string(),
            });
        let client = InstitutionalPlatformRuntimeClient::new(manifest, transport);

        assert_eq!(
            client.query_dashboard().await.expect("dashboard"),
            dashboard
        );
        assert_eq!(
            client
                .query_supported_workflows()
                .await
                .expect("workflows")
                .len(),
            0
        );
        assert_eq!(
            client
                .query_macro_financial_analysis("analysis-1")
                .await
                .expect("analysis")
                .expect("present"),
            analysis
        );
        assert!(client
            .query_latest_knowledge_publication_status()
            .await
            .expect("status")
            .is_some());
        let events = client.subscribe_events().collect::<Vec<_>>().await;
        assert_eq!(events, vec![event]);
    }
}

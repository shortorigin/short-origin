use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Classification;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSourceKindV1 {
    Imf,
    Bis,
    WorldBank,
    Fred,
    OfficialDocument,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeDocumentFormatV1 {
    Json,
    Xml,
    Html,
    Pdf,
    Text,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisObjectiveV1 {
    PolicyEval,
    InvestmentStrategy,
    RiskMgmt,
    StrategyLonghorizon,
}

impl AnalysisObjectiveV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::PolicyEval => "POLICY_EVAL",
            Self::InvestmentStrategy => "INVESTMENT_STRATEGY",
            Self::RiskMgmt => "RISK_MGMT",
            Self::StrategyLonghorizon => "STRATEGY_LONGHORIZON",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisHorizonV1 {
    Nowcast,
    OneToThreeMonths,
    ThreeToTwelveMonths,
    OneToFiveYears,
}

impl AnalysisHorizonV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Nowcast => "NOWCAST",
            Self::OneToThreeMonths => "1-3M",
            Self::ThreeToTwelveMonths => "3-12M",
            Self::OneToFiveYears => "1-5Y",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProbabilityV1 {
    Low,
    Medium,
    High,
}

impl ProbabilityV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceV1 {
    Weak,
    Moderate,
    Strong,
}

impl ConfidenceV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Weak => "WEAK",
            Self::Moderate => "MODERATE",
            Self::Strong => "STRONG",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalMagnitudeV1 {
    Low,
    Medium,
    High,
}

impl SignalMagnitudeV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GlobalLiquidityPhaseV1 {
    Ease,
    Tighten,
    Stress,
}

impl GlobalLiquidityPhaseV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Ease => "EASE",
            Self::Tighten => "TIGHTEN",
            Self::Stress => "STRESS",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKindV1 {
    Base,
    Upside,
    Downside,
    TailLiquidityEvent,
}

impl ScenarioKindV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Base => "BASE",
            Self::Upside => "UPSIDE",
            Self::Downside => "DOWNSIDE",
            Self::TailLiquidityEvent => "TAIL_LIQUIDITY_EVENT",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QualityFlagV1 {
    BreaksInSeries,
    Estimated,
    ProxyUsed,
}

impl QualityFlagV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::BreaksInSeries => "BREAKS_IN_SERIES",
            Self::Estimated => "ESTIMATED",
            Self::ProxyUsed => "PROXY_USED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRelationshipV1 {
    DerivedFrom,
    Supports,
    Cites,
    RetainedBy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DriverBucketV1 {
    RateDifferentialsExpectedPolicyPaths,
    RiskSentimentGlobalFinancialCycleExposure,
    FlowShocks,
    TermsOfTradeCommodityChannel,
    FundingHedgingPremia,
    GeopoliticalFractureShocks,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DirectionalBiasV1 {
    Positive,
    Negative,
    Stable,
    Mixed,
}

impl DirectionalBiasV1 {
    #[must_use]
    pub fn directive_label(self) -> &'static str {
        match self {
            Self::Positive => "+",
            Self::Negative => "-",
            Self::Stable => "STABLE",
            Self::Mixed => "MIXED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AnalysisCoverageV1 {
    pub countries: Vec<String>,
    pub regions: Vec<String>,
    pub currencies: Vec<String>,
    pub fx_pairs: Vec<String>,
    pub asset_classes: Vec<String>,
}

impl AnalysisCoverageV1 {
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.countries.is_empty() {
            parts.push(format!("countries={}", self.countries.join(", ")));
        }
        if !self.regions.is_empty() {
            parts.push(format!("regions={}", self.regions.join(", ")));
        }
        if !self.currencies.is_empty() {
            parts.push(format!("currencies={}", self.currencies.join(", ")));
        }
        if !self.fx_pairs.is_empty() {
            parts.push(format!("fx_pairs={}", self.fx_pairs.join(", ")));
        }
        if !self.asset_classes.is_empty() {
            parts.push(format!("asset_classes={}", self.asset_classes.join(", ")));
        }
        if parts.is_empty() {
            "MISSING".to_owned()
        } else {
            parts.join("; ")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SourceConstraintsV1 {
    pub allowed_sources: Vec<String>,
    pub forbidden_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSourceFetchSpecV1 {
    pub source_id: String,
    pub kind: KnowledgeSourceKindV1,
    pub title: String,
    pub country_area: String,
    pub url: String,
    pub series_name: Option<String>,
    pub expected_format: KnowledgeDocumentFormatV1,
    pub release_lag: Option<String>,
    pub units: Option<String>,
    pub transform: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSourceIngestRequestV1 {
    pub ingestion_id: String,
    pub classification: Classification,
    pub sources: Vec<KnowledgeSourceFetchSpecV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSourceV1 {
    pub source_id: String,
    pub ingestion_id: String,
    pub kind: KnowledgeSourceKindV1,
    pub title: String,
    pub country_area: String,
    pub series_name: Option<String>,
    pub source_url: String,
    pub source_domain: String,
    pub format: KnowledgeDocumentFormatV1,
    pub mime_type: String,
    pub classification: Classification,
    pub acquired_at: DateTime<Utc>,
    pub content_digest: String,
    pub content_text: String,
    pub last_observation: Option<String>,
    pub units: Option<String>,
    pub transform: Option<String>,
    pub release_lag: Option<String>,
    pub quality_flags: Vec<QualityFlagV1>,
    pub notes: Vec<String>,
    pub provider_metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgePublicationRequestV1 {
    pub publication_id: String,
    pub capsule_id: String,
    pub title: String,
    pub source_ids: Vec<String>,
    pub classification: Classification,
    pub retention_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeCapsuleV1 {
    pub capsule_id: String,
    pub publication_id: String,
    pub title: String,
    pub source_ids: Vec<String>,
    pub source_count: usize,
    pub storage_ref: String,
    pub artifact_hash: String,
    pub version: String,
    pub memvid_version: String,
    pub published_at: DateTime<Utc>,
    pub classification: Classification,
    pub retention_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgePublicationStatusV1 {
    pub publication_id: String,
    pub capsule_id: String,
    pub published_at: DateTime<Utc>,
    pub source_count: usize,
    pub storage_ref: String,
    pub artifact_hash: String,
    pub version: String,
}

impl KnowledgePublicationStatusV1 {
    #[must_use]
    pub fn from_capsule(capsule: &KnowledgeCapsuleV1) -> Self {
        Self {
            publication_id: capsule.publication_id.clone(),
            capsule_id: capsule.capsule_id.clone(),
            published_at: capsule.published_at,
            source_count: capsule.source_count,
            storage_ref: capsule.storage_ref.clone(),
            artifact_hash: capsule.artifact_hash.clone(),
            version: capsule.version.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeEdgeV1 {
    pub edge_id: String,
    pub from_id: String,
    pub to_id: String,
    pub relationship: KnowledgeRelationshipV1,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MacroFinancialAnalysisRequestV1 {
    pub analysis_id: String,
    pub objective: AnalysisObjectiveV1,
    pub horizon: AnalysisHorizonV1,
    pub coverage: AnalysisCoverageV1,
    pub data_vintage: Option<String>,
    pub source_ids: Vec<String>,
    pub capsule_id: Option<String>,
    pub classification: Classification,
    pub constraints: SourceConstraintsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankedRiskV1 {
    pub risk: String,
    pub summary: String,
    pub probability: ProbabilityV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignalSummaryEntryV1 {
    pub signal: String,
    pub direction: DirectionalBiasV1,
    pub magnitude: SignalMagnitudeV1,
    pub confidence: ConfidenceV1,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisImplicationsV1 {
    pub policy_evaluation: String,
    pub investment_strategy: String,
    pub risk_management: String,
    pub long_horizon_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutiveBriefV1 {
    pub as_of_date: String,
    pub as_of_timezone: String,
    pub data_vintage: String,
    pub objective: AnalysisObjectiveV1,
    pub horizon: AnalysisHorizonV1,
    pub coverage: AnalysisCoverageV1,
    pub key_judgments_facts: Vec<String>,
    pub key_judgments_inferences: Vec<String>,
    pub key_risks: Vec<RankedRiskV1>,
    pub signal_summary: Vec<SignalSummaryEntryV1>,
    pub implications: AnalysisImplicationsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataRegisterEntryV1 {
    pub series_name: String,
    pub country_area: String,
    pub source: String,
    pub frequency: String,
    pub last_obs: String,
    pub units: String,
    pub transform: String,
    pub lag: String,
    pub quality_flag: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MechanismMapV1 {
    pub current_account_narrative: String,
    pub financial_account_funding_mix: String,
    pub reserves_and_backstops: String,
    pub fx_swap_basis_state: String,
    pub dollar_funding_stress_state: String,
    pub risk_sentiment_linkage: String,
    pub spillover_channels: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FxDriverAssessmentV1 {
    pub bucket: DriverBucketV1,
    pub direction: DirectionalBiasV1,
    pub magnitude: SignalMagnitudeV1,
    pub confidence: ConfidenceV1,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchlistIndicatorV1 {
    pub indicator: String,
    pub threshold: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioCaseV1 {
    pub scenario: ScenarioKindV1,
    pub triggers: String,
    pub transmission_path: String,
    pub fx_outcome: String,
    pub capital_flows_outcome: String,
    pub liquidity_funding_outcome: String,
    pub systemic_risk_outcome: String,
    pub policy_response_space: String,
    pub strategy_implications: String,
    pub watchlist: Vec<WatchlistIndicatorV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskRegisterEntryV1 {
    pub risk: String,
    pub mechanism: String,
    pub early_indicators: String,
    pub impact_channels: String,
    pub mitigants_or_hedges: String,
    pub probability: ProbabilityV1,
    pub confidence: ConfidenceV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeAppendixV1 {
    pub definitions: Vec<String>,
    pub indicator_dictionary: Vec<String>,
    pub playbooks: Vec<String>,
    pub common_failure_modes: Vec<String>,
    pub source_note: String,
    pub assumptions_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MacroFinancialAnalysisV1 {
    pub analysis_id: String,
    pub generated_at: DateTime<Utc>,
    pub trace_ref: String,
    pub objective: AnalysisObjectiveV1,
    pub horizon: AnalysisHorizonV1,
    pub coverage: AnalysisCoverageV1,
    pub data_vintage: String,
    pub required_inputs: Vec<String>,
    pub dependent_variables: Vec<String>,
    pub global_liquidity_phase: GlobalLiquidityPhaseV1,
    pub driver_decomposition: Vec<FxDriverAssessmentV1>,
    pub executive_brief: ExecutiveBriefV1,
    pub data_register: Vec<DataRegisterEntryV1>,
    pub mechanism_map: MechanismMapV1,
    pub scenario_matrix: Vec<ScenarioCaseV1>,
    pub risk_register: Vec<RiskRegisterEntryV1>,
    pub knowledge_appendix: KnowledgeAppendixV1,
    pub source_ids: Vec<String>,
    pub capsule_id: Option<String>,
    pub rendered_output: String,
    pub retrieval_context: Vec<String>,
}

use chrono::{TimeZone, Utc};
use contracts::{
    AnalysisCoverageV1, AnalysisHorizonV1, AnalysisImplicationsV1, AnalysisObjectiveV1,
    Classification, ConfidenceV1, DataRegisterEntryV1, DirectionalBiasV1, DriverBucketV1,
    ExecutiveBriefV1, FxDriverAssessmentV1, GlobalLiquidityPhaseV1, KnowledgeAppendixV1,
    KnowledgeDocumentFormatV1, KnowledgePublicationRequestV1, KnowledgeRelationshipV1,
    KnowledgeSourceFetchSpecV1, KnowledgeSourceIngestRequestV1, KnowledgeSourceKindV1,
    MacroFinancialAnalysisRequestV1, MacroFinancialAnalysisV1, MechanismMapV1, ProbabilityV1,
    RankedRiskV1, RiskRegisterEntryV1, ScenarioCaseV1, ScenarioKindV1, SignalMagnitudeV1,
    SignalSummaryEntryV1, SourceConstraintsV1, WatchlistIndicatorV1,
};

#[test]
fn knowledge_contracts_round_trip_through_json() {
    let request = KnowledgeSourceIngestRequestV1 {
        ingestion_id: "ingest-1".to_string(),
        classification: Classification::Internal,
        sources: vec![KnowledgeSourceFetchSpecV1 {
            source_id: "source-1".to_string(),
            kind: KnowledgeSourceKindV1::Imf,
            title: "IMF BOP".to_string(),
            country_area: "Japan".to_string(),
            url: "https://www.imf.org/example".to_string(),
            series_name: Some("BOP".to_string()),
            expected_format: KnowledgeDocumentFormatV1::Xml,
            release_lag: Some("T+30d".to_string()),
            units: Some("USD bn".to_string()),
            transform: Some("yoy".to_string()),
            notes: vec!["Primary".to_string()],
        }],
    };
    let publication = KnowledgePublicationRequestV1 {
        publication_id: "publication-1".to_string(),
        capsule_id: "capsule-1".to_string(),
        title: "GMF capsule".to_string(),
        source_ids: vec!["source-1".to_string()],
        classification: Classification::Internal,
        retention_class: "institutional_record".to_string(),
    };
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
            evidence: "Flows stabilized".to_string(),
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
                evidence: "Flows stabilized".to_string(),
            }],
            implications: AnalysisImplicationsV1 {
                policy_evaluation: "Maintain flexibility".to_string(),
                investment_strategy: "Keep hedges".to_string(),
                risk_management: "Watch funding".to_string(),
                long_horizon_strategy: "Track buffers".to_string(),
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

    let request_json = serde_json::to_value(&request).expect("serialize request");
    let publication_json = serde_json::to_value(&publication).expect("serialize publication");
    let analysis_json = serde_json::to_value(&analysis).expect("serialize analysis");

    let _: KnowledgeSourceIngestRequestV1 =
        serde_json::from_value(request_json).expect("parse request");
    let _: KnowledgePublicationRequestV1 =
        serde_json::from_value(publication_json).expect("parse publication");
    let _: MacroFinancialAnalysisV1 =
        serde_json::from_value(analysis_json).expect("parse analysis");
}

#[test]
fn knowledge_enums_preserve_directive_labels() {
    assert_eq!(
        AnalysisObjectiveV1::PolicyEval.directive_label(),
        "POLICY_EVAL"
    );
    assert_eq!(AnalysisHorizonV1::Nowcast.directive_label(), "NOWCAST");
    assert_eq!(
        ScenarioKindV1::TailLiquidityEvent.directive_label(),
        "TAIL_LIQUIDITY_EVENT"
    );
    assert_eq!(
        KnowledgeRelationshipV1::DerivedFrom,
        KnowledgeRelationshipV1::DerivedFrom
    );

    let request = MacroFinancialAnalysisRequestV1 {
        analysis_id: "analysis-2".to_string(),
        objective: AnalysisObjectiveV1::RiskMgmt,
        horizon: AnalysisHorizonV1::ThreeToTwelveMonths,
        coverage: AnalysisCoverageV1 {
            countries: vec!["Brazil".to_string()],
            regions: Vec::new(),
            currencies: vec!["BRL".to_string()],
            fx_pairs: Vec::new(),
            asset_classes: vec!["credit".to_string()],
        },
        data_vintage: None,
        source_ids: vec!["source-1".to_string()],
        capsule_id: None,
        classification: Classification::Internal,
        constraints: SourceConstraintsV1::default(),
    };
    assert_eq!(request.objective.directive_label(), "RISK_MGMT");
}

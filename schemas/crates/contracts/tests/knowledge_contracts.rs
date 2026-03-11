use chrono::{TimeZone, Utc};
use contracts::{
    AnalysisAssumptionV1, AnalysisCoverageV1, AnalysisHorizonV1, AnalysisImplicationsV1,
    AnalysisObjectiveV1, ClaimEvidenceV1, ClaimKindV1, Classification, ConfidenceV1,
    DataRegisterEntryV1, DirectionalBiasV1, DriverBucketV1, ExecutiveBriefV1,
    ExternalAccountsBalanceSheetMapV1, FxDriverAssessmentV1, GlobalLiquidityFundingConditionsV1,
    GlobalLiquidityPhaseV1, InferenceStepV1, KnowledgeAppendixV1, KnowledgeDocumentFormatV1,
    KnowledgeEvidenceUseV1, KnowledgePublicationRequestV1, KnowledgeRelationshipV1,
    KnowledgeSourceFetchSpecV1, KnowledgeSourceIngestRequestV1, KnowledgeSourceKindV1,
    KnowledgeSourceProvenanceV1, MacroFinancialAnalysisRequestV1, MacroFinancialAnalysisV1,
    MacroFinancialDirectInputsV1, MechanismMapV1, PipelineStepIdV1, PipelineStepTraceV1,
    PolicyFrictionObservationV1, PolicyRegimeDiagnosisV1, ProbabilityV1, ProblemContractV1,
    RankedRiskV1, RiskRegisterEntryV1, ScenarioCaseV1, ScenarioKindV1, SignalMagnitudeV1,
    SignalSummaryEntryV1, SourceConstraintsV1, SourceGovernanceDecisionV1, SovereignSystemicRiskV1,
    TransmissionChannelV1, WatchlistIndicatorV1,
};

#[test]
fn knowledge_contracts_round_trip_through_json() {
    let request = KnowledgeSourceIngestRequestV1 {
        ingestion_id: "ingest-1".to_string(),
        classification: Classification::Internal,
        constraints: SourceConstraintsV1::default(),
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
        constraints: SourceConstraintsV1::default(),
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
            dollar_funding_stress_state: "Moderate".to_string(),
            backstop_availability: "Adequate".to_string(),
            missing_inputs: Vec::new(),
        },
        external_accounts_map: ExternalAccountsBalanceSheetMapV1 {
            current_account_pressures: "Stable".to_string(),
            financial_account_decomposition: "Portfolio funded".to_string(),
            external_debt_structure: "Mostly local currency".to_string(),
            currency_mismatch_indicators: "Contained".to_string(),
            marginal_financer: "Portfolio investors".to_string(),
            flow_reversal_vulnerability: "Portfolio outflows lead".to_string(),
            missing_inputs: Vec::new(),
        },
        policy_regime_diagnosis: PolicyRegimeDiagnosisV1 {
            monetary_policy_regime: "Inflation targeting".to_string(),
            credibility_signals: "Moderate credibility".to_string(),
            exchange_rate_regime: "Managed float".to_string(),
            intervention_pattern: "Smoothing".to_string(),
            frictions: vec![PolicyFrictionObservationV1 {
                friction: "Shallow FX market".to_string(),
                observable_indicators: vec!["volatility".to_string()],
                confidence: ConfidenceV1::Moderate,
            }],
            missing_inputs: Vec::new(),
        },
        driver_decomposition: vec![FxDriverAssessmentV1 {
            bucket: DriverBucketV1::FlowShocks,
            direction: DirectionalBiasV1::Positive,
            magnitude: SignalMagnitudeV1::Medium,
            confidence: ConfidenceV1::Moderate,
            evidence: "Flows stabilized".to_string(),
        }],
        sovereign_systemic_risk: SovereignSystemicRiskV1 {
            debt_sustainability_state: "Stable".to_string(),
            gross_financing_needs: "Manageable".to_string(),
            rollover_risk: "Contained".to_string(),
            sovereign_bank_nonbank_nexus: "Present but stable".to_string(),
            key_amplifiers: vec!["Leverage".to_string()],
            cross_border_spillovers: "Contained".to_string(),
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
        source_governance: vec![SourceGovernanceDecisionV1 {
            source_id: "source-1".to_string(),
            source_domain: "imf.org".to_string(),
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
        direct_inputs: MacroFinancialDirectInputsV1::default(),
        classification: Classification::Internal,
        constraints: SourceConstraintsV1::default(),
    };
    assert_eq!(request.objective.directive_label(), "RISK_MGMT");
}

#[test]
fn macro_financial_requests_support_direct_source_and_mixed_inputs() {
    let direct_only = MacroFinancialAnalysisRequestV1 {
        analysis_id: "analysis-direct".to_string(),
        objective: AnalysisObjectiveV1::PolicyEval,
        horizon: AnalysisHorizonV1::Nowcast,
        coverage: AnalysisCoverageV1 {
            countries: vec!["Japan".to_string()],
            regions: Vec::new(),
            currencies: vec!["JPY".to_string()],
            fx_pairs: vec!["USD/JPY".to_string()],
            asset_classes: vec!["rates".to_string()],
        },
        data_vintage: None,
        source_ids: Vec::new(),
        capsule_id: None,
        direct_inputs: MacroFinancialDirectInputsV1 {
            fx_levels_returns: vec![contracts::AnalysisSeriesInputV1 {
                series_name: "USDJPY".to_string(),
                country_area: "Japan".to_string(),
                source_label: "USER".to_string(),
                frequency: Some("Daily".to_string()),
                last_observation: Some("2026-03-09=149.20".to_string()),
                units: Some("spot".to_string()),
                transform: Some("level".to_string()),
                observations: vec![contracts::AnalysisObservationV1 {
                    timestamp: "2026-03-09".to_string(),
                    value: "149.20".to_string(),
                }],
            }],
            ..MacroFinancialDirectInputsV1::default()
        },
        classification: Classification::Internal,
        constraints: SourceConstraintsV1::default(),
    };
    let source_only = MacroFinancialAnalysisRequestV1 {
        analysis_id: "analysis-source".to_string(),
        objective: AnalysisObjectiveV1::RiskMgmt,
        horizon: AnalysisHorizonV1::OneToThreeMonths,
        coverage: direct_only.coverage.clone(),
        data_vintage: Some("2026-03-09".to_string()),
        source_ids: vec!["source-1".to_string()],
        capsule_id: None,
        direct_inputs: MacroFinancialDirectInputsV1::default(),
        classification: Classification::Internal,
        constraints: SourceConstraintsV1::default(),
    };
    let mixed = MacroFinancialAnalysisRequestV1 {
        analysis_id: "analysis-mixed".to_string(),
        objective: AnalysisObjectiveV1::InvestmentStrategy,
        horizon: AnalysisHorizonV1::ThreeToTwelveMonths,
        coverage: direct_only.coverage.clone(),
        data_vintage: Some("2026-03-09".to_string()),
        source_ids: vec!["source-1".to_string()],
        capsule_id: Some("capsule-1".to_string()),
        direct_inputs: direct_only.direct_inputs.clone(),
        classification: Classification::Internal,
        constraints: SourceConstraintsV1 {
            allowed_sources: vec!["imf".to_string()],
            forbidden_sources: Vec::new(),
            required_output_format: Some("strict".to_string()),
        },
    };

    for request in [direct_only, source_only, mixed] {
        let json = serde_json::to_value(&request).expect("serialize analysis request");
        let round_trip: MacroFinancialAnalysisRequestV1 =
            serde_json::from_value(json).expect("deserialize analysis request");
        assert_eq!(round_trip.analysis_id, request.analysis_id);
    }
}

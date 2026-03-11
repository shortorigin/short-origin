use contracts::{AgentActionRequestV1, WorkflowBoundaryV1};
use enforcement::GuardedMutationRequest;
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use events::WeatherBackfillCompletedV1;
use identity::{AggregateId, EnvironmentId, ServiceId, WorkflowId};
use meteorological_service::{
    MeteorologicalService, WeatherFixtureBatchV1, WeatherIngestionReport,
};
use orchestrator::WorkflowEngine;

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherIngestionWorkflowReport {
    pub ingestion: WeatherIngestionReport,
    pub backfill: WeatherBackfillCompletedV1,
}

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "weather_ingestion".to_owned(),
        touched_domains: vec!["meteorological_intelligence".to_owned()],
        target_services: vec![
            "meteorological-service".to_owned(),
            "approval-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}

pub async fn execute<P, A, E>(
    engine: &mut WorkflowEngine<P, A, E>,
    weather_service: &mut MeteorologicalService,
    action: &AgentActionRequestV1,
    batch: WeatherFixtureBatchV1,
) -> InstitutionalResult<WeatherIngestionWorkflowReport>
where
    P: policy_sdk::PolicyDecisionPort,
    A: policy_sdk::ApprovalVerificationPort,
    E: evidence_sdk::EvidenceSink,
{
    let guarded_request = GuardedMutationRequest {
        action_id: action.action_id.clone(),
        workflow_name: WorkflowId::from("weather_ingestion"),
        target_service: ServiceId::from("meteorological-service"),
        target_aggregate: AggregateId::from("weather_dataset"),
        actor_ref: action.actor_ref.clone(),
        impact_tier: action.impact_tier,
        classification: action.classification,
        policy_refs: action.policy_refs.clone(),
        required_approver_roles: action.required_approver_roles.clone(),
        environment: EnvironmentId::from("prod"),
        cross_domain: false,
    };

    let mut approved = None;
    engine
        .execute_mutation(guarded_request, |context| {
            approved = Some(context.clone());
            Ok(())
        })
        .await?;
    let _context = approved.ok_or_else(|| {
        InstitutionalError::invariant(
            OperationContext::new("workflows/weather_ingestion", "execute"),
            "weather ingestion authorization context missing",
        )
    })?;

    let region_id = batch.region_id.clone();
    let product_refs = batch
        .normalized_products
        .iter()
        .map(|product| product.product_id.clone())
        .collect::<Vec<_>>();
    let batch_id = batch.batch_id.clone();
    let completed_at = batch.availability.generated_at;
    let ingestion = weather_service.ingest_fixture_batch(batch)?;
    Ok(WeatherIngestionWorkflowReport {
        ingestion,
        backfill: WeatherBackfillCompletedV1 {
            run_id: batch_id,
            region_ids: vec![region_id],
            product_refs,
            completed_at,
        },
    })
}

#[cfg(test)]
mod tests {
    use approval_service::ApprovalService;
    use contracts::{AgentActionRequestV1, Classification, ImpactTier};
    use evidence_service::EvidenceService;
    use identity::ActorRef;
    use meteorological_service::WeatherFixtureBatchV1;
    use orchestrator::WorkflowEngine;
    use policy_service::PolicyService;

    use super::{execute, workflow_boundary};

    fn load_fixture() -> WeatherFixtureBatchV1 {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/fixtures/weather/run-2026-03-10/noaa_weather_batch.json"
        )))
        .expect("weather fixture json")
    }

    fn test_action() -> AgentActionRequestV1 {
        AgentActionRequestV1 {
            action_id: "action-weather-1".into(),
            actor_ref: ActorRef("weather-operator".to_string()),
            objective: "Refresh west coast weather products".to_string(),
            requested_workflow: "weather_ingestion".into(),
            impact_tier: ImpactTier::Tier0,
            classification: Classification::Internal,
            required_approver_roles: Vec::new(),
            policy_refs: vec!["ops.weather.ingestion".to_string()],
        }
    }

    #[test]
    fn workflow_boundary_declares_weather_domain() {
        let boundary = workflow_boundary();
        assert_eq!(boundary.workflow_name, "weather_ingestion");
        assert_eq!(
            boundary.touched_domains,
            vec!["meteorological_intelligence"]
        );
        assert!(boundary.mutation_path_only);
    }

    #[tokio::test]
    async fn workflow_executes_fixture_backfill() {
        let mut engine = WorkflowEngine::new(
            PolicyService::institutional_default(),
            ApprovalService::default(),
            EvidenceService::default(),
        );
        let mut service = meteorological_service::MeteorologicalService::default();

        let report = execute(&mut engine, &mut service, &test_action(), load_fixture())
            .await
            .expect("workflow");

        assert_eq!(report.ingestion.batch_id, "weather-batch-2026-03-10");
        assert_eq!(report.backfill.region_ids, vec!["us-west"]);
        assert_eq!(service.published_products().len(), 4);
    }
}

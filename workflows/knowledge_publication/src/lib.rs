use contracts::{
    AgentActionRequestV1, KnowledgePublicationRequestV1, KnowledgePublicationStatusV1,
    KnowledgeSourceIngestRequestV1, WorkflowBoundaryV1,
};
use enforcement::GuardedMutationRequest;
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use evidence_sdk::EvidenceSink;
use governed_storage::KnowledgeStore;
use identity::{AggregateId, EnvironmentId, ServiceId, WorkflowId};
use knowledge_service::KnowledgeService;
use orchestrator::WorkflowEngine;
use policy_sdk::{ApprovalVerificationPort, PolicyDecisionPort};

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "knowledge_publication".to_owned(),
        touched_domains: vec!["data_knowledge".to_owned()],
        target_services: vec![
            "knowledge-service".to_owned(),
            "approval-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}

pub async fn execute<P, A, E, M, R>(
    engine: &mut WorkflowEngine<P, A, E>,
    knowledge_service: &KnowledgeService<M>,
    repositories: &R,
    action: &AgentActionRequestV1,
    ingest_request: KnowledgeSourceIngestRequestV1,
    publication_request: KnowledgePublicationRequestV1,
) -> InstitutionalResult<KnowledgePublicationStatusV1>
where
    P: PolicyDecisionPort,
    A: ApprovalVerificationPort,
    E: EvidenceSink,
    M: memory_provider::KnowledgeMemoryProvider + Clone,
    R: KnowledgeStore,
{
    let guarded_request = GuardedMutationRequest {
        action_id: action.action_id.clone(),
        workflow_name: WorkflowId::from("knowledge_publication"),
        target_service: ServiceId::from("knowledge-service"),
        target_aggregate: AggregateId::from("knowledge_capsule"),
        actor_ref: action.actor_ref.clone(),
        impact_tier: action.impact_tier,
        classification: action.classification,
        policy_refs: action.policy_refs.clone(),
        required_approver_roles: action.required_approver_roles.clone(),
        environment: EnvironmentId::from("prod"),
        cross_domain: false,
    };

    let mut approved_context = None;
    engine
        .execute_mutation(guarded_request, |context| {
            approved_context = Some(context.clone());
            Ok(())
        })
        .await?;
    let context = approved_context.ok_or_else(|| {
        InstitutionalError::invariant(
            OperationContext::new("workflows/knowledge_publication", "execute"),
            "knowledge publication authorization context missing",
        )
    })?;

    knowledge_service
        .ingest_sources(&context, repositories, ingest_request)
        .await?;
    let capsule = knowledge_service
        .publish_capsule(&context, repositories, publication_request)
        .await?;
    Ok(KnowledgePublicationStatusV1::from_capsule(&capsule))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use approval_service::ApprovalService;
    use chrono::{TimeZone, Utc};
    use evidence_service::EvidenceService;
    use governed_storage::{KnowledgeStore, connect_in_memory};
    use memory_provider::MemvidMemoryProvider;
    use orchestrator::WorkflowEngine;
    use policy_service::PolicyService;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use trading_core::{FixedClock, SequenceIdGenerator};

    use super::{execute, workflow_boundary};

    async fn spawn_source_server() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let address = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let mut buffer = [0_u8; 2048];
                let read = stream.read(&mut buffer).await.expect("read");
                let request = String::from_utf8_lossy(&buffer[..read]);
                let body = if request.contains("/fred") {
                    r#"{"observations":[{"date":"2026-02-01","value":"1.27"}]}"#
                } else {
                    "<html><body><p>Reserve adequacy remains strong.</p></body></html>"
                };
                let content_type = if request.contains("/fred") {
                    "application/json"
                } else {
                    "text/html"
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.expect("write");
            }
        });
        address
    }

    #[tokio::test]
    async fn workflow_executes_ingest_and_publication() {
        let address = spawn_source_server().await;
        let repositories = connect_in_memory().await.expect("repo");
        let dir = tempfile::tempdir().expect("tempdir");
        let service = knowledge_service::KnowledgeService::new(
            MemvidMemoryProvider::new(dir.path()),
            Arc::new(FixedClock::new(
                Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0)
                    .single()
                    .expect("time"),
            )),
            Arc::new(SequenceIdGenerator::new("knowledge")),
        )
        .with_official_document_host("127.0.0.1");
        let mut engine = WorkflowEngine::new(
            PolicyService::institutional_default(),
            ApprovalService::default(),
            EvidenceService::default(),
        );
        let action = contracts::AgentActionRequestV1 {
            action_id: "action-1".into(),
            actor_ref: identity::ActorRef("agent:strategist".to_string()),
            objective: "Publish macro knowledge capsule".to_string(),
            requested_workflow: "knowledge_publication".into(),
            impact_tier: contracts::ImpactTier::Tier0,
            classification: contracts::Classification::Internal,
            required_approver_roles: Vec::new(),
            policy_refs: vec!["policy.data_knowledge".to_string()],
        };

        let status = execute(
            &mut engine,
            &service,
            &repositories,
            &action,
            contracts::KnowledgeSourceIngestRequestV1 {
                ingestion_id: "ingest-1".to_string(),
                classification: contracts::Classification::Internal,
                constraints: contracts::SourceConstraintsV1::default(),
                sources: vec![
                    contracts::KnowledgeSourceFetchSpecV1 {
                        source_id: "source-fred".to_string(),
                        kind: contracts::KnowledgeSourceKindV1::Fred,
                        title: "FRED FX".to_string(),
                        country_area: "United States".to_string(),
                        url: format!("http://{address}/fred"),
                        series_name: Some("DXY".to_string()),
                        expected_format: contracts::KnowledgeDocumentFormatV1::Json,
                        release_lag: Some("T+1d".to_string()),
                        units: Some("index".to_string()),
                        transform: Some("level".to_string()),
                        notes: vec!["Primary".to_string()],
                    },
                    contracts::KnowledgeSourceFetchSpecV1 {
                        source_id: "source-doc".to_string(),
                        kind: contracts::KnowledgeSourceKindV1::OfficialDocument,
                        title: "Central bank bulletin".to_string(),
                        country_area: "Japan".to_string(),
                        url: format!("http://{address}/bulletin"),
                        series_name: None,
                        expected_format: contracts::KnowledgeDocumentFormatV1::Html,
                        release_lag: None,
                        units: None,
                        transform: None,
                        notes: vec!["Official bulletin".to_string()],
                    },
                ],
            },
            contracts::KnowledgePublicationRequestV1 {
                publication_id: "publication-1".to_string(),
                capsule_id: "capsule-1".to_string(),
                title: "GMF capsule".to_string(),
                source_ids: vec!["source-fred".to_string(), "source-doc".to_string()],
                classification: contracts::Classification::Internal,
                retention_class: "institutional_record".to_string(),
                constraints: contracts::SourceConstraintsV1::default(),
            },
        )
        .await
        .expect("execute");

        assert_eq!(status.capsule_id, "capsule-1");
        assert_eq!(
            repositories
                .load_sources(&["source-fred".to_string()])
                .await
                .expect("load sources")
                .len(),
            1
        );
        assert!(
            repositories
                .load_capsule("capsule-1")
                .await
                .expect("load")
                .is_some()
        );
        assert_eq!(workflow_boundary().workflow_name, "knowledge_publication");
    }
}

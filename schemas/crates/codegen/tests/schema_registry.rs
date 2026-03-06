use codegen::{embedded_contract_schemas, embedded_event_schemas, embedded_surrealdb_schemas};

#[test]
fn schema_registry_embeds_contract_event_and_record_documents() {
    let contract_schemas = embedded_contract_schemas().unwrap();
    let event_schemas = embedded_event_schemas().unwrap();
    let surrealdb_schemas = embedded_surrealdb_schemas().unwrap();

    assert_eq!(contract_schemas.len(), 25);
    assert_eq!(event_schemas.len(), 9);
    assert_eq!(surrealdb_schemas.len(), 1);

    assert!(contract_schemas
        .iter()
        .any(|schema| schema.name == "work-item-v1"));
    assert!(contract_schemas
        .iter()
        .any(|schema| schema.name == "validation-report-v1"));
}

use ontology_model::{load_domains, load_entities, load_relationships};

#[test]
fn ontology_catalogs_are_non_empty_and_cover_required_domains() {
    let domains = load_domains().unwrap();
    let entities = load_entities().unwrap();
    let relationships = load_relationships().unwrap();

    assert_eq!(domains.version, "v1");
    assert!(domains.domains.len() >= 14);
    assert!(entities.entities.len() >= 10);
    assert!(relationships.relationships.len() >= 9);
    assert!(
        domains
            .domains
            .iter()
            .any(|domain| domain.domain == "resilience_continuity")
    );
}

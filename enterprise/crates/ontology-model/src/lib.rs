use error_model::{InstitutionalError, InstitutionalResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainEntry {
    pub domain: String,
    pub policy_owner: String,
    pub primary_service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainCatalog {
    pub version: String,
    pub domains: Vec<DomainEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityEntry {
    pub name: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityCatalog {
    pub version: String,
    pub entities: Vec<EntityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationshipEntry {
    pub from: String,
    pub to: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationshipCatalog {
    pub version: String,
    pub relationships: Vec<RelationshipEntry>,
}

const DOMAINS_JSON: &str = include_str!("../../../ontology/domains.json");
const ENTITIES_JSON: &str = include_str!("../../../ontology/entities.json");
const RELATIONSHIPS_JSON: &str = include_str!("../../../ontology/relationships.json");

pub fn load_domains() -> InstitutionalResult<DomainCatalog> {
    serde_json::from_str(DOMAINS_JSON).map_err(|error| {
        InstitutionalError::parse("enterprise/ontology/domains.json", error.to_string())
    })
}

pub fn load_entities() -> InstitutionalResult<EntityCatalog> {
    serde_json::from_str(ENTITIES_JSON).map_err(|error| {
        InstitutionalError::parse("enterprise/ontology/entities.json", error.to_string())
    })
}

pub fn load_relationships() -> InstitutionalResult<RelationshipCatalog> {
    serde_json::from_str(RELATIONSHIPS_JSON).map_err(|error| {
        InstitutionalError::parse("enterprise/ontology/relationships.json", error.to_string())
    })
}

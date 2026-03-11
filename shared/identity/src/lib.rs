use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("`{kind}` identifiers must not be empty")]
pub struct IdentifierParseError {
    kind: &'static str,
}

impl IdentifierParseError {
    fn empty(kind: &'static str) -> Self {
        Self { kind }
    }
}

macro_rules! string_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(raw: impl Into<String>) -> Self {
                Self(raw.into())
            }

            pub fn parse(raw: impl Into<String>) -> Result<Self, IdentifierParseError> {
                let raw = raw.into();
                if raw.trim().is_empty() {
                    Err(IdentifierParseError::empty(stringify!($name)))
                } else {
                    Ok(Self(raw))
                }
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Self::parse(raw).map_err(D::Error::custom)
            }
        }
    };
}

string_id_type!(ActionId);
string_id_type!(WorkflowId);
string_id_type!(ServiceId);
string_id_type!(AggregateId);
string_id_type!(EnvironmentId);
string_id_type!(DecisionId);
string_id_type!(EvidenceId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActorRef(pub String);

impl From<&str> for ActorRef {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Human,
    Workload,
    Agent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum InstitutionalRole {
    InstitutionalCouncil,
    CFO,
    CHRO,
    ChiefComplianceOfficer,
    ChiefDataOfficer,
    ChiefProcurementOfficer,
    ChiefRevenueOfficer,
    Ciso,
    Coo,
    Cto,
    DomainOwner,
    GeneralCounsel,
    HeadOfInfrastructure,
    HeadOfInternalAudit,
    HeadOfResilience,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorV1 {
    pub actor_ref: ActorRef,
    pub actor_kind: ActorKind,
    pub roles: Vec<InstitutionalRole>,
    pub active: bool,
}

impl ActorV1 {
    #[must_use]
    pub fn has_role(&self, role: InstitutionalRole) -> bool {
        self.roles.contains(&role)
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionId, IdentifierParseError, WorkflowId};

    #[test]
    fn typed_ids_reject_empty_values() {
        let error = ActionId::parse("   ").expect_err("empty ids must fail");
        assert_eq!(error, IdentifierParseError::empty("ActionId"));
    }

    #[test]
    fn typed_ids_round_trip_through_serde() {
        let workflow_id = WorkflowId::parse("knowledge_publication").expect("workflow id");
        let encoded = serde_json::to_string(&workflow_id).expect("serialize workflow id");
        assert_eq!(encoded, "\"knowledge_publication\"");
        let decoded: WorkflowId = serde_json::from_str(&encoded).expect("deserialize workflow id");
        assert_eq!(decoded, workflow_id);
    }
}

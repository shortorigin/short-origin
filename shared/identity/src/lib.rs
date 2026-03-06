use serde::{Deserialize, Serialize};

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

use contracts::{DecisionContextV1, DecisionMetadataV1};

/// Output surface for future learned representation models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepresentationOutput {
    /// Deterministic metadata describing the produced representation.
    pub metadata: DecisionMetadataV1,
}

/// Output surface for future learned forecast adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForecastAdapterOutput {
    /// Deterministic metadata describing the forecast contribution.
    pub metadata: DecisionMetadataV1,
}

/// Output surface for future learned policy-model adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyModelOutput {
    /// Deterministic metadata describing the policy-model contribution.
    pub metadata: DecisionMetadataV1,
}

/// Errors surfaced by learned-model extension points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnedAdapterError {
    /// No learned adapter is configured for the requested capability.
    NotConfigured,
}

/// Extension point for future representation learning.
pub trait RepresentationModel {
    /// Produces a deterministic representation or returns `NotConfigured`.
    fn represent(
        &self,
        context: &DecisionContextV1,
    ) -> Result<RepresentationOutput, LearnedAdapterError>;
}

/// Extension point for future learned forecast assistance.
pub trait ForecastAdapter {
    /// Produces deterministic learned forecast metadata or returns `NotConfigured`.
    fn forecast(
        &self,
        context: &DecisionContextV1,
    ) -> Result<ForecastAdapterOutput, LearnedAdapterError>;
}

/// Extension point for future learned policy-model assistance.
pub trait PolicyModelAdapter {
    /// Produces deterministic policy-model metadata or returns `NotConfigured`.
    fn evaluate_policy(
        &self,
        context: &DecisionContextV1,
    ) -> Result<PolicyModelOutput, LearnedAdapterError>;
}

/// Deterministic stub used until learned adapters are explicitly configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotConfiguredLearnedAdapter;

impl RepresentationModel for NotConfiguredLearnedAdapter {
    fn represent(
        &self,
        _context: &DecisionContextV1,
    ) -> Result<RepresentationOutput, LearnedAdapterError> {
        Err(LearnedAdapterError::NotConfigured)
    }
}

impl ForecastAdapter for NotConfiguredLearnedAdapter {
    fn forecast(
        &self,
        _context: &DecisionContextV1,
    ) -> Result<ForecastAdapterOutput, LearnedAdapterError> {
        Err(LearnedAdapterError::NotConfigured)
    }
}

impl PolicyModelAdapter for NotConfiguredLearnedAdapter {
    fn evaluate_policy(
        &self,
        _context: &DecisionContextV1,
    ) -> Result<PolicyModelOutput, LearnedAdapterError> {
        Err(LearnedAdapterError::NotConfigured)
    }
}

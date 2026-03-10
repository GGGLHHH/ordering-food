use thiserror::Error;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("duplicate context registration for `{context_id}`")]
    DuplicateContextId { context_id: &'static str },
    #[error("context `{context_id}` depends on missing context `{dependency_id}`")]
    MissingDependency {
        context_id: &'static str,
        dependency_id: &'static str,
    },
    #[error("cyclic context dependency detected: {context_ids:?}")]
    CyclicDependency { context_ids: Vec<&'static str> },
    #[error("{phase} phase failed for context `{context_id}`")]
    PhaseFailed {
        phase: &'static str,
        context_id: &'static str,
        #[source]
        source: BoxError,
    },
}

impl RegistryError {
    pub fn phase_failed(phase: &'static str, context_id: &'static str, source: BoxError) -> Self {
        Self::PhaseFailed {
            phase,
            context_id,
            source,
        }
    }
}

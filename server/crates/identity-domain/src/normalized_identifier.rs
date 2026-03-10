use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedIdentifier(String);

impl NormalizedIdentifier {
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let normalized = value.as_ref().trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(DomainError::EmptyIdentifier);
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

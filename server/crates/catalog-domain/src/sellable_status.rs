use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SellableStatus {
    Sellable,
    Unsellable,
}

impl SellableStatus {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "sellable" => Ok(Self::Sellable),
            "unsellable" => Ok(Self::Unsellable),
            other => Err(DomainError::InvalidSellableStatus(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sellable => "sellable",
            Self::Unsellable => "unsellable",
        }
    }

    pub fn is_sellable(&self) -> bool {
        matches!(self, Self::Sellable)
    }
}

#[cfg(test)]
mod tests {
    use super::SellableStatus;

    #[test]
    fn sellable_status_exposes_catalog_language() {
        assert!(SellableStatus::Sellable.is_sellable());
        assert_eq!(SellableStatus::Unsellable.as_str(), "unsellable");
    }
}

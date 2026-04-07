use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Placed,
    CancelledByCustomer,
}

impl OrderStatus {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "placed" => Ok(Self::Placed),
            "cancelled_by_customer" => Ok(Self::CancelledByCustomer),
            other => Err(DomainError::InvalidOrderStatus(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Placed => "placed",
            Self::CancelledByCustomer => "cancelled_by_customer",
        }
    }

    pub fn cancel_by_customer(self) -> Result<Self, DomainError> {
        match self {
            Self::Placed => Ok(Self::CancelledByCustomer),
            Self::CancelledByCustomer => Err(DomainError::InvalidTransition {
                event: "cancel_by_customer".to_string(),
                status: self.as_str().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_legacy_pending_acceptance_status() {
        let error = OrderStatus::parse("pending_acceptance").unwrap_err();

        assert_eq!(
            error,
            DomainError::InvalidOrderStatus("pending_acceptance".to_string())
        );
    }

    #[test]
    fn parse_supports_commercial_statuses_only() {
        assert_eq!(OrderStatus::parse("placed").unwrap(), OrderStatus::Placed);
        assert_eq!(
            OrderStatus::parse("cancelled_by_customer").unwrap(),
            OrderStatus::CancelledByCustomer
        );
    }
}

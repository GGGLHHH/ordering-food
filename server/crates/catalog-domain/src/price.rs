use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Price {
    amount: i64,
}

impl Price {
    pub fn new(amount: i64) -> Result<Self, DomainError> {
        if amount < 0 {
            return Err(DomainError::NegativePriceAmount);
        }

        Ok(Self { amount })
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::Price;
    use crate::DomainError;

    #[test]
    fn price_rejects_negative_amount() {
        let error = Price::new(-1).unwrap_err();

        assert_eq!(error, DomainError::NegativePriceAmount);
    }
}

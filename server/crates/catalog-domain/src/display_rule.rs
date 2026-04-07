#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayRule {
    Listed,
    Hidden,
}

impl DisplayRule {
    pub fn listed() -> Self {
        Self::Listed
    }

    pub fn hidden() -> Self {
        Self::Hidden
    }

    pub fn is_listed(&self) -> bool {
        matches!(self, Self::Listed)
    }

    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::Hidden)
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayRule;

    #[test]
    fn listed_and_hidden_helpers_encode_display_semantics() {
        assert!(DisplayRule::listed().is_listed());
        assert!(DisplayRule::hidden().is_hidden());
    }
}

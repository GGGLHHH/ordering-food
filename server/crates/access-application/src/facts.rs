#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSubjectStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessSubjectFacts {
    subject_id: String,
    status: AccessSubjectStatus,
}

impl AccessSubjectFacts {
    pub fn new(subject_id: impl Into<String>, status: AccessSubjectStatus) -> Self {
        Self {
            subject_id: subject_id.into(),
            status,
        }
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    pub fn status(&self) -> AccessSubjectStatus {
        self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessStoreScopeFacts {
    store_id: String,
    brand_id: String,
}

impl AccessStoreScopeFacts {
    pub fn new(store_id: impl Into<String>, brand_id: impl Into<String>) -> Self {
        Self {
            store_id: store_id.into(),
            brand_id: brand_id.into(),
        }
    }

    pub fn store_id(&self) -> &str {
        &self.store_id
    }

    pub fn brand_id(&self) -> &str {
        &self.brand_id
    }
}

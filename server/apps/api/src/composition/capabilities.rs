use std::{any::Any, collections::HashMap, sync::RwLock};

pub const ACCESS_ORDER_MANAGEMENT_GATEWAY: &str = "access.order_management_gateway";
pub const IDENTITY_ACCESS_TOKEN_VERIFIER: &str = "identity.access_token_verifier";
pub const IDENTITY_SUBJECT_LOOKUP_GATEWAY: &str = "identity.subject_lookup_gateway";
pub const ORGANIZATION_BRAND_LOOKUP_GATEWAY: &str = "organization.brand_lookup_gateway";
pub const ORGANIZATION_STORE_SCOPE_GATEWAY: &str = "organization.store_scope_gateway";

#[derive(Default)]
pub struct ApiCapabilityRegistry {
    entries: RwLock<HashMap<&'static str, Box<dyn Any + Send + Sync>>>,
}

impl ApiCapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish<T>(&self, key: &'static str, value: T)
    where
        T: Any + Send + Sync + 'static,
    {
        self.entries.write().unwrap().insert(key, Box::new(value));
    }

    pub fn resolve<T>(&self, key: &'static str) -> Option<T>
    where
        T: Any + Clone + Send + Sync + 'static,
    {
        self.entries
            .read()
            .unwrap()
            .get(key)
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::ApiCapabilityRegistry;

    #[test]
    fn resolves_published_capability_by_type() {
        let registry = ApiCapabilityRegistry::new();
        registry.publish("test.capability", String::from("ok"));

        let resolved = registry.resolve::<String>("test.capability");

        assert_eq!(resolved.as_deref(), Some("ok"));
    }

    #[test]
    fn returns_none_for_type_mismatch() {
        let registry = ApiCapabilityRegistry::new();
        registry.publish("test.capability", String::from("ok"));

        let resolved = registry.resolve::<u64>("test.capability");

        assert!(resolved.is_none());
    }
}

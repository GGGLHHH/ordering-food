use crate::composition::{contribution::ApiContextContribution, platform::ApiPlatform};
use ordering_food_bootstrap_core::{
    BootstrapRegistration, ContextDescriptor, MigrationRegistration,
};

pub struct ApiContextRegistration {
    descriptor: ContextDescriptor,
    migration_factory: fn(ContextDescriptor) -> MigrationRegistration<ApiPlatform>,
    bootstrap_factory:
        fn(ContextDescriptor) -> BootstrapRegistration<ApiPlatform, ApiContextContribution>,
}

impl ApiContextRegistration {
    pub fn new(
        descriptor: ContextDescriptor,
        migration_factory: fn(ContextDescriptor) -> MigrationRegistration<ApiPlatform>,
        bootstrap_factory: fn(
            ContextDescriptor,
        )
            -> BootstrapRegistration<ApiPlatform, ApiContextContribution>,
    ) -> Self {
        Self {
            descriptor,
            migration_factory,
            bootstrap_factory,
        }
    }

    pub fn descriptor(&self) -> ContextDescriptor {
        self.descriptor
    }

    pub fn migration_registration(&self) -> MigrationRegistration<ApiPlatform> {
        (self.migration_factory)(self.descriptor)
    }

    pub fn bootstrap_registration(
        &self,
    ) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
        (self.bootstrap_factory)(self.descriptor)
    }
}

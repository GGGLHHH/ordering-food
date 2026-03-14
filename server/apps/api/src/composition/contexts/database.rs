use crate::composition::{
    context_registration::ApiContextRegistration, contribution::ApiContextContribution,
    platform::ApiPlatform,
};
use ordering_food_bootstrap_core::{
    BootstrapRegistration, ContextDescriptor, MigrationRegistration,
};
use ordering_food_database_infrastructure_sqlx::MIGRATOR;

pub fn register_database() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "database",
        depends_on: &[],
    };

    ApiContextRegistration::new(
        descriptor,
        database_migration_registration,
        database_bootstrap_registration,
    )
}

fn database_migration_registration(
    descriptor: ContextDescriptor,
) -> MigrationRegistration<ApiPlatform> {
    MigrationRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let auto_migrate = platform.settings.app.auto_migrate;
        let pg_pool = platform.pg_pool.clone();
        async move {
            if auto_migrate {
                MIGRATOR.run(&pg_pool).await?;
            }

            Ok::<_, sqlx::migrate::MigrateError>(())
        }
    })
}

fn database_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |_platform: &ApiPlatform| {
        let context_id = descriptor.id;
        async move { Ok::<_, std::io::Error>(ApiContextContribution::empty(context_id)) }
    })
}

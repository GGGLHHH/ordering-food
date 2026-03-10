use crate::composition::{
    context_registration::ApiContextRegistration,
    contribution::{ApiContextContribution, ApiNamedReadinessCheck},
    platform::ApiPlatform,
};
use ordering_food_bootstrap_core::{
    BootstrapRegistration, ContextDescriptor, MigrationRegistration,
};
use ordering_food_identity_infrastructure_sqlx::{MIGRATOR, build_identity_module};

pub fn register_identity() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "identity",
        depends_on: &[],
    };

    ApiContextRegistration::new(
        descriptor,
        identity_migration_registration,
        identity_bootstrap_registration,
    )
}

fn identity_migration_registration(
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

fn identity_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = platform.clock.clone();
        let id_generator = platform.id_generator.clone();
        async move {
            let module = build_identity_module(pg_pool, clock, id_generator);
            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.retain_private(module);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}

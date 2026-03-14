use anyhow::{Context, Result, anyhow, bail, ensure};
use ordering_food_database_infrastructure_sqlx::MIGRATOR;
use sqlx::{PgPool, Row};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppliedMigrationInfo {
    version: i64,
    checksum: Vec<u8>,
    success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigrationStatusLine {
    version: i64,
    description: String,
    status: &'static str,
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow!("DATABASE_URL must be set before running migration-info"))?;
    ensure!(
        !database_url.trim().is_empty(),
        "DATABASE_URL must not be empty"
    );

    let pool = PgPool::connect(&database_url)
        .await
        .context("failed to connect to postgres for migration-info")?;

    let applied_migrations = load_applied_migrations(&pool).await?;
    let status_lines = build_status_lines(&applied_migrations)?;

    for line in status_lines {
        println!("{}/{} {}", line.version, line.status, line.description);
    }

    Ok(())
}

async fn load_applied_migrations(pool: &PgPool) -> Result<Vec<AppliedMigrationInfo>> {
    let migrations_table_exists = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT to_regclass('_sqlx_migrations')::text
        "#,
    )
    .fetch_one(pool)
    .await
    .context("failed to check _sqlx_migrations existence")?
    .is_some();

    if !migrations_table_exists {
        return Ok(Vec::new());
    }

    Ok(sqlx::query(
        r#"
        SELECT version, checksum, success
        FROM _sqlx_migrations
        ORDER BY version ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to query _sqlx_migrations")?
    .into_iter()
    .map(|row| AppliedMigrationInfo {
        version: row.get("version"),
        checksum: row.get::<Vec<u8>, _>("checksum"),
        success: row.get("success"),
    })
    .collect::<Vec<_>>())
}

fn build_status_lines(
    applied_migrations: &[AppliedMigrationInfo],
) -> Result<Vec<MigrationStatusLine>> {
    if let Some(dirty) = applied_migrations
        .iter()
        .find(|migration| !migration.success)
    {
        bail!(
            "migration {} is partially applied; fix and remove row from `_sqlx_migrations` table",
            dirty.version
        );
    }

    let resolved_migrations = MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| {
            (
                migration.version,
                (
                    migration.description.to_string(),
                    migration.checksum.as_ref().to_vec(),
                ),
            )
        })
        .collect::<HashMap<_, _>>();

    for applied in applied_migrations {
        let Some((_, checksum)) = resolved_migrations.get(&applied.version) else {
            bail!(
                "migration {} was previously applied but is missing in the resolved migrations",
                applied.version
            );
        };

        if checksum != &applied.checksum {
            bail!(
                "migration {} was previously applied but has been modified",
                applied.version
            );
        }
    }

    let applied_versions = applied_migrations
        .iter()
        .map(|migration| migration.version)
        .collect::<BTreeSet<_>>();

    Ok(MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| MigrationStatusLine {
            version: migration.version,
            description: migration.description.to_string(),
            status: if applied_versions.contains(&migration.version) {
                "installed"
            } else {
                "pending"
            },
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checksum(seed: u8) -> Vec<u8> {
        vec![seed; 4]
    }

    #[test]
    fn build_status_lines_marks_unapplied_migrations_as_pending() {
        let lines = build_status_lines(&[]).unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].version, 202603140001);
        assert_eq!(lines[0].status, "pending");
        assert_eq!(lines[0].description, "baseline");
        assert_eq!(lines[1].version, 202603150001);
        assert_eq!(lines[1].status, "pending");
        assert_eq!(lines[1].description, "ordering");
    }

    #[test]
    fn build_status_lines_rejects_dirty_migrations() {
        let error = build_status_lines(&[AppliedMigrationInfo {
            version: 202603140001,
            checksum: MIGRATOR.iter().next().unwrap().checksum.as_ref().to_vec(),
            success: false,
        }])
        .unwrap_err();

        assert!(error.to_string().contains("is partially applied"));
    }

    #[test]
    fn build_status_lines_rejects_missing_versions() {
        let error = build_status_lines(&[AppliedMigrationInfo {
            version: 202603140099,
            checksum: checksum(1),
            success: true,
        }])
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("is missing in the resolved migrations")
        );
    }

    #[test]
    fn build_status_lines_rejects_checksum_mismatch() {
        let error = build_status_lines(&[AppliedMigrationInfo {
            version: 202603140001,
            checksum: checksum(9),
            success: true,
        }])
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("was previously applied but has been modified")
        );
    }
}

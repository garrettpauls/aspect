extern crate migrant_lib;

use std::path::Path;

use migrant_lib::{Config, EmbeddedMigration, Migrator, Settings};
use crate::support::ErrToString;

pub fn migrate(db_file: &Path) -> Result<(), String> {
    log::info!("Migrating database: {}", db_file.display());

    let settings = Settings::configure_sqlite()
        .database_path(db_file).err_to_string()?
        .build().err_to_string()?;

    let mut config = Config::with_settings(&settings);

    log::info!("Setting up migrations...");
    config.setup().err_to_string()?;

    log::info!("Loading migrations...");
    config.use_migrations(&[
        EmbeddedMigration::with_tag("20190112025101_create-file-table")
            .up(include_str!("20190112025101_create-file-table.sql"))
            .boxed(),
    ]).err_to_string()?;

    let config = config.reload().err_to_string()?;

    log::info!("Applying migrations...");
    Migrator::with_config(&config)
        .all(true)
        .show_output(false)
        .swallow_completion(true)
        .apply().err_to_string()?;

    log::info!("Migrations applied successfully.");
    Ok(())
}

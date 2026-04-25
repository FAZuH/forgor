use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::embed_migrations;

pub mod model;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

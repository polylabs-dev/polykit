//! PolyKit ESLite — schema migrations, query engine, sync
//!
//! All ESLite operations run in WASM. The TypeScript layer never reads,
//! writes, or queries ESLite directly — it receives render-ready data
//! from WASM.

pub mod migrations;
pub mod schema;
pub mod query;
pub mod sync;

pub use migrations::{Migration, MigrationRunner};
pub use schema::{TableDef, ColumnDef, ColumnType};
pub use query::QueryResult;

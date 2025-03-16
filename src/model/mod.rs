//! Model Laye
//! 
//! Design: 
//! 
//! - The model layer normalizes the apllication's data type
//!   structures and access
//! - All application code data access must go through the Model layer.
//! - The `ModelManager` holds the internal states/resources
//!   needed by ModelControllers to access the data.
//!   (e.g., dp_pool, S3 client, redis client).
//! - Model Controllers (e.g., `TaskBmc`, `ProjectBmc`) implement 
//!   CRUD and other data access methods on a given "Entity"
//!   (e.q., `Task`, `Project`).
//!   (`Bmc` is short for Backend Model Controller).
//! - In frameworks like axum, Tauri, `ModelManager` are typically used as App State.
//! - ModelManager are designed to be passed as an argument
//!   to all Model Controller functions.

// region:    --- Modules

mod error;
mod store;
pub mod task;

use store::{new_db_pool, Db};

pub use self::error::{Error, Result};

// endregion: --- Modules

#[derive(Clone)]
pub struct ModelManager {
	db: Db,
}

impl ModelManager {
	pub async fn new() -> Result<Self> {
		let db = new_db_pool().await?;
		// FIXME - TBC
		Ok(ModelManager {
			db
		})
	}
	pub(in crate::model) fn db(&self) -> &Db {
		&self.db
	}
}

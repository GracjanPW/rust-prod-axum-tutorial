use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::path::PathBuf;
use std::{fs, time::Duration};
use tracing::info;

use crate::ctx::Ctx;
use crate::model::user::{User, UserBmc};
use crate::model::ModelManager;

type Db = Pool<Postgres>;

// NOTE: Hardcode to prevent deployed system update.
const PG_DEV_POSTGRES_URL: &str =
	"postgres://postgres:localpassword@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// sql files
const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

const DEMO_PWD: &str = "passwordlogin";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
	info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

	// -- Create the app_db/app_user with postgres user.
	{
		let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
		pexec(&root_db, SQL_RECREATE_DB).await?;
	}

	// -- Get sql files.
	let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.collect();
	paths.sort();

	// -- SQL Ececute each file.
	let app_db = new_db_pool(PG_DEV_APP_URL).await?;
	for path in paths {
		if let Some(path) = path.to_str() {
			let path = path.replace('\\', "/"); // for windows.

			// Only take the .sql files and skip the SQL_RECREATE_DB file.
			if path.ends_with(".sql") && path != SQL_RECREATE_DB {
				pexec(&app_db, &path).await?;
			}
		}
	}

	// -- Init model layer.
	let mm = ModelManager::new().await?;
	let ctx = Ctx::root_ctx();

	// -- Set demo1 pwd
	let demo1_user: User = UserBmc::first_by_username(&ctx, &mm, "demo1")
		.await?
		.unwrap();
	UserBmc::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD).await?;
	info!("{:<12} - init_dev_db - set demo1 pwd", "FOR-DEV-ONLY");

	Ok(())
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
	info!("{:<12} - pexec: {file}", "FOR-DEV-ONLY");

	// -- Read the file.
	let content = fs::read_to_string(file)?;

	// FIXME: Make the split more sql proof.
	let sqls: Vec<&str> = content.split(";").collect();

	for sql in sqls {
		sqlx::query(sql).execute(db).await?;
	}

	Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
	PgPoolOptions::new()
		.max_connections(1)
		.acquire_timeout(Duration::from_millis(500))
		.connect(db_con_url)
		.await
}

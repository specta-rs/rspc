use std::{sync::Arc, fs::{create_dir_all, File}, path::PathBuf};
use crate::prisma::{self, PrismaClient};

pub async fn load_and_migrate(db_url: PathBuf) -> Arc<PrismaClient> {
    let db_dir = db_url.parent().unwrap();
    if !db_dir.exists() {
        create_dir_all(db_dir).unwrap()
    }

    if !db_url.exists() {
        File::create(db_url.clone()).unwrap();
    }

    let db_url = format!("file:{}", db_url.to_str().unwrap());

    let client = prisma::new_client_with_url(&db_url).await.unwrap();

    #[cfg(debug_assertions)]
    client._db_push().await.unwrap();

    #[cfg(not(debug_assertions))]
    client._migrate_deploy().await.unwrap();

    Arc::new(client)
}

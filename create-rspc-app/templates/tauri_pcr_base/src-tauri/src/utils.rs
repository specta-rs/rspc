use std::path::PathBuf;

pub async fn load_and_migrate(db_url: PathBuf) -> prisma::PrismaClient {
    let db_dir = db_url.parent().unwrap();
    if !db_dir.exists() {
        std::fs::create_dir_all(db_dir).unwrap();
    }

    if !db_url.exists() {
        std::fs::File::create(db_url.clone()).unwrap();
    }

    let db_url = format!("file:{}", db_url.to_str().unwrap());

    let client = prisma::new_client_with_url(&db_url).await.unwrap();

    #[cfg(debug_assertions)]
    client._db_push().await.unwrap();

    #[cfg(not(debug_assertions))]
    client._migrate_deploy().await.unwrap();

    client
}

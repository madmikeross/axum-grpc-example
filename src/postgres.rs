use std::{env, fs};

fn get_connection_string() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER not set");
    let password_file = env::var("POSTGRES_PASSWORD_FILE").expect("POSTGRES_PASSWORD_FILE not set");
    let password =
        fs::read_to_string(password_file).unwrap_or_else(|_| panic!("Could not read password"));
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST not set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT not set");

    format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db_name
    )
}

pub async fn connection_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let mut retries = 5;
    let delay_seconds = 5;

    while retries > 0 {
        match sqlx::PgPool::connect(&get_connection_string()).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                eprintln!("Database connection failed: {}. Retrying...", e);
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
            }
        }
    }

    Err(sqlx::Error::Configuration(
        "Failed to connect to database".into(),
    ))
}

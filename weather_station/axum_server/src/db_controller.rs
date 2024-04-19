use sqlx::{mysql::MySqlPool, Pool, Row};

// Define a struct to represent your database configuration
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    // Constructor to create a new instance of Database
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Create a connection pool
        let pool = MySqlPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    // Function to perform a database query and return the result
    pub async fn get_user_count(&self) -> Result<i64, sqlx::Error> {
        // Perform a query to get the count of users
        let count = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
    // Other database-related functions can be defined here
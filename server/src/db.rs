use r2d2_sqlite::SqliteConnectionManager;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;

pub fn get_db_pool() -> Pool {
    let manager = SqliteConnectionManager::file("db.db");
    r2d2::Pool::new(manager).unwrap()
}
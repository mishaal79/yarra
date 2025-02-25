use duckdb::{params, Connection, Result};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Analytics {
    conn: Mutex<Connection>,
}

impl Analytics {
    pub fn new() -> Result<Self> {
        let db_path = match Self::db_path() {
            Ok(path) => path,
            Err(e) => return Err(duckdb::Error::InvalidPath(PathBuf::from(e.to_string()))),
        };
        
        if let Err(e) = std::fs::create_dir_all(db_path.parent().unwrap()) {
            return Err(duckdb::Error::InvalidPath(PathBuf::from(e.to_string())));
        }

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                start_time VARCHAR NOT NULL,
                duration INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS blocked_attempts (
                id INTEGER PRIMARY KEY,
                timestamp VARCHAR NOT NULL,
                site TEXT NOT NULL
            );
            "#
        )?;

        Ok(Analytics { conn: Mutex::new(conn) })
    }

    fn db_path() -> Result<PathBuf, std::io::Error> {
        dirs::data_dir()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to find data directory"
            ))
            .map(|p| p.join("yarra/analytics.ddb"))
    }

    pub fn log_session(&self, start_time: DateTime<Utc>, duration: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (start_time, duration) VALUES (?, ?)",
            params![start_time.to_rfc3339(), duration],
        )?;
        Ok(())
    }

    pub fn log_blocked_attempt(&self, site: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO blocked_attempts (timestamp, site) VALUES (?, ?)",
            params![Utc::now().to_rfc3339(), site],
        )?;
        Ok(())
    }

    pub fn total_focus_time(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COALESCE(SUM(duration), 0) FROM sessions",
            [],
            |row| row.get(0),
        )
    }

    pub fn todays_blocked_attempts(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM blocked_attempts WHERE DATE(timestamp) = DATE('now')",
            [],
            |row| row.get(0),
        )
    }
}

impl Drop for Analytics {
    fn drop(&mut self) {
        if let Ok(mut conn) = self.conn.lock() {
            let conn = std::mem::replace(&mut *conn, Connection::open_in_memory().unwrap());
            let _ = conn.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_analytics() -> Result<()> {
        let analytics = Analytics::new()?;
        let start_time = Utc.with_ymd_and_hms(2023, 10, 1, 9, 0, 0).unwrap();

        analytics.log_session(start_time, 25)?;
        assert_eq!(analytics.total_focus_time()?, 25);

        analytics.log_blocked_attempt("youtube.com")?;
        assert_eq!(analytics.todays_blocked_attempts()?, 1);

        Ok(())
    }
}
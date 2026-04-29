// db_hosxp.rs — HOSxP MySQL connection + query execution via sqlx

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Column, MySql, Pool, Row, TypeInfo};

use std::sync::Mutex;

// ── Forbidden SQL keywords ───────────────────────────────────────────────
static FORBIDDEN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let keywords = [
        r"\bINSERT\b",
        r"\bUPDATE\b",
        r"\bDELETE\b",
        r"\bDROP\b",
        r"\bTRUNCATE\b",
        r"\bALTER\b",
        r"\bCREATE\b",
        r"\bREPLACE\b",
        r"\bMERGE\b",
        r"\bCALL\b",
        r"\bEXEC\b",
        r"\bEXECUTE\b",
        r"\bGRANT\b",
        r"\bREVOKE\b",
        r"\bLOCK\b",
    ];
    keywords.iter().map(|p| Regex::new(p).unwrap()).collect()
});

// ── Global pool ──────────────────────────────────────────────────────────
static POOL: Lazy<Mutex<Option<Pool<MySql>>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 3306,
            user: String::new(),
            password: String::new(),
            database: "hosxp_pcu".into(),
        }
    }
}

/// Validate that SQL is SELECT-only
pub fn validate_sql(query: &str) -> Result<(), String> {
    if query.trim().is_empty() {
        return Err("คำสั่ง SQL ว่างเปล่า".into());
    }

    // Strip comments
    let re_single = Regex::new(r"--[^\n]*").unwrap();
    let re_multi = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let cleaned = re_single.replace_all(query, " ");
    let cleaned = re_multi.replace_all(&cleaned, " ");
    let upper = cleaned.to_uppercase();

    for pat in FORBIDDEN_PATTERNS.iter() {
        if pat.is_match(&upper) {
            let kw = pat.as_str().replace(r"\b", "");
            return Err(format!("ไม่อนุญาต: พบคำสั่ง {} ซึ่งเป็นคำสั่งแก้ไขข้อมูล", kw));
        }
    }

    let re_select = Regex::new(r"\bSELECT\b").unwrap();
    if !re_select.is_match(&upper) {
        return Err("คำสั่ง SQL ต้องเริ่มต้นด้วย SELECT เท่านั้น".into());
    }

    Ok(())
}

/// Inject date range into BETWEEN clauses and replace {{date_from}}/{{date_to}} placeholders
pub fn inject_dates(query: &str, date_from: &str, date_to: &str) -> String {
    // First replace {{date_from}} / {{date_to}} template placeholders
    let query = query
        .replace("{{date_from}}", date_from)
        .replace("{{date_to}}", date_to);

    // Then replace any remaining hardcoded BETWEEN 'YYYY-MM-DD' AND 'YYYY-MM-DD' patterns
    let re = Regex::new(r"(?i)(BETWEEN\s*)'(\d{4}-\d{2}-\d{2})'(\s*AND\s*)'(\d{4}-\d{2}-\d{2})'")
        .unwrap();
    re.replace_all(&query, |caps: &regex::Captures| {
        format!("{}'{}'{}'{}'", &caps[1], date_from, &caps[3], date_to)
    })
    .to_string()
}

/// Connect and store pool globally
pub async fn connect(config: &DbConfig) -> Result<String, String> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.user, config.password, config.host, config.port, config.database
    );
    let pool: sqlx::Pool<sqlx::MySql> = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&url)
        .await
        .map_err(|e| format!("เชื่อมต่อล้มเหลว: {}", e))?;

    // Test connection and get version
    let row = sqlx::query("SELECT VERSION() AS ver")
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let version: String = row.try_get("ver").unwrap_or_else(|_| "unknown".into());

    let mut guard = POOL.lock().unwrap();
    *guard = Some(pool);
    Ok(format!("เชื่อมต่อสำเร็จ (MariaDB/MySQL {})", version))
}

/// Test connection without storing pool
pub async fn test_connection(config: &DbConfig) -> Result<String, String> {
    connect(config).await
}

/// Execute a SELECT query and return (columns, rows)
pub async fn execute_query(
    sql: &str,
    date_from: &str,
    date_to: &str,
) -> Result<QueryResult, String> {
    let sql = inject_dates(sql, date_from, date_to);
    validate_sql(&sql)?;

    let pool = {
        let guard = POOL.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| "ยังไม่ได้เชื่อมต่อฐานข้อมูล".to_string())?
            .clone()
    };

    let start = std::time::Instant::now();
    let rows = sqlx::query(&sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let elapsed = start.elapsed().as_secs_f64();

    if rows.is_empty() {
        return Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            elapsed_sec: elapsed,
            row_count: 0,
        });
    }

    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    let result_rows: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            row.columns()
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let type_name = col.type_info().name().to_uppercase();
                    value_from_row(row, i, &type_name)
                })
                .collect()
        })
        .collect();

    Ok(QueryResult {
        row_count: result_rows.len(),
        columns,
        rows: result_rows,
        elapsed_sec: elapsed,
    })
}

fn value_from_row(row: &sqlx::mysql::MySqlRow, idx: usize, type_name: &str) -> serde_json::Value {
    // Handle date/time types first (return ISO-like strings)
    if type_name.contains("DATE")
        || type_name.contains("TIME")
        || type_name.contains("TIMESTAMP")
        || type_name.contains("DATETIME")
    {
        // Try MySQL DATETIME/TIMESTAMP -> chrono::NaiveDateTime
        if let Ok(Some(v)) = row.try_get::<Option<chrono::NaiveDateTime>, _>(idx) {
            return serde_json::Value::String(v.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        // Try MySQL DATE -> chrono::NaiveDate
        if let Ok(Some(v)) = row.try_get::<Option<chrono::NaiveDate>, _>(idx) {
            return serde_json::Value::String(v.format("%Y-%m-%d").to_string());
        }
        // Fallback: try as string if the driver returned a textual representation
        if let Ok(Some(s)) = row.try_get::<Option<String>, _>(idx) {
            return serde_json::Value::String(s);
        }
        // If explicitly NULL
        if let Ok(None) = row.try_get::<Option<String>, _>(idx) {
            return serde_json::Value::Null;
        }
    }

    // Try booleans (some MySQL schemas use TINYINT(1) or BOOL)
    if type_name.contains("BOOL") || type_name.contains("TINYINT") {
        if let Ok(Some(b)) = row.try_get::<Option<bool>, _>(idx) {
            return serde_json::json!(b);
        }
        // If stored as numeric 0/1 -> map to bool where appropriate, otherwise return numeric
        if let Ok(Some(n)) = row.try_get::<Option<i64>, _>(idx) {
            if n == 0 {
                return serde_json::json!(false);
            } else if n == 1 {
                return serde_json::json!(true);
            } else {
                return serde_json::json!(n);
            }
        }
    }

    // Integer types
    if type_name.contains("INT") || type_name.contains("BIGINT") || type_name.contains("SMALLINT") {
        if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(idx) {
            return serde_json::json!(v);
        }
        if let Ok(Some(v)) = row.try_get::<Option<u64>, _>(idx) {
            return serde_json::json!(v);
        }
    }

    // Floating point / decimal types
    if type_name.contains("DECIMAL")
        || type_name.contains("FLOAT")
        || type_name.contains("DOUBLE")
        || type_name.contains("NUMERIC")
    {
        // Try native f64 first (works for FLOAT / DOUBLE columns)
        if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(idx) {
            return serde_json::json!(v);
        }
        // DECIMAL / NEWDECIMAL — sqlx 0.8 without bigdecimal feature cannot decode these
        // directly as f64 or String.  Use rust_decimal which IS wired up via the
        // "rust_decimal" sqlx feature added in Cargo.toml.
        if let Ok(Some(d)) = row.try_get::<Option<rust_decimal::Decimal>, _>(idx) {
            let f: f64 = d.to_string().parse().unwrap_or(0.0);
            return serde_json::json!(f);
        }
        if let Ok(None) = row.try_get::<Option<rust_decimal::Decimal>, _>(idx) {
            return serde_json::Value::Null;
        }
        // Last fallback: try as string (older driver versions / MariaDB may return text)
        if let Ok(Some(s)) = row.try_get::<Option<String>, _>(idx) {
            if let Ok(n) = s.parse::<f64>() {
                return serde_json::json!(n);
            } else {
                return serde_json::Value::String(s);
            }
        }
    }

    // Generic string / text
    if let Ok(Some(s)) = row.try_get::<Option<String>, _>(idx) {
        return serde_json::Value::String(s);
    }
    // Explicit NULL
    if let Ok(None) = row.try_get::<Option<String>, _>(idx) {
        return serde_json::Value::Null;
    }

    // Last-resort: try numeric as float
    if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(idx) {
        return serde_json::json!(v);
    }

    serde_json::Value::Null
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub elapsed_sec: f64,
    pub row_count: usize,
}

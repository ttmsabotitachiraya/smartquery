// db_local.rs — SQLite local storage (queries, departments, settings, history)

use once_cell::sync::Lazy;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

fn db_path() -> std::path::PathBuf {
    let mut p = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push("SmartQuery");
    std::fs::create_dir_all(&p).ok();
    p.push("sql_storage.db");
    p
}

static CONN: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let path = db_path();
    let conn = Connection::open(&path).expect("Cannot open SQLite DB");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .ok();
    init_db(&conn).expect("Cannot init DB");
    Mutex::new(conn)
});

pub fn get_conn() -> std::sync::MutexGuard<'static, Connection> {
    CONN.lock().unwrap()
}

fn init_db(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(DDL)?;
    migrate(conn)?;
    seed_if_empty(conn)?;
    Ok(())
}

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS sql_queries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    mode        TEXT    NOT NULL CHECK(mode IN ('audit', 'report')),
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    sql_text    TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    enabled     INTEGER NOT NULL DEFAULT 1,
    is_starred  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE TABLE IF NOT EXISTS sql_groups (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE TABLE IF NOT EXISTS sql_query_groups (
    query_id   INTEGER NOT NULL REFERENCES sql_queries(id) ON DELETE CASCADE,
    group_id   INTEGER NOT NULL REFERENCES sql_groups(id)  ON DELETE CASCADE,
    PRIMARY KEY (query_id, group_id)
);
CREATE TABLE IF NOT EXISTS execution_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    query_id     INTEGER,
    query_name   TEXT    NOT NULL,
    mode         TEXT    NOT NULL,
    date_from    TEXT    NOT NULL DEFAULT '',
    date_to      TEXT    NOT NULL DEFAULT '',
    row_count    INTEGER NOT NULL DEFAULT 0,
    elapsed_sec  REAL    NOT NULL DEFAULT 0,
    status       TEXT    NOT NULL DEFAULT 'ok',
    error_msg    TEXT    NOT NULL DEFAULT '',
    executed_at  TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS departments (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);
"#;

fn get_column_names(conn: &Connection, table: &str) -> Vec<String> {
    let sql = format!("PRAGMA table_info({})", table);
    let mut result = Vec::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(1)) {
            for row in rows.flatten() {
                result.push(row);
            }
        }
    }
    result
}

fn migrate(conn: &Connection) -> SqlResult<()> {
    // Ensure is_starred column
    let cols = get_column_names(conn, "sql_queries");
    if !cols.iter().any(|c| c == "is_starred") {
        conn.execute_batch(
            "ALTER TABLE sql_queries ADD COLUMN is_starred INTEGER NOT NULL DEFAULT 0",
        )?;
    }
    // execution_history columns
    let hist_cols = get_column_names(conn, "execution_history");
    if !hist_cols.iter().any(|c| c == "date_from") {
        conn.execute_batch(
            r#"
            ALTER TABLE execution_history RENAME TO execution_history_old;
            CREATE TABLE execution_history (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                query_id     INTEGER,
                query_name   TEXT NOT NULL,
                mode         TEXT NOT NULL,
                date_from    TEXT NOT NULL DEFAULT '',
                date_to      TEXT NOT NULL DEFAULT '',
                row_count    INTEGER NOT NULL DEFAULT 0,
                elapsed_sec  REAL NOT NULL DEFAULT 0,
                status       TEXT NOT NULL DEFAULT 'ok',
                error_msg    TEXT NOT NULL DEFAULT '',
                executed_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
        "#,
        )?;
    }
    // Add department_id to sql_queries if not present
    let query_cols = get_column_names(conn, "sql_queries");
    if !query_cols.iter().any(|c| c == "department_id") {
        conn.execute_batch("ALTER TABLE sql_queries ADD COLUMN department_id INTEGER")?;
    }
    Ok(())
}

fn seed_if_empty(_conn: &Connection) -> SqlResult<()> {
    Ok(())
}

// ── TYPES ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlQuery {
    pub id: i64,
    pub mode: String,
    pub name: String,
    pub description: String,
    pub sql_text: String,
    pub sort_order: i64,
    pub enabled: i64,
    pub is_starred: i64,
    pub created_at: String,
    pub updated_at: String,
    pub department_id: Option<i64>,
    pub department_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Department {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
}

/// Kept for backward compatibility with import_from_legacy_db
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlGroup {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionHistory {
    pub id: i64,
    pub query_id: Option<i64>,
    pub query_name: String,
    pub mode: String,
    pub date_from: String,
    pub date_to: String,
    pub row_count: i64,
    pub elapsed_sec: f64,
    pub status: String,
    pub error_msg: String,
    pub executed_at: String,
}

// ── DEPARTMENTS ───────────────────────────────────────────────────────────

pub fn get_all_departments() -> Vec<Department> {
    let conn = get_conn();
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order FROM departments ORDER BY sort_order ASC, id ASC")
        .unwrap();
    stmt.query_map([], |r| {
        Ok(Department {
            id: r.get(0)?,
            name: r.get(1)?,
            sort_order: r.get(2)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

pub fn insert_department(name: &str) -> i64 {
    let conn = get_conn();
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM departments",
            [],
            |r| r.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "INSERT INTO departments (name, sort_order) VALUES (?1, ?2)",
        params![name.trim(), max_order + 1],
    )
    .ok();
    conn.last_insert_rowid()
}

pub fn update_department(dept_id: i64, name: &str) -> bool {
    let conn = get_conn();
    conn.execute(
        "UPDATE departments SET name = ?1 WHERE id = ?2",
        params![name.trim(), dept_id],
    )
    .map(|n| n > 0)
    .unwrap_or(false)
}

pub fn delete_department(dept_id: i64) -> bool {
    let conn = get_conn();
    // Unassign queries from this department first
    conn.execute(
        "UPDATE sql_queries SET department_id = NULL WHERE department_id = ?1",
        params![dept_id],
    )
    .ok();
    conn.execute("DELETE FROM departments WHERE id = ?1", params![dept_id])
        .map(|n| n > 0)
        .unwrap_or(false)
}

// ── QUERIES CRUD ─────────────────────────────────────────────────────────

fn map_query_row(r: &rusqlite::Row) -> rusqlite::Result<SqlQuery> {
    Ok(SqlQuery {
        id: r.get(0)?,
        mode: r.get(1)?,
        name: r.get(2)?,
        description: r.get(3)?,
        sql_text: r.get(4)?,
        sort_order: r.get(5)?,
        enabled: r.get(6)?,
        is_starred: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
        department_id: r.get(10)?,
        department_name: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
    })
}

pub fn get_all(mode: &str) -> Vec<SqlQuery> {
    let conn = get_conn();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.mode, q.name, q.description, q.sql_text, q.sort_order, q.enabled, \
         q.is_starred, q.created_at, q.updated_at, q.department_id, \
         COALESCE(d.name, '') as department_name \
         FROM sql_queries q \
         LEFT JOIN departments d ON q.department_id = d.id \
         WHERE q.mode = ?1 \
         ORDER BY q.sort_order ASC, q.id ASC",
        )
        .unwrap();
    stmt.query_map(params![mode], |r| map_query_row(r))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

#[allow(dead_code)]
pub fn get_enabled(mode: &str) -> Vec<SqlQuery> {
    let conn = get_conn();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.mode, q.name, q.description, q.sql_text, q.sort_order, q.enabled, \
         q.is_starred, q.created_at, q.updated_at, q.department_id, \
         COALESCE(d.name, '') as department_name \
         FROM sql_queries q \
         LEFT JOIN departments d ON q.department_id = d.id \
         WHERE q.mode = ?1 AND q.enabled = 1 \
         ORDER BY q.sort_order ASC, q.id ASC",
        )
        .unwrap();
    stmt.query_map(params![mode], |r| map_query_row(r))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

/// Get enabled queries filtered by department_id.
/// - department_id = 0  → return ALL enabled queries (including those with NULL department)
/// - department_id > 0  → return only queries whose department_id matches
pub fn get_enabled_by_dept(mode: &str, department_id: i64) -> Vec<SqlQuery> {
    let conn = get_conn();
    if department_id <= 0 {
        // All departments: return everything (NULL dept included)
        let mut stmt = conn
            .prepare(
                "SELECT q.id, q.mode, q.name, q.description, q.sql_text, q.sort_order, q.enabled, \
             q.is_starred, q.created_at, q.updated_at, q.department_id, \
             COALESCE(d.name, '') as department_name \
             FROM sql_queries q \
             LEFT JOIN departments d ON q.department_id = d.id \
             WHERE q.mode = ?1 AND q.enabled = 1 \
             ORDER BY q.sort_order ASC, q.id ASC",
            )
            .unwrap();
        stmt.query_map(params![mode], |r| map_query_row(r))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    } else {
        // Specific department
        let mut stmt = conn
            .prepare(
                "SELECT q.id, q.mode, q.name, q.description, q.sql_text, q.sort_order, q.enabled, \
             q.is_starred, q.created_at, q.updated_at, q.department_id, \
             COALESCE(d.name, '') as department_name \
             FROM sql_queries q \
             LEFT JOIN departments d ON q.department_id = d.id \
             WHERE q.mode = ?1 AND q.enabled = 1 AND q.department_id = ?2 \
             ORDER BY q.sort_order ASC, q.id ASC",
            )
            .unwrap();
        stmt.query_map(params![mode, department_id], |r| map_query_row(r))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }
}

pub fn get_by_id(query_id: i64) -> Option<SqlQuery> {
    let conn = get_conn();
    conn.query_row(
        "SELECT q.id, q.mode, q.name, q.description, q.sql_text, q.sort_order, q.enabled, \
         q.is_starred, q.created_at, q.updated_at, q.department_id, \
         COALESCE(d.name, '') as department_name \
         FROM sql_queries q \
         LEFT JOIN departments d ON q.department_id = d.id \
         WHERE q.id = ?1",
        params![query_id],
        |r| map_query_row(r),
    )
    .ok()
}

pub fn insert_query(
    mode: &str,
    name: &str,
    description: &str,
    sql_text: &str,
    department_id: Option<i64>,
    is_starred: bool,
) -> i64 {
    let conn = get_conn();
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM sql_queries WHERE mode = ?1",
            params![mode],
            |r| r.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "INSERT INTO sql_queries \
         (mode, name, description, sql_text, sort_order, enabled, is_starred, department_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
        params![
            mode,
            name.trim(),
            description.trim(),
            sql_text.trim(),
            max_order + 1,
            if is_starred { 1 } else { 0 },
            department_id,
        ],
    )
    .ok();
    conn.last_insert_rowid()
}

pub fn update_query(
    query_id: i64,
    name: &str,
    description: &str,
    sql_text: &str,
    enabled: bool,
    department_id: Option<i64>,
    is_starred: Option<bool>,
) -> bool {
    let conn = get_conn();
    let rows = if let Some(starred) = is_starred {
        conn.execute(
            "UPDATE sql_queries \
             SET name=?1, description=?2, sql_text=?3, enabled=?4, is_starred=?5, \
             department_id=?6, updated_at=datetime('now','localtime') \
             WHERE id=?7",
            params![
                name.trim(),
                description.trim(),
                sql_text.trim(),
                if enabled { 1 } else { 0 },
                if starred { 1 } else { 0 },
                department_id,
                query_id,
            ],
        )
        .unwrap_or(0)
    } else {
        conn.execute(
            "UPDATE sql_queries \
             SET name=?1, description=?2, sql_text=?3, enabled=?4, \
             department_id=?5, updated_at=datetime('now','localtime') \
             WHERE id=?6",
            params![
                name.trim(),
                description.trim(),
                sql_text.trim(),
                if enabled { 1 } else { 0 },
                department_id,
                query_id,
            ],
        )
        .unwrap_or(0)
    };
    rows > 0
}

pub fn delete_query(query_id: i64) -> bool {
    let conn = get_conn();
    conn.execute("DELETE FROM sql_queries WHERE id = ?1", params![query_id])
        .map(|n| n > 0)
        .unwrap_or(false)
}

pub fn set_enabled(query_id: i64, enabled: bool) {
    let conn = get_conn();
    conn.execute(
        "UPDATE sql_queries SET enabled = ?1 WHERE id = ?2",
        params![if enabled { 1 } else { 0 }, query_id],
    )
    .ok();
}

pub fn set_starred(query_id: i64, starred: bool) {
    let conn = get_conn();
    conn.execute(
        "UPDATE sql_queries SET is_starred = ?1 WHERE id = ?2",
        params![if starred { 1 } else { 0 }, query_id],
    )
    .ok();
}

pub fn search_queries(
    mode: &str,
    keyword: &str,
    status_filter: &str,
    department_id: i64,
    starred_only: bool,
) -> Vec<SqlQuery> {
    let conn = get_conn();
    let kw = format!("%{}%", keyword);

    // Build WHERE conditions and a parallel Vec of owned values
    let mut conditions: Vec<&str> = vec!["q.mode = ?"];
    let mut values: Vec<String> = vec![mode.to_string()];

    if !keyword.is_empty() {
        conditions.push("(q.name LIKE ? OR q.description LIKE ?)");
        values.push(kw.clone());
        values.push(kw.clone());
    }
    if status_filter == "enabled" {
        conditions.push("q.enabled = 1");
    } else if status_filter == "disabled" {
        conditions.push("q.enabled = 0");
    }
    if starred_only {
        conditions.push("q.is_starred = 1");
    }
    if department_id > 0 {
        conditions.push("q.department_id = ?");
        values.push(department_id.to_string());
    }

    let where_clause = conditions.join(" AND ");

    let sql = format!(
        "SELECT q.id, q.mode, q.name, q.description, q.sql_text, \
         q.sort_order, q.enabled, q.is_starred, q.created_at, q.updated_at, \
         q.department_id, COALESCE(d.name, '') as department_name \
         FROM sql_queries q \
         LEFT JOIN departments d ON q.department_id = d.id \
         WHERE {} \
         ORDER BY q.sort_order ASC, q.id ASC",
        where_clause
    );

    let params: Vec<&dyn rusqlite::ToSql> =
        values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let result: Vec<SqlQuery> = match stmt.query_map(params.as_slice(), |r| map_query_row(r)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    };
    result
}

pub fn count_queries(mode: &str) -> i64 {
    let conn = get_conn();
    conn.query_row(
        "SELECT COUNT(*) FROM sql_queries WHERE mode = ?1",
        params![mode],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

#[allow(dead_code)]
pub fn count_enabled_queries(mode: &str) -> i64 {
    let conn = get_conn();
    conn.query_row(
        "SELECT COUNT(*) FROM sql_queries WHERE mode = ?1 AND enabled = 1",
        params![mode],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

// ── EXECUTION HISTORY ─────────────────────────────────────────────────────

pub fn log_execution(
    query_id: Option<i64>,
    query_name: &str,
    mode: &str,
    date_from: &str,
    date_to: &str,
    row_count: i64,
    elapsed_sec: f64,
    status: &str,
    error_msg: &str,
) {
    let conn = get_conn();
    conn.execute(
        "INSERT INTO execution_history \
         (query_id, query_name, mode, date_from, date_to, row_count, elapsed_sec, status, error_msg) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            query_id, query_name, mode, date_from, date_to,
            row_count, elapsed_sec, status, error_msg,
        ],
    )
    .ok();
}

pub fn get_execution_history(
    limit: i64,
    mode_filter: &str,
    status_filter: &str,
) -> Vec<ExecutionHistory> {
    let conn = get_conn();
    let mut conditions: Vec<String> = vec![];
    if mode_filter != "all" {
        conditions.push(format!("mode = '{}'", mode_filter));
    }
    if status_filter != "all" {
        conditions.push(format!("status = '{}'", status_filter));
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT id, query_id, query_name, mode, date_from, date_to, row_count, elapsed_sec, \
         status, error_msg, executed_at \
         FROM execution_history {} ORDER BY id DESC LIMIT ?1",
        where_clause
    );
    let mut stmt = conn.prepare(&sql).unwrap();
    stmt.query_map(params![limit], |r| {
        Ok(ExecutionHistory {
            id: r.get(0)?,
            query_id: r.get(1)?,
            query_name: r.get(2)?,
            mode: r.get(3)?,
            date_from: r.get(4)?,
            date_to: r.get(5)?,
            row_count: r.get(6)?,
            elapsed_sec: r.get(7)?,
            status: r.get(8)?,
            error_msg: r.get(9)?,
            executed_at: r.get(10)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

pub fn clear_execution_history() {
    let conn = get_conn();
    conn.execute("DELETE FROM execution_history", []).ok();
}

// ── SETTINGS ──────────────────────────────────────────────────────────────

pub fn get_setting(key: &str, default: &str) -> String {
    let conn = get_conn();
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |r| r.get(0),
    )
    .unwrap_or_else(|_| default.to_string())
}

pub fn set_setting(key: &str, value: &str) {
    let conn = get_conn();
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .ok();
}

pub fn save_db_config(host: &str, port: u16, user: &str, password: &str, database: &str) {
    set_setting("db_host", host);
    set_setting("db_port", &port.to_string());
    set_setting("db_user", user);
    set_setting("db_password", password);
    set_setting("db_database", database);
}

pub fn load_db_config() -> super::db_hosxp::DbConfig {
    super::db_hosxp::DbConfig {
        host: get_setting("db_host", "localhost"),
        port: get_setting("db_port", "3306").parse().unwrap_or(3306),
        user: get_setting("db_user", ""),
        password: get_setting("db_password", ""),
        database: get_setting("db_database", "hosxp_pcu"),
    }
}

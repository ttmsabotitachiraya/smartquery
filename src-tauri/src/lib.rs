// lib.rs — Tauri application entry point

mod db_hosxp;
mod db_local;

use db_hosxp::{DbConfig, QueryResult};
use db_local::{Department, ExecutionHistory, SqlQuery};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

pub mod commands {
    use super::*;

    // ── DB CONFIG ─────────────────────────────────────────────────────────

    #[tauri::command]
    pub async fn load_db_config() -> DbConfig {
        db_local::load_db_config()
    }

    #[tauri::command]
    pub async fn save_db_config(
        host: String,
        port: u16,
        user: String,
        password: String,
        database: String,
    ) {
        db_local::save_db_config(&host, port, &user, &password, &database);
    }

    #[tauri::command]
    pub async fn test_connection(config: DbConfig) -> Result<String, String> {
        db_hosxp::test_connection(&config).await
    }

    #[tauri::command]
    pub async fn connect_db(config: DbConfig) -> Result<String, String> {
        db_hosxp::connect(&config).await
    }

    // ── QUERY EXECUTION ──────────────────────────────────────────────────

    #[tauri::command]
    pub async fn execute_query(
        sql: String,
        date_from: String,
        date_to: String,
    ) -> Result<QueryResult, String> {
        db_hosxp::execute_query(&sql, &date_from, &date_to).await
    }

    #[tauri::command]
    pub async fn validate_sql(sql: String) -> Result<String, String> {
        db_hosxp::validate_sql(&sql).map(|_| "ผ่านการตรวจสอบ".to_string())
    }

    // ── DEPARTMENTS ──────────────────────────────────────────────────────

    #[tauri::command]
    pub fn get_all_departments() -> Vec<Department> {
        db_local::get_all_departments()
    }

    #[tauri::command]
    pub fn insert_department(name: String) -> i64 {
        db_local::insert_department(&name)
    }

    #[tauri::command]
    pub fn update_department(dept_id: i64, name: String) -> bool {
        db_local::update_department(dept_id, &name)
    }

    #[tauri::command]
    pub fn delete_department(dept_id: i64) -> bool {
        db_local::delete_department(dept_id)
    }

    // ── QUERIES CRUD ─────────────────────────────────────────────────────

    #[tauri::command]
    pub fn get_all_queries(mode: String) -> Vec<SqlQuery> {
        db_local::get_all(&mode)
    }

    #[tauri::command]
    pub fn get_enabled_queries(mode: String, department_id: Option<i64>) -> Vec<SqlQuery> {
        db_local::get_enabled_by_dept(&mode, department_id.unwrap_or(0))
    }

    #[tauri::command]
    pub fn get_query_by_id(query_id: i64) -> Option<SqlQuery> {
        db_local::get_by_id(query_id)
    }

    #[tauri::command]
    pub fn insert_query(
        mode: String,
        name: String,
        description: String,
        sql_text: String,
        department_id: Option<i64>,
        is_starred: bool,
    ) -> i64 {
        db_local::insert_query(
            &mode,
            &name,
            &description,
            &sql_text,
            department_id,
            is_starred,
        )
    }

    #[tauri::command]
    pub fn update_query(
        query_id: i64,
        name: String,
        description: String,
        sql_text: String,
        enabled: bool,
        department_id: Option<i64>,
        is_starred: Option<bool>,
    ) -> bool {
        db_local::update_query(
            query_id,
            &name,
            &description,
            &sql_text,
            enabled,
            department_id,
            is_starred,
        )
    }

    #[tauri::command]
    pub fn delete_query(query_id: i64) -> bool {
        db_local::delete_query(query_id)
    }

    #[tauri::command]
    pub fn set_query_enabled(query_id: i64, enabled: bool) {
        db_local::set_enabled(query_id, enabled);
    }

    #[tauri::command]
    pub fn set_query_starred(query_id: i64, starred: bool) {
        db_local::set_starred(query_id, starred);
    }

    #[tauri::command]
    pub fn search_queries(
        mode: String,
        keyword: String,
        status_filter: String,
        department_id: i64,
        starred_only: bool,
    ) -> Vec<SqlQuery> {
        db_local::search_queries(&mode, &keyword, &status_filter, department_id, starred_only)
    }

    #[tauri::command]
    pub fn count_queries(mode: String) -> i64 {
        db_local::count_queries(&mode)
    }

    // ── EXECUTION HISTORY ─────────────────────────────────────────────────

    #[derive(Serialize, Deserialize)]
    pub struct LogExecutionArgs {
        pub query_id: Option<i64>,
        pub query_name: String,
        pub mode: String,
        pub date_from: String,
        pub date_to: String,
        pub row_count: i64,
        pub elapsed_sec: f64,
        pub status: String,
        pub error_msg: String,
    }

    #[tauri::command]
    pub fn log_execution(args: LogExecutionArgs) {
        db_local::log_execution(
            args.query_id,
            &args.query_name,
            &args.mode,
            &args.date_from,
            &args.date_to,
            args.row_count,
            args.elapsed_sec,
            &args.status,
            &args.error_msg,
        );
    }

    #[tauri::command]
    pub fn get_execution_history(
        limit: i64,
        mode_filter: String,
        status_filter: String,
    ) -> Vec<ExecutionHistory> {
        db_local::get_execution_history(limit, &mode_filter, &status_filter)
    }

    #[tauri::command]
    pub fn clear_execution_history() {
        db_local::clear_execution_history();
    }

    // ── IMPORT FROM LEGACY DB ─────────────────────────────────────────────

    /// Import queries and settings from a legacy Python app SQLite database.
    /// Returns a summary string of what was imported.
    #[tauri::command]
    pub fn import_from_legacy_db(db_path: String) -> Result<String, String> {
        let src =
            Connection::open(&db_path).map_err(|e| format!("Cannot open legacy DB: {}", e))?;

        let dst_conn = db_local::get_conn();

        // ── Import queries ─────────────────────────────────────────────────
        let mut queries_imported = 0i64;
        {
            let mut stmt = src
                .prepare(
                    "SELECT id, mode, name, description, sql_text, sort_order, enabled, \
                     is_starred FROM sql_queries ORDER BY sort_order ASC",
                )
                .map_err(|e| e.to_string())?;

            #[derive(Debug)]
            struct LegacyQuery {
                mode: String,
                name: String,
                description: String,
                sql_text: String,
                sort_order: i64,
                enabled: i64,
                is_starred: i64,
            }

            let legacy_queries: Vec<LegacyQuery> = stmt
                .query_map([], |r| {
                    Ok(LegacyQuery {
                        mode: r.get(1)?,
                        name: r.get(2)?,
                        description: r.get(3)?,
                        sql_text: r.get(4)?,
                        sort_order: r.get(5)?,
                        enabled: r.get(6)?,
                        is_starred: r.get(7)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();

            for lq in &legacy_queries {
                let exists: i64 = dst_conn
                    .query_row(
                        "SELECT COUNT(*) FROM sql_queries WHERE name = ?1 AND mode = ?2",
                        params![lq.name, lq.mode],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if exists == 0 {
                    dst_conn
                        .execute(
                            "INSERT INTO sql_queries \
                             (mode, name, description, sql_text, sort_order, enabled, is_starred) \
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            params![
                                lq.mode,
                                lq.name,
                                lq.description,
                                lq.sql_text,
                                lq.sort_order,
                                lq.enabled,
                                lq.is_starred,
                            ],
                        )
                        .ok();
                    queries_imported += 1;
                }
            }
        }

        // ── Import settings (db config) ────────────────────────────────────
        let settings_keys = [
            "db_host",
            "db_port",
            "db_user",
            "db_password",
            "db_database",
        ];
        let mut settings_imported = 0i64;
        for key in &settings_keys {
            let val: Option<String> = src
                .query_row(
                    "SELECT value FROM app_settings WHERE key = ?1",
                    params![key],
                    |r| r.get(0),
                )
                .ok();
            if let Some(v) = val {
                let already: String = dst_conn
                    .query_row(
                        "SELECT COALESCE(value,'') FROM app_settings WHERE key = ?1",
                        params![key],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                if already.is_empty() {
                    dst_conn
                        .execute(
                            "INSERT INTO app_settings (key, value) VALUES (?1, ?2) \
                             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                            params![key, v],
                        )
                        .ok();
                    settings_imported += 1;
                }
            }
        }

        Ok(format!(
            "นำเข้าสำเร็จ: {} query, {} การตั้งค่า",
            queries_imported, settings_imported
        ))
    }
} // end pub mod commands

// ── APP SETUP ─────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::import_from_legacy_db,
            commands::load_db_config,
            commands::save_db_config,
            commands::test_connection,
            commands::connect_db,
            commands::execute_query,
            commands::validate_sql,
            commands::get_all_departments,
            commands::insert_department,
            commands::update_department,
            commands::delete_department,
            commands::get_all_queries,
            commands::get_enabled_queries,
            commands::get_query_by_id,
            commands::insert_query,
            commands::update_query,
            commands::delete_query,
            commands::set_query_enabled,
            commands::set_query_starred,
            commands::search_queries,
            commands::count_queries,
            commands::log_execution,
            commands::get_execution_history,
            commands::clear_execution_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

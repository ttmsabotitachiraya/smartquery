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

fn seed_if_empty(conn: &Connection) -> SqlResult<()> {
    // ── 1. Seed default departments ──────────────────────────────────────
    let dept_count: i64 = conn.query_row("SELECT COUNT(*) FROM departments", [], |r| r.get(0))?;
    if dept_count == 0 {
        let departments = ["TTM (แพทย์แผนไทย)", "OPD", "IPD", "ER", "Lab"];
        for (i, d) in departments.iter().enumerate() {
            conn.execute(
                "INSERT INTO departments (name, sort_order) VALUES (?1, ?2)",
                params![d, i],
            )?;
        }
    }

    // ── 2. Seed default SQL queries ──────────────────────────────────────
    let qry_count: i64 = conn.query_row("SELECT COUNT(*) FROM sql_queries", [], |r| r.get(0))?;
    if qry_count == 0 {
        struct QuerySeed {
            mode: &'static str,
            name: &'static str,
            description: &'static str,
            sql_text: &'static str,
        }

        let queries: &[QuerySeed] = &[
            // ── Report 1 ──────────────────────────────────────────────────
            QuerySeed {
                mode: "report",
                name: "ยาสมุนไพร 32 รายการ — รายได้เรียกเก็บ UC (รวม)",
                description: "รายได้เรียกเก็บยาสมุนไพร 32 รายการ สิทธิ์ UC รวมต่อ visit",
                sql_text: r#"SELECT
  o.vstdate AS visit_date,
  p.cid AS cid,
  CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
  o.hn AS hn,
  GROUP_CONCAT(di.name SEPARATOR ', ') AS drug_items_list,
  SUM(oi.sum_price) AS total_drug_price,
  (
    MAX(CASE WHEN oi.icode = '1660007' THEN 40 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1580016' THEN 73 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1620004' THEN 100 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1580019' THEN 60 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1670027' THEN 37 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1580003' THEN 84 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1610009' THEN 84 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1500018' THEN 56 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1530080' THEN 204 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1580023' THEN 31.5 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1610025' THEN 70 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1650006' THEN 420 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1550013' THEN 46 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1600027' THEN 0 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1580004' THEN 92 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1660013' THEN 50 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1590008' THEN 84 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1530061' THEN 33 ELSE 0 END) +
    MAX(CASE WHEN oi.icode = '1650044' THEN 420 ELSE 0 END)
  ) AS nhso_price
FROM ovst o
INNER JOIN patient p ON o.hn = p.hn
INNER JOIN opitemrece oi ON o.vn = oi.vn
INNER JOIN drugitems di ON oi.icode = di.icode
INNER JOIN pttype pt ON o.pttype = pt.pttype
WHERE
  o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
  AND di.icode IN ('1660007','1580016','1620004','1580019','1670027',
      '1580003','1610009','1500018','1530080','1580023','1610025','1650006',
      '1550013','1600027','1580004','1660013','1590008','1530061','1650044')
  AND pt.hipdata_code IN ('WEL','UCS')
GROUP BY o.vstdate, p.cid, o.hn, p.pname, p.fname, p.lname
ORDER BY o.vstdate, p.fname, p.lname"#,
            },
            // ── Report 2 ──────────────────────────────────────────────────
            QuerySeed {
                mode: "report",
                name: "ยาสมุนไพร 32 รายการ — รายได้เรียกเก็บ UC (แยกรายการ)",
                description: "รายได้เรียกเก็บยาสมุนไพร 32 รายการ สิทธิ์ UC แยกรายการยา",
                sql_text: r#"SELECT
  o.vstdate AS visit_date,
  p.cid AS cid,
  CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
  o.hn AS hn,
  di.name AS drug_item,
  oi.icode,
  oi.sum_price AS drug_price,
  CASE oi.icode
    WHEN '1660007' THEN 40
    WHEN '1580016' THEN 73
    WHEN '1620004' THEN 100
    WHEN '1580019' THEN 60
    WHEN '1670027' THEN 37
    WHEN '1580003' THEN 84
    WHEN '1610009' THEN 84
    WHEN '1500018' THEN 56
    WHEN '1530080' THEN 204
    WHEN '1580023' THEN 31.5
    WHEN '1610025' THEN 70
    WHEN '1650006' THEN 420
    WHEN '1550013' THEN 46
    WHEN '1600027' THEN 0
    WHEN '1580004' THEN 92
    WHEN '1660013' THEN 50
    WHEN '1590008' THEN 84
    WHEN '1530061' THEN 33
    WHEN '1650044' THEN 420
    ELSE 0
  END AS nhso_price
FROM ovst o
INNER JOIN patient p ON o.hn = p.hn
INNER JOIN opitemrece oi ON o.vn = oi.vn
INNER JOIN drugitems di ON oi.icode = di.icode
INNER JOIN pttype pt ON o.pttype = pt.pttype
WHERE
  o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
  AND di.icode IN ('1660007','1580016','1620004','1580019','1670027',
      '1580003','1610009','1500018','1530080','1580023','1610025','1650006',
      '1550013','1600027','1580004','1660013','1590008','1530061','1650044')
  AND pt.hipdata_code IN ('WEL','UCS')
  AND oi.sum_price <> 0
ORDER BY o.vstdate, p.fname, p.lname, di.name"#,
            },
            // ── Report 3 ──────────────────────────────────────────────────
            QuerySeed {
                mode: "report",
                name: "รายงานการคำนวณ Point — แยกหัตถการ UC",
                description: "คำนวณ Point หัตถการแพทย์แผนไทย สิทธิ์ UC แยกรายหัตถการ",
                sql_text: r#"SELECT
  V.service_date AS service_date,
  V.cid AS cid,
  V.patient_name AS patient_name,
  V.main_dep AS main_dep,
  V.hn,
  V.vn,
  R.procedure_name AS procedure_name,
  R.points AS points
FROM (
  SELECT
    o.vstdate AS service_date,
    o.hn, o.vn, p.cid,
    CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
    pt.name AS main_dep,
    GROUP_CONCAT(DISTINCT oi.icode ORDER BY oi.icode SEPARATOR ',') AS icodes
  FROM ovst AS o
  INNER JOIN patient AS p ON o.hn = p.hn
  LEFT JOIN pttype AS pt ON o.pttype = pt.pttype
  INNER JOIN opitemrece AS oi ON o.vn = oi.vn
  WHERE
    o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
    AND pt.hipdata_code IN ('UCS', 'WEL')
    AND oi.icode IN ('3003013','3003014','3003012','3003016','3003021','3003038','3002724','3003888','3002725')
  GROUP BY o.vstdate, o.hn, o.vn, p.cid, patient_name, main_dep
) AS V
INNER JOIN (
  SELECT 'บริการฟื้นฟูสมรรถภาพมารดาหลังคลอด' AS procedure_name, 500 AS points, 1 AS rule_id UNION ALL
  SELECT 'บริการนวดและประคบ', 250, 2 UNION ALL
  SELECT 'บริการนวด', 200, 3 UNION ALL
  SELECT 'บริการประคบ', 150, 4 UNION ALL
  SELECT 'บริการพอกเข่า', 100, 5 UNION ALL
  SELECT 'บริการอบสมุนไพร', 120, 6
) AS R
ON (R.rule_id = 1 AND V.icodes LIKE '%3003013%' AND V.icodes LIKE '%3003014%' AND V.icodes LIKE '%3003012%' AND V.icodes LIKE '%3003016%' AND V.icodes LIKE '%3003021%')
OR (R.rule_id = 2 AND V.icodes LIKE '%3003038%' AND V.icodes LIKE '%3002724%')
OR (R.rule_id = 3 AND V.icodes LIKE '%3003038%' AND V.icodes NOT LIKE '%3002724%')
OR (R.rule_id = 4 AND V.icodes LIKE '%3002724%' AND V.icodes NOT LIKE '%3003038%')
OR (R.rule_id = 5 AND V.icodes LIKE '%3003888%')
OR (R.rule_id = 6 AND V.icodes LIKE '%3002725%')
ORDER BY V.service_date, V.hn, V.vn"#,
            },
            // ── Report 4 ──────────────────────────────────────────────────
            QuerySeed {
                mode: "report",
                name: "รายงานการคำนวณ Point — รวมหัตถการ UC",
                description: "คำนวณ Point หัตถการแพทย์แผนไทย สิทธิ์ UC รวม Point ต่อ visit",
                sql_text: r#"SELECT
  T.service_date AS visit_date,
  T.cid AS cid_number,
  T.patient_name AS full_name,
  T.main_dep AS pttype_name,
  T.hn, T.vn,
  GROUP_CONCAT(T.procedure_name ORDER BY T.rule_id SEPARATOR ', ') AS procedure_list,
  SUM(T.points) AS total_points
FROM (
  SELECT
    V.service_date, V.cid, V.patient_name, V.main_dep, V.hn, V.vn,
    R.procedure_name, R.points, R.rule_id
  FROM (
    SELECT
      o.vstdate AS service_date, o.hn, o.vn, p.cid,
      CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
      pt.name AS main_dep,
      GROUP_CONCAT(DISTINCT oi.icode ORDER BY oi.icode SEPARATOR ',') AS icodes
    FROM ovst AS o
    INNER JOIN patient AS p ON o.hn = p.hn
    LEFT JOIN pttype AS pt ON o.pttype = pt.pttype
    INNER JOIN opitemrece AS oi ON o.vn = oi.vn
    WHERE
      o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
      AND pt.hipdata_code IN ('UCS', 'WEL')
      AND oi.icode IN ('3003013','3003014','3003012','3003016','3003021','3003038','3002724','3003888','3002725')
    GROUP BY o.vstdate, o.hn, o.vn, p.cid, patient_name, main_dep
  ) AS V
  INNER JOIN (
    SELECT 'บริการฟื้นฟูสมรรถภาพมารดาหลังคลอด' AS procedure_name, 500 AS points, 1 AS rule_id UNION ALL
    SELECT 'บริการนวดและประคบ', 250, 2 UNION ALL
    SELECT 'บริการนวด', 200, 3 UNION ALL
    SELECT 'บริการประคบ', 150, 4 UNION ALL
    SELECT 'บริการพอกเข่า', 100, 5 UNION ALL
    SELECT 'บริการอบสมุนไพร', 120, 6
  ) AS R
  ON (R.rule_id = 1 AND V.icodes LIKE '%3003013%' AND V.icodes LIKE '%3003014%' AND V.icodes LIKE '%3003012%' AND V.icodes LIKE '%3003016%' AND V.icodes LIKE '%3003021%')
  OR (R.rule_id = 2 AND V.icodes LIKE '%3003038%' AND V.icodes LIKE '%3002724%')
  OR (R.rule_id = 3 AND V.icodes LIKE '%3003038%' AND V.icodes NOT LIKE '%3002724%')
  OR (R.rule_id = 4 AND V.icodes LIKE '%3002724%' AND V.icodes NOT LIKE '%3003038%')
  OR (R.rule_id = 5 AND V.icodes LIKE '%3003888%')
  OR (R.rule_id = 6 AND V.icodes LIKE '%3002725%')
) AS T
GROUP BY T.service_date, T.cid, T.patient_name, T.main_dep, T.hn, T.vn
ORDER BY T.service_date, T.hn, T.vn"#,
            },
            // ── Report 5 ──────────────────────────────────────────────────
            QuerySeed {
                mode: "report",
                name: "รายงานทุกบริการ — สิทธิ์ ขรก",
                description: "รายงานบริการแพทย์แผนไทยทั้งหมด สำหรับสิทธิ์ข้าราชการและ อปท.",
                sql_text: r#"SELECT DISTINCT
  o.vstdate,
  p.cid,
  CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
  pt.name AS pttype_name,
  o.hn,
  filtered_items.item_list,
  filtered_items.total_price
FROM ovst o
INNER JOIN (
  SELECT
    oi.hn, oi.vstdate,
    SUM(oi.sum_price) AS total_price,
    GROUP_CONCAT(DISTINCT COALESCE(d.name, n.name) SEPARATOR ', ') AS item_list
  FROM opitemrece oi
  LEFT JOIN drugitems d ON oi.icode = d.icode
  LEFT JOIN nondrugitems n ON oi.icode = n.icode
  WHERE oi.icode IN (
    '3002725','3003038','3002724','3003309','3003308','3003310',
    '3003311','3003312','3003313','3003307','3003888','1660007',
    '1580016','1620004','1580019','1670027','1580003','1610009',
    '1500018','1530080','1580023','1610025','1650006','1550013',
    '1600027','1580004','1660013','1590008','1530061','1610007'
  )
  GROUP BY oi.hn, oi.vstdate
) AS filtered_items ON o.hn = filtered_items.hn AND o.vstdate = filtered_items.vstdate
INNER JOIN health_med_service hms ON o.hn = hms.hn AND o.vstdate = hms.service_date
INNER JOIN patient p ON o.hn = p.hn
INNER JOIN pttype pt ON o.pttype = pt.pttype
WHERE
  o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
  AND pt.hipdata_code IN ('OFC', 'LGO')
ORDER BY o.vstdate, o.hn"#,
            },
            // ── Report 6 ──────────────────────────────────────────────────
            QuerySeed {
                mode: "report",
                name: "รายงานการคำนวณ Point — นวด/ประคบ/อบ/พอกเข่า/หลังคลอด UC",
                description: "คำนวณ Point หัตถการนวด ประคบ อบสมุนไพร พอกเข่า และหลังคลอด สิทธิ์ UC จาก health_med_service",
                sql_text: r#"SELECT
  T.service_date AS วันที่,
  T.cid AS เลขประชาชน,
  T.patient_name AS ชื่อสกุล,
  T.main_dep AS สิทธิ์การรักษา,
  T.hn,
  T.vn,
  CONCAT_WS(', ',
    CASE WHEN T.icodes LIKE '%3003013%' AND T.icodes LIKE '%3003014%' AND T.icodes LIKE '%3003012%' AND T.icodes LIKE '%3003016%' AND T.icodes LIKE '%3003021%'
      THEN 'บริการฟื้นฟูสมรรถภาพมารดาหลังคลอด' ELSE NULL END,
    CASE WHEN T.icodes LIKE '%3003038%' AND T.icodes LIKE '%3002724%' THEN 'บริการนวดและประคบ' ELSE NULL END,
    CASE WHEN T.icodes LIKE '%3003038%' AND T.icodes NOT LIKE '%3002724%' THEN 'บริการนวด' ELSE NULL END,
    CASE WHEN T.icodes LIKE '%3002724%' AND T.icodes NOT LIKE '%3003038%' THEN 'บริการประคบ' ELSE NULL END,
    CASE WHEN T.icodes LIKE '%3003888%' THEN 'บริการพอกเข่า' ELSE NULL END,
    CASE WHEN T.icodes LIKE '%3002725%' THEN 'บริการอบสมุนไพร' ELSE NULL END
  ) AS รายการหัตถการ,
  (
    CASE WHEN T.icodes LIKE '%3003013%' AND T.icodes LIKE '%3003014%' AND T.icodes LIKE '%3003012%' AND T.icodes LIKE '%3003016%' AND T.icodes LIKE '%3003021%' THEN 500 ELSE 0 END
    + CASE WHEN T.icodes LIKE '%3003038%' AND T.icodes LIKE '%3002724%' THEN 250 ELSE 0 END
    + CASE WHEN T.icodes LIKE '%3003038%' AND T.icodes NOT LIKE '%3002724%' THEN 200 ELSE 0 END
    + CASE WHEN T.icodes LIKE '%3002724%' AND T.icodes NOT LIKE '%3003038%' THEN 150 ELSE 0 END
    + CASE WHEN T.icodes LIKE '%3003888%' THEN 100 ELSE 0 END
    + CASE WHEN T.icodes LIKE '%3002725%' THEN 120 ELSE 0 END
  ) AS จำนวน_Point
FROM (
  SELECT
    hms.service_date, p.cid,
    CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
    pt.name AS main_dep, hms.hn, o.vn,
    GROUP_CONCAT(DISTINCT hmo.service_icode ORDER BY hmo.service_icode SEPARATOR ',') AS icodes
  FROM health_med_service AS hms
  INNER JOIN health_med_service_operation AS hmo ON hms.health_med_service_id = hmo.health_med_service_id
  INNER JOIN ovst AS o ON hms.hn = o.hn AND hms.service_date = o.vstdate
  INNER JOIN patient AS p ON hms.hn = p.hn
  LEFT JOIN pttype AS pt ON o.pttype = pt.pttype
  WHERE hms.service_date BETWEEN '{{date_from}}' AND '{{date_to}}'
    AND pt.hipdata_code IN ('UCS', 'WEL')
  GROUP BY hms.service_date, p.cid, patient_name, pt.name, hms.hn, o.vn
) AS T
ORDER BY T.service_date, T.hn"#,
            },
            // ── Audit 1 ───────────────────────────────────────────────────
            QuerySeed {
                mode: "audit",
                name: "ยาสมุนไพร 32 รายการ — ยังไม่ลงรหัส U (สิทธิ์ UC)",
                description: "สิทธิ์ UC กับยาสมุนไพร ที่ยังไม่ลงรหัส U",
                sql_text: r#"SELECT
  o.vstdate,
  p.cid,
  CONCAT(p.pname, p.fname, ' ', p.lname) AS name,
  pt.name AS main_dep,
  di.name AS icode_name,
  oi.qty,
  oi.sum_price,
  CASE
    WHEN EXISTS (SELECT 1 FROM ovstdiag od WHERE od.vn = o.vn AND (
      (oi.icode IN ('1660007','1580016','1620004','1580019','1650044') AND od.icd10 REGEXP '^U750|^U572|^U573') OR
      (oi.icode = '1670027' AND od.icd10 LIKE 'U6980') OR
      (oi.icode = '1580003' AND od.icd10 REGEXP '^U6682|^U613') OR
      (oi.icode = '1610009' AND od.icd10 REGEXP '^U6680|^U6670') OR
      (oi.icode = '1500018' AND od.icd10 REGEXP '^U6680|^U6670|^U6684') OR
      (oi.icode = '1530080' AND od.icd10 LIKE 'U680') OR
      (oi.icode = '1580023' AND od.icd10 REGEXP '^U6984|^U6985') OR
      (oi.icode IN ('161025','1610025') AND od.icd10 LIKE 'U6131') OR
      (oi.icode = '1650006' AND od.icd10 REGEXP '^U7522|^U756') OR
      (oi.icode = '1550013' AND od.icd10 REGEXP '^U756|^U6570|^U561|^U569') OR
      (oi.icode = '1600027' AND od.icd10 REGEXP '^U569|^U6570') OR
      (oi.icode = '1580004' AND od.icd10 REGEXP '^U561|^U569') OR
      (oi.icode IN ('1660013','1590008') AND od.icd10 LIKE 'U643') OR
      (oi.icode = '1530061' AND od.icd10 REGEXP '^U707|^U5603')
    )) THEN ''
    ELSE
      CASE
        WHEN oi.icode IN ('1660007','1580016','1620004','1580019','1650044') THEN 'U750%, U572%, U573%'
        WHEN oi.icode = '1670027' THEN 'U6980'
        WHEN oi.icode = '1580003' THEN 'U6682, U6131'
        WHEN oi.icode = '1610009' THEN 'U6680, U6670'
        WHEN oi.icode = '1500018' THEN 'U6680, U6670%, U6684'
        WHEN oi.icode = '1530080' THEN 'U680'
        WHEN oi.icode = '1580023' THEN 'U6984, U6985'
        WHEN oi.icode IN ('161025','1610025') THEN 'U6131'
        WHEN oi.icode = '1650006' THEN 'U7522, U756'
        WHEN oi.icode = '1550013' THEN 'U756, U6570, U561%, U569'
        WHEN oi.icode = '1600027' THEN 'U569, U6570'
        WHEN oi.icode = '1580004' THEN 'U561%, U569'
        WHEN oi.icode IN ('1660013','1590008') THEN 'U643'
        WHEN oi.icode = '1530061' THEN 'U707%, U5603'
        ELSE ''
      END
  END AS Suggest_U,
  o.hn
FROM ovst o
INNER JOIN patient p ON o.hn = p.hn
INNER JOIN opitemrece oi ON o.vn = oi.vn
INNER JOIN drugitems di ON oi.icode = di.icode
INNER JOIN pttype pt ON o.pttype = pt.pttype
WHERE o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
  AND di.icode IN ('1660007','1580016','1620004','1580019','1670027','1580003','1610009',
      '1500018','1530080','1580023','1610025','1650006','1550013','1600027','1580004',
      '1660013','1590008','1530061','1650044')
  AND pt.hipdata_code IN ('WEL','UCS')
GROUP BY oi.hos_guid
HAVING Suggest_U <> ''
ORDER BY o.vstdate, p.fname, p.lname"#,
            },
            // ── Audit 2 ───────────────────────────────────────────────────
            QuerySeed {
                mode: "audit",
                name: "ยาสมุนไพร 32 รายการ — ลง Diag ทั้งหมด / ยังไม่ลงรหัส U, M",
                description: "สิทธิ์ UC กับยาสมุนไพร ที่ลง Diag ทั้งหมด และที่ยังไม่ลงรหัส U, M",
                sql_text: r#"SELECT
  o.vstdate,
  p.cid,
  CONCAT(p.pname, p.fname, ' ', p.lname) AS name,
  pt.name AS main_dep,
  di.name AS icode_name,
  oi.qty,
  oi.sum_price,
  (SELECT GROUP_CONCAT(DISTINCT od.icd10 ORDER BY od.diagtype) FROM ovstdiag od WHERE od.vn = o.vn) AS all_icd10,
  CASE
    WHEN EXISTS (SELECT 1 FROM ovstdiag od WHERE od.vn = o.vn AND (
      (oi.icode IN ('1660007','1580016','1620004','1580019','1650044') AND od.icd10 REGEXP '^M791|^M796|^M5499|^M545|^M255|^M626|^M17') OR
      (oi.icode = '1670027' AND od.icd10 LIKE 'A099%') OR
      (oi.icode = '1580003' AND od.icd10 REGEXP '^R11|^R42') OR
      (oi.icode = '1610009' AND od.icd10 REGEXP '^R14|^R101') OR
      (oi.icode = '1500018' AND od.icd10 REGEXP '^R14|^R101|^R291') OR
      (oi.icode = '1530080' AND od.icd10 LIKE 'K64%') OR
      (oi.icode = '1580023' AND od.icd10 LIKE 'K590%') OR
      (oi.icode IN ('161025','1610025') AND od.icd10 REGEXP '^R42|^G470|^F510') OR
      (oi.icode = '1650006' AND od.icd10 REGEXP '^G470|^F510|^R630|^R53') OR
      (oi.icode = '1550013' AND od.icd10 REGEXP '^R630|^K120|^J00') OR
      (oi.icode = '1600027' AND od.icd10 REGEXP '^J00|^K120') OR
      (oi.icode = '1580004' AND od.icd10 REGEXP '^I00|^J029') OR
      (oi.icode IN ('1660013','1590008') AND od.icd10 REGEXP '^R05|^R070|^R093') OR
      (oi.icode = '1530061' AND od.icd10 REGEXP '^B00|^B02')
    )) THEN ''
    ELSE
      CASE
        WHEN oi.icode IN ('1660007','1580016','1620004','1580019','1650044') THEN 'M791%, M796%, M5499%, M545%, M255%, M626%, M17%'
        WHEN oi.icode = '1670027' THEN 'A099%'
        WHEN oi.icode = '1580003' THEN 'R11%, R42%'
        WHEN oi.icode = '1610009' THEN 'R14%, R101%'
        WHEN oi.icode = '1500018' THEN 'R14%, R101%, R291%'
        WHEN oi.icode = '1530080' THEN 'K64%'
        WHEN oi.icode = '1580023' THEN 'K590%'
        WHEN oi.icode IN ('161025','1610025') THEN 'R42%, G470%, F510%'
        WHEN oi.icode = '1650006' THEN 'G470%, F510%, R630%, R53%'
        WHEN oi.icode = '1550013' THEN 'R630%, K120%, J00%'
        WHEN oi.icode = '1600027' THEN 'J00%, K120%'
        WHEN oi.icode = '1580004' THEN 'I00%, J029%'
        WHEN oi.icode IN ('1660013','1590008') THEN 'R05%, R070%, R093%'
        WHEN oi.icode = '1530061' THEN 'B00%, B02%'
        ELSE ''
      END
  END AS Suggest_M,
  CASE
    WHEN EXISTS (SELECT 1 FROM ovstdiag od WHERE od.vn = o.vn AND (
      (oi.icode IN ('1660007','1580016','1620004','1580019','1650044') AND od.icd10 REGEXP '^U750|^U572|^U573') OR
      (oi.icode = '1670027' AND od.icd10 LIKE 'U6980%') OR
      (oi.icode = '1580003' AND od.icd10 REGEXP '^U6682|^U613') OR
      (oi.icode = '1610009' AND od.icd10 REGEXP '^U6680|^U6670') OR
      (oi.icode = '1500018' AND od.icd10 REGEXP '^U6680|^U6670|^U6684') OR
      (oi.icode = '1530080' AND od.icd10 LIKE 'U680%') OR
      (oi.icode = '1580023' AND od.icd10 REGEXP '^U6984|^U6985') OR
      (oi.icode IN ('161025','1610025') AND od.icd10 LIKE 'U613%') OR
      (oi.icode = '1650006' AND od.icd10 REGEXP '^U7522|^U756') OR
      (oi.icode = '1550013' AND od.icd10 REGEXP '^U756|^U6570|^U561|^U569') OR
      (oi.icode = '1600027' AND od.icd10 REGEXP '^U569|^U6570') OR
      (oi.icode = '1580004' AND od.icd10 REGEXP '^U561|^U569') OR
      (oi.icode IN ('1660013','1590008') AND od.icd10 LIKE 'U643%') OR
      (oi.icode = '1530061' AND od.icd10 REGEXP '^U707|^U5603')
    )) THEN ''
    ELSE
      CASE
        WHEN oi.icode IN ('1660007','1580016','1620004','1580019','1650044') THEN 'U750%, U572%, U573%'
        WHEN oi.icode = '1670027' THEN 'U6980%'
        WHEN oi.icode = '1580003' THEN 'U6682%, U613%'
        WHEN oi.icode = '1610009' THEN 'U6680%, U6670%'
        WHEN oi.icode = '1500018' THEN 'U6680%, U6670%, U6684%'
        WHEN oi.icode = '1530080' THEN 'U680%'
        WHEN oi.icode = '1580023' THEN 'U6984%, U6985%'
        WHEN oi.icode IN ('161025','1610025') THEN 'U613%'
        WHEN oi.icode = '1650006' THEN 'U7522%, U756%'
        WHEN oi.icode = '1550013' THEN 'U756%, U6570%, U561%, U569%'
        WHEN oi.icode = '1600027' THEN 'U569%, U6570%'
        WHEN oi.icode = '1580004' THEN 'U561%, U569%'
        WHEN oi.icode IN ('1660013','1590008') THEN 'U643%'
        WHEN oi.icode = '1530061' THEN 'U707%, U5603%'
        ELSE ''
      END
  END AS Suggest_U,
  o.hn
FROM ovst o
INNER JOIN patient p ON o.hn = p.hn
INNER JOIN opitemrece oi ON o.vn = oi.vn
INNER JOIN drugitems di ON oi.icode = di.icode
INNER JOIN pttype pt ON o.pttype = pt.pttype
WHERE o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
  AND di.icode IN ('1660007','1580016','1620004','1580019','1670027','1580003','1610009',
      '1500018','1530080','1580023','1610025','1650006','1550013','1600027','1580004',
      '1660013','1590008','1530061','1650044')
  AND pt.hipdata_code IN ('WEL','UCS')
GROUP BY oi.hos_guid
HAVING Suggest_M <> '' OR Suggest_U <> ''
ORDER BY o.vstdate, p.fname, p.lname"#,
            },
            // ── Audit 3 ───────────────────────────────────────────────────
            QuerySeed {
                mode: "audit",
                name: "ตรวจสอบการปิดสิทธิ์ UC / อปท. (หัตถการ+ยาสมุนไพร)",
                description: "ตรวจสอบการปิดสิทธิ์ UC / อปท. ที่เกี่ยวข้องกับแพทย์แผนไทย (หัตถการ+ยาสมุนไพร)",
                sql_text: r#"SELECT
    o.vstdate AS visit_date,
    o.vn AS VN,
    o.hn AS HN,
    p.cid AS CID,
    CONCAT(p.pname, p.fname, ' ', p.lname) AS patient_name,
    pt.name AS pttype_name,
    GROUP_CONCAT(DISTINCT
        CONCAT(COALESCE(d.name, n.name), ' [', oi.qty, ']')
        SEPARATOR ', '
    ) AS item_details,
    SUM(oi.sum_price) AS total_visit_cost,
    vp.auth_code
FROM ovst o
INNER JOIN patient p ON p.hn = o.hn
INNER JOIN opitemrece oi ON o.vn = oi.vn
INNER JOIN pttype pt ON pt.pttype = o.pttype
INNER JOIN visit_pttype vp ON o.vn = vp.vn
LEFT JOIN nondrugitems n ON n.icode = oi.icode
LEFT JOIN drugitems d ON d.icode = oi.icode
WHERE
    o.vstdate BETWEEN '{{date_from}}' AND '{{date_to}}'
    -- Exclude admitted patients
    AND (o.an IS NULL OR o.an = '')
    -- Filter by hipdata_code instead of pttype list
    AND pt.hipdata_code IN ('UCS', 'WEL', 'LGO')
    -- Filter out any auth_code starting with 'EP'
    AND (vp.auth_code NOT LIKE 'EP%' OR vp.auth_code IS NULL)
GROUP BY
    o.vn,
    o.vstdate,
    o.hn,
    p.cid,
    patient_name,
    pttype_name,
    vp.auth_code
HAVING
    -- Must have at least 1 record from the specified icode list
    SUM(CASE WHEN oi.icode IN (
        '3003038', '3002724', '3003012', '3003013', '3003021', '3003014', '3003016', '3003888',
        '1660007', '1580016', '1620004', '1580019', '1670027', '1580003', '1610009', '1500018',
        '1530080', '1580023', '1610025', '1650006', '1550013', '1600027', '1580004', '1660013',
        '1590008', '1530061', '1650044'
    ) THEN 1 ELSE 0 END) >= 1
ORDER BY o.vstdate DESC, o.vn"#,
            },
        ];

        for (i, q) in queries.iter().enumerate() {
            conn.execute(
                "INSERT INTO sql_queries (mode, name, description, sql_text, sort_order, enabled, is_starred) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, 0)",
                params![q.mode, q.name, q.description, q.sql_text, i as i64],
            )?;
        }

        // Assign all seeded queries to TTM department
        let ttm_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM departments WHERE name = 'TTM (แพทย์แผนไทย)'",
                [],
                |r| r.get(0),
            )
            .ok();
        if let Some(ttm) = ttm_id {
            conn.execute_batch(&format!(
                "UPDATE sql_queries SET department_id = {} WHERE department_id IS NULL",
                ttm
            ))
            .ok();
        }
    }

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

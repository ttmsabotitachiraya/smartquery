// tauri-api.ts — typed wrappers for Tauri commands

import { invoke } from "@tauri-apps/api/core";
import type {
  DbConfig,
  SqlQuery,
  Department,
  QueryResult,
  ExecutionHistory,
} from "./types";

export const api = {
  // DB Config
  loadDbConfig: () => invoke<DbConfig>("load_db_config"),
  saveDbConfig: (config: DbConfig) =>
    invoke<void>("save_db_config", {
      host: config.host,
      port: config.port,
      user: config.user,
      password: config.password,
      database: config.database,
    }),
  testConnection: (config: DbConfig) =>
    invoke<string>("test_connection", { config }),
  connectDb: (config: DbConfig) => invoke<string>("connect_db", { config }),

  // Query Execution
  executeQuery: (sql: string, dateFrom: string, dateTo: string) =>
    invoke<QueryResult>("execute_query", { sql, dateFrom, dateTo }),
  validateSql: (sql: string) => invoke<string>("validate_sql", { sql }),

  // Departments
  getAllDepartments: () => invoke<Department[]>("get_all_departments"),
  insertDepartment: (name: string) =>
    invoke<number>("insert_department", { name }),
  updateDepartment: (deptId: number, name: string) =>
    invoke<boolean>("update_department", { deptId, name }),
  deleteDepartment: (deptId: number) =>
    invoke<boolean>("delete_department", { deptId }),

  // Queries CRUD
  getAllQueries: (mode: string) =>
    invoke<SqlQuery[]>("get_all_queries", { mode }),
  getEnabledQueries: (mode: string, departmentId?: number) =>
    invoke<SqlQuery[]>("get_enabled_queries", {
      mode,
      departmentId: departmentId ?? 0,
    }),
  getQueryById: (queryId: number) =>
    invoke<SqlQuery | null>("get_query_by_id", { queryId }),
  insertQuery: (
    mode: string,
    name: string,
    description: string,
    sqlText: string,
    departmentId: number | null,
    isStarred: boolean,
  ) =>
    invoke<number>("insert_query", {
      mode,
      name,
      description,
      sqlText,
      departmentId,
      isStarred,
    }),
  updateQuery: (
    queryId: number,
    name: string,
    description: string,
    sqlText: string,
    enabled: boolean,
    departmentId: number | null,
    isStarred?: boolean,
  ) =>
    invoke<boolean>("update_query", {
      queryId,
      name,
      description,
      sqlText,
      enabled,
      departmentId,
      isStarred: isStarred ?? null,
    }),
  deleteQuery: (queryId: number) =>
    invoke<boolean>("delete_query", { queryId }),
  setQueryEnabled: (queryId: number, enabled: boolean) =>
    invoke<void>("set_query_enabled", { queryId, enabled }),
  setQueryStarred: (queryId: number, starred: boolean) =>
    invoke<void>("set_query_starred", { queryId, starred }),
  searchQueries: (
    mode: string,
    keyword: string,
    statusFilter: string,
    departmentId: number,
    starredOnly: boolean,
  ) =>
    invoke<SqlQuery[]>("search_queries", {
      mode,
      keyword,
      statusFilter,
      departmentId,
      starredOnly,
    }),
  countQueries: (mode: string) => invoke<number>("count_queries", { mode }),

  // Execution History
  logExecution: (args: {
    queryId: number | null;
    queryName: string;
    mode: string;
    dateFrom: string;
    dateTo: string;
    rowCount: number;
    elapsedSec: number;
    status: string;
    errorMsg: string;
  }) =>
    invoke<void>("log_execution", {
      args: {
        query_id: args.queryId,
        query_name: args.queryName,
        mode: args.mode,
        date_from: args.dateFrom,
        date_to: args.dateTo,
        row_count: args.rowCount,
        elapsed_sec: args.elapsedSec,
        status: args.status,
        error_msg: args.errorMsg,
      },
    }),
  getExecutionHistory: (
    limit: number,
    modeFilter: string,
    statusFilter: string,
  ) =>
    invoke<ExecutionHistory[]>("get_execution_history", {
      limit,
      modeFilter,
      statusFilter,
    }),
  clearExecutionHistory: () => invoke<void>("clear_execution_history"),
};

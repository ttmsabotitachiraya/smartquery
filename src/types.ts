// types.ts — shared TypeScript types

export interface DbConfig {
  host: string;
  port: number;
  user: string;
  password: string;
  database: string;
}

export interface SqlQuery {
  id: number;
  mode: "audit" | "report";
  name: string;
  description: string;
  sql_text: string;
  sort_order: number;
  enabled: number;
  is_starred: number;
  created_at: string;
  updated_at: string;
  department_id: number | null;
  department_name: string;
}

export interface Department {
  id: number;
  name: string;
  sort_order: number;
}

export interface QueryResult {
  columns: string[];
  rows: (string | number | null)[][];
  elapsed_sec: number;
  row_count: number;
}

export interface ExecutionHistory {
  id: number;
  query_id: number | null;
  query_name: string;
  mode: string;
  date_from: string;
  date_to: string;
  row_count: number;
  elapsed_sec: number;
  status: string;
  error_msg: string;
  executed_at: string;
}

export type QueryStatus = "idle" | "running" | "pass" | "error" | "stopped";

export interface AuditQueryRow extends SqlQuery {
  status: QueryStatus;
  row_count?: number;
  elapsed_sec?: number;
  error_msg?: string;
  result_columns?: string[];
  result_rows?: (string | number | null)[][];
}

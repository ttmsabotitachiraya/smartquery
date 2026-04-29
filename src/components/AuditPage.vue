<template>
    <div class="flex flex-col h-full">
        <!-- Header -->
        <div class="page-header">
            <Search :size="22" class="page-header-icon" />
            <div>
                <h1 class="text-base font-bold text-[#E9ECEF]">Audit</h1>
                <p class="text-xs text-[#888]">
                    ตรวจสอบความถูกต้องของข้อมูล HOSxP
                </p>
            </div>
            <div class="ml-auto flex items-center gap-2">
                <div
                    v-if="running"
                    class="flex items-center gap-2 text-xs text-[#FFB700]"
                >
                    <Loader2 :size="14" class="animate-spin" />
                    {{ doneCount }}/{{ totalCount }}
                </div>
                <span
                    v-if="!dbConnected"
                    class="text-xs text-[#FF4D00] flex items-center gap-1"
                >
                    <AlertCircle :size="14" /> ไม่ได้เชื่อมต่อ
                </span>
            </div>
        </div>

        <!-- Toolbar -->
        <div class="toolbar">
            <!-- Date range (vue-datepicker-next) -->
            <div class="flex items-center gap-2 text-sm">
                <CalendarDays :size="15" class="text-[#888]" />
                <CustomDatePicker
                    v-model="dateFromObj"
                    placeholder="วันเริ่มต้น"
                />
                <span class="text-[#555]">—</span>
                <CustomDatePicker
                    v-model="dateToObj"
                    placeholder="วันสิ้นสุด"
                />
            </div>

            <div class="toolbar-divider"></div>

            <!-- Run buttons -->
            <button
                class="btn-primary text-sm"
                :disabled="running || !dbConnected"
                @click="runSelected"
            >
                <PlayCircle :size="14" /> Run Selected
            </button>
            <button
                class="btn-secondary text-sm"
                :disabled="running || !dbConnected"
                @click="runAll"
            >
                <Play :size="14" /> Run All
            </button>
            <button
                class="btn-warning text-sm"
                :disabled="running || !dbConnected"
                @click="runStarred"
            >
                <Star :size="14" /> Run Starred
            </button>
            <button v-if="running" class="btn-danger text-sm" @click="stop">
                <Square :size="14" /> Stop
            </button>
            <button
                class="btn-ghost text-sm"
                :disabled="running"
                @click="resetAll"
            >
                <RotateCcw :size="14" /> Reset
            </button>
        </div>

        <!-- Filter bar -->
        <div class="filter-bar">
            <Search :size="14" class="text-[#555] flex-shrink-0" />
            <input
                v-model="searchText"
                class="input-field flex-1 text-xs"
                placeholder="ค้นหา Query..."
            />
            <div class="select-wrap w-44 flex-shrink-0">
                <select
                    v-model="departmentFilter"
                    class="text-xs"
                    @change="onDeptFilterChange"
                >
                    <option :value="0">All Departments</option>
                    <option v-for="d in departments" :key="d.id" :value="d.id">
                        {{ d.name }}
                    </option>
                </select>
            </div>
            <div class="select-wrap w-32 flex-shrink-0">
                <select v-model="statusFilter" class="text-xs">
                    <option value="all">All Status</option>
                    <option value="pass">Pass</option>
                    <option value="notpass">Not Pass</option>
                    <option value="error">Error</option>
                </select>
            </div>
        </div>

        <!-- Progress bar -->
        <div v-if="running" class="h-1 bg-[#1A1A1B]">
            <div
                class="h-1 bg-[#FF4D00] transition-all duration-300"
                :style="{ width: `${progress}%` }"
            ></div>
        </div>

        <!-- Column header row -->
        <div
            class="sticky top-0 z-10 bg-[#252526] border-b border-[#3A3A3B] px-4 py-2 flex items-center gap-3 text-[10px] font-bold uppercase tracking-wider text-[#666]"
        >
            <!-- Select All checkbox -->
            <div class="w-4 flex items-center justify-center">
                <input
                    type="checkbox"
                    :checked="allSelected"
                    @change="toggleSelectAll"
                    class="w-4 h-4 accent-[#FF4D00] cursor-pointer"
                    title="Select All / Deselect All"
                />
            </div>
            <!-- Star placeholder -->
            <div class="w-4"></div>
            <!-- Name (sortable) -->
            <div
                class="flex-1 flex items-center gap-1 cursor-pointer select-none hover:text-[#aaa] transition-colors"
                @click="toggleSort('name')"
            >
                Query / ชื่อ
                <span class="text-[#444]">
                    <span v-if="sortKey === 'name'">{{
                        sortDir === "asc" ? "▲" : "▼"
                    }}</span>
                    <span v-else>⇅</span>
                </span>
            </div>
            <!-- Department (sortable) -->
            <div
                class="w-32 flex items-center gap-1 cursor-pointer select-none hover:text-[#aaa] transition-colors"
                @click="toggleSort('department')"
            >
                Department
                <span class="text-[#444]">
                    <span v-if="sortKey === 'department'">{{
                        sortDir === "asc" ? "▲" : "▼"
                    }}</span>
                    <span v-else>⇅</span>
                </span>
            </div>
            <!-- Status (sortable) -->
            <div
                class="w-24 flex items-center justify-center gap-1 cursor-pointer select-none hover:text-[#aaa] transition-colors"
                @click="toggleSort('status')"
            >
                Status
                <span class="text-[#444]">
                    <span v-if="sortKey === 'status'">{{
                        sortDir === "asc" ? "▲" : "▼"
                    }}</span>
                    <span v-else>⇅</span>
                </span>
            </div>
            <!-- Row count (sortable) -->
            <div
                class="w-16 flex items-center justify-end gap-1 cursor-pointer select-none hover:text-[#aaa] transition-colors"
                @click="toggleSort('rows')"
            >
                Rows
                <span class="text-[#444]">
                    <span v-if="sortKey === 'rows'">{{
                        sortDir === "asc" ? "▲" : "▼"
                    }}</span>
                    <span v-else>⇅</span>
                </span>
            </div>
            <!-- Elapsed time (sortable) -->
            <div
                class="w-16 flex items-center justify-end gap-1 cursor-pointer select-none hover:text-[#aaa] transition-colors"
                @click="toggleSort('time')"
            >
                Time
                <span class="text-[#444]">
                    <span v-if="sortKey === 'time'">{{
                        sortDir === "asc" ? "▲" : "▼"
                    }}</span>
                    <span v-else>⇅</span>
                </span>
            </div>
            <!-- Export All button -->
            <div class="w-[120px] flex items-center justify-end flex-shrink-0">
                <button
                    class="btn-secondary text-xs px-2 py-1"
                    :disabled="notPassRows.length === 0"
                    @click="exportAllXlsx"
                    title="ส่งออก Excel ทั้งหมดที่มีปัญหา (Not Pass)"
                >
                    <FileDown :size="13" /> Export All
                </button>
            </div>
        </div>

        <!-- Query rows -->
        <div class="flex-1 overflow-y-auto">
            <div
                v-if="sortedRows.length === 0"
                class="flex flex-col items-center justify-center h-64 text-[#555] gap-3"
            >
                <Database :size="48" />
                <p>ไม่พบ Query</p>
            </div>

            <div
                v-for="(row, idx) in sortedRows"
                :key="row.id"
                :class="[
                    'border-b border-[#3A3A3B]/50 px-4 py-3 flex items-center gap-3 transition-colors',
                    idx % 2 === 1 ? 'bg-white/[0.02]' : '',
                    row.status === 'running' ? 'bg-[#FFB700]/5' : '',
                ]"
            >
                <!-- Checkbox -->
                <div class="w-4 flex items-center justify-center">
                    <input
                        type="checkbox"
                        v-model="row.selected"
                        class="w-4 h-4 accent-[#FF4D00] cursor-pointer rounded"
                    />
                </div>

                <!-- Star -->
                <div class="w-4 flex items-center justify-center">
                    <button
                        @click="toggleStar(row)"
                        :class="[
                            'p-0.5 transition-colors',
                            row.is_starred
                                ? 'text-[#FFB700]'
                                : 'text-[#555] hover:text-[#FFB700]',
                        ]"
                    >
                        <Star
                            :size="15"
                            :fill="row.is_starred ? '#FFB700' : 'none'"
                        />
                    </button>
                </div>

                <!-- Query info -->
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                        <span
                            class="font-medium text-sm text-[#E9ECEF] truncate"
                            >{{ row.name }}</span
                        >
                    </div>
                    <p
                        v-if="row.description"
                        class="text-xs text-[#888] truncate mt-0.5"
                    >
                        {{ row.description }}
                    </p>
                </div>

                <!-- Department -->
                <div class="w-32 flex-shrink-0 text-xs text-[#aaa] truncate">
                    <span
                        v-if="row.department_name"
                        class="badge-department text-[10px]"
                        >{{ row.department_name }}</span
                    >
                    <span v-else class="text-[#555]">—</span>
                </div>

                <!-- Status badge -->
                <div class="w-24 flex justify-center flex-shrink-0">
                    <span
                        v-if="row.status === 'running'"
                        class="badge-warning flex items-center gap-1"
                    >
                        <Loader2 :size="10" class="animate-spin" /> Running
                    </span>
                    <span
                        v-else-if="
                            row.status === 'pass' && (row.row_count ?? 0) === 0
                        "
                        class="badge-success"
                        >✓ Pass</span
                    >
                    <span
                        v-else-if="
                            row.status === 'pass' && (row.row_count ?? 0) > 0
                        "
                        class="badge-notpass"
                        >✗ Not Pass</span
                    >
                    <span
                        v-else-if="row.status === 'error'"
                        class="badge-error"
                        :title="row.error_msg"
                        >✗ Error</span
                    >
                    <span
                        v-else-if="row.status === 'stopped'"
                        class="badge-neutral"
                        >Stopped</span
                    >
                </div>

                <!-- Row count -->
                <div
                    class="w-16 text-right text-xs tabular-nums text-[#aaa] flex-shrink-0"
                >
                    <span v-if="row.status === 'pass'">
                        {{ row.row_count?.toLocaleString() }} rows
                    </span>
                </div>

                <!-- Elapsed time -->
                <div
                    class="w-16 text-right text-xs tabular-nums text-[#888] flex-shrink-0"
                >
                    <span v-if="row.elapsed_sec !== undefined">
                        {{ row.elapsed_sec.toFixed(2) }}s
                    </span>
                </div>

                <!-- Action buttons (show only when row_count > 0) -->
                <div
                    class="w-[120px] flex items-center justify-end gap-1 flex-shrink-0"
                >
                    <button
                        v-if="
                            row.status === 'pass' &&
                            row.result_columns &&
                            (row.row_count ?? 0) > 0
                        "
                        class="btn-ghost text-xs px-2 py-1"
                        @click="viewResult(row)"
                    >
                        <Eye :size="13" /> View
                    </button>
                    <button
                        v-if="
                            row.status === 'pass' &&
                            row.result_columns &&
                            (row.row_count ?? 0) > 0
                        "
                        class="btn-secondary text-xs px-2 py-1"
                        @click="exportXlsx(row)"
                    >
                        <FileDown :size="13" /> Excel
                    </button>
                </div>
            </div>
        </div>

        <!-- Summary bar -->
        <div
            class="px-6 py-2 border-t border-[#3A3A3B] flex justify-between items-center text-xs text-[#555]"
        >
            <span>พัฒนาโดย: กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์ ลพบุรี</span>
            <span>
                {{ passCount }} pass · {{ notPassCount }} not pass ·
                {{ errorCount }} error · {{ auditRows.length }} queries
            </span>
        </div>

        <!-- Result Modal -->
        <ResultModal
            v-if="modalQuery"
            :query="modalQuery"
            @close="modalQuery = null"
        />

        <!-- Export Toast Notification -->
        <Transition name="toast-fade">
            <div
                v-if="exporting"
                class="fixed bottom-6 right-6 z-50 flex items-center gap-3 px-4 py-3 rounded-xl bg-[#1A1A1B] border border-[#3A3A3B] shadow-2xl text-sm text-[#E9ECEF]"
            >
                <Loader2
                    v-if="
                        !exportMsg.startsWith('✓') && !exportMsg.startsWith('✗')
                    "
                    :size="16"
                    class="animate-spin text-[#80B918] flex-shrink-0"
                />
                <span
                    v-else-if="exportMsg.startsWith('✓')"
                    class="text-[#80B918] flex-shrink-0 text-base leading-none"
                    >✓</span
                >
                <span
                    v-else
                    class="text-[#EF233C] flex-shrink-0 text-base leading-none"
                    >✗</span
                >
                <span>{{ exportMsg }}</span>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from "vue";
import {
    Search,
    CalendarDays,
    Play,
    PlayCircle,
    Star,
    Square,
    RotateCcw,
    Database,
    Loader2,
    AlertCircle,
    Eye,
    FileDown,
} from "lucide-vue-next";
import CustomDatePicker from "./CustomDatePicker.vue";
import { utils, write as xlsxWrite } from "xlsx";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { writeFile as tauriWriteFile } from "@tauri-apps/plugin-fs";
import ResultModal from "./ResultModal.vue";
import { api } from "../tauri-api";
import type { SqlQuery, Department, DbConfig, AuditQueryRow } from "../types";

interface AuditRow extends AuditQueryRow {
    selected: boolean;
    is_starred: number;
    description: string;
}

defineProps<{
    dbConfig: DbConfig | null;
    dbConnected: boolean;
}>();

// ── Date helpers ──────────────────────────────────────────────────────────────
const now = new Date();
const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
const endOfMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);

/** Two-way binding for the DatePicker (Date objects) */
const dateFromObj = ref<Date | null>(startOfMonth);
const dateToObj = ref<Date | null>(endOfMonth);

/** Format a Date → "YYYY-MM-DD" string for the API */
function fmtDate(d: Date): string {
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

const dateFrom = computed(() =>
    dateFromObj.value ? fmtDate(dateFromObj.value) : "",
);
const dateTo = computed(() =>
    dateToObj.value ? fmtDate(dateToObj.value) : "",
);

// ── State ─────────────────────────────────────────────────────────────────────
const auditRows = ref<AuditRow[]>([]);
const departments = ref<Department[]>([]);
const searchText = ref("");
const departmentFilter = ref<number>(0);
const statusFilter = ref("all");
const running = ref(false);
const stopRequested = ref(false);
const doneCount = ref(0);
const totalCount = ref(0);
const modalQuery = ref<AuditQueryRow | null>(null);
const exporting = ref(false);
const exportMsg = ref("");

/** Returns the logical display status for filter/badge purposes */
function getDisplayStatus(row: AuditRow): string {
    if (!row.status || row.status === "idle") return "idle";
    if (row.status === "pass") {
        return (row.row_count ?? 0) > 0 ? "notpass" : "pass";
    }
    return row.status;
}

// ── Computed: filtered rows ───────────────────────────────────────────────────
// ── Sort state ────────────────────────────────────────────────────────────────
const sortKey = ref<"name" | "department" | "status" | "rows" | "time" | null>(
    null,
);
const sortDir = ref<"asc" | "desc">("asc");

function toggleSort(key: "name" | "department" | "status" | "rows" | "time") {
    if (sortKey.value === key) {
        sortDir.value = sortDir.value === "asc" ? "desc" : "asc";
    } else {
        sortKey.value = key;
        sortDir.value = "asc";
    }
}

const filteredRows = computed(() => {
    return auditRows.value.filter((row) => {
        const matchText =
            !searchText.value ||
            row.name.toLowerCase().includes(searchText.value.toLowerCase()) ||
            row.description
                ?.toLowerCase()
                .includes(searchText.value.toLowerCase());

        // departmentFilter === 0  → show everything (All Departments)
        // departmentFilter > 0    → match by department_id
        const matchGroup =
            departmentFilter.value === 0 ||
            row.department_id === departmentFilter.value;

        const matchStatus =
            statusFilter.value === "all" ||
            getDisplayStatus(row) === statusFilter.value;

        return matchText && matchGroup && matchStatus;
    });
});

const sortedRows = computed(() => {
    if (!sortKey.value) return filteredRows.value;
    const key = sortKey.value;
    const dir = sortDir.value === "asc" ? 1 : -1;
    return [...filteredRows.value].sort((a, b) => {
        let av: string | number | null | undefined;
        let bv: string | number | null | undefined;
        if (key === "name") {
            av = a.name?.toLowerCase() ?? "";
            bv = b.name?.toLowerCase() ?? "";
        } else if (key === "department") {
            av = a.department_name?.toLowerCase() ?? "";
            bv = b.department_name?.toLowerCase() ?? "";
        } else if (key === "status") {
            av = getDisplayStatus(a);
            bv = getDisplayStatus(b);
        } else if (key === "rows") {
            av = a.row_count ?? -1;
            bv = b.row_count ?? -1;
        } else if (key === "time") {
            av = a.elapsed_sec ?? -1;
            bv = b.elapsed_sec ?? -1;
        }
        if (av === undefined || av === null) av = "";
        if (bv === undefined || bv === null) bv = "";
        if (av < bv) return -1 * dir;
        if (av > bv) return 1 * dir;
        return 0;
    });
});

const allSelected = computed(
    () =>
        filteredRows.value.length > 0 &&
        filteredRows.value.every((r) => r.selected),
);

const passCount = computed(
    () =>
        auditRows.value.filter(
            (r) => r.status === "pass" && (r.row_count ?? 0) === 0,
        ).length,
);
const notPassCount = computed(
    () =>
        auditRows.value.filter(
            (r) => r.status === "pass" && (r.row_count ?? 0) > 0,
        ).length,
);
const errorCount = computed(
    () => auditRows.value.filter((r) => r.status === "error").length,
);
const progress = computed(() =>
    totalCount.value > 0
        ? Math.round((doneCount.value / totalCount.value) * 100)
        : 0,
);

/** All rows that have issues (Not Pass) and have result data */
const notPassRows = computed(() =>
    auditRows.value.filter(
        (r) =>
            r.status === "pass" && (r.row_count ?? 0) > 0 && r.result_columns,
    ),
);

// ── Load data ─────────────────────────────────────────────────────────────────
/**
 * Load enabled queries for the currently selected department.
 * departmentFilter === 0 → getEnabledQueries("audit", 0) → backend returns ALL
 * departmentFilter > 0   → getEnabledQueries("audit", deptId) → backend filters
 */
async function loadData() {
    const [queries, depts] = await Promise.all([
        api.getEnabledQueries("audit", departmentFilter.value),
        api.getAllDepartments(),
    ]);
    departments.value = depts;
    auditRows.value = queries.map((q: SqlQuery) => ({
        ...q,
        status: "idle" as const,
        selected: false,
        row_count: undefined,
        elapsed_sec: undefined,
        error_msg: "",
        result_columns: undefined,
        result_rows: undefined,
    }));
}

/** Called when the department dropdown changes */
async function onDeptFilterChange() {
    // Only refilter client-side; do NOT reload or reset results
}

function toggleSelectAll() {
    const all = allSelected.value;
    filteredRows.value.forEach((r) => (r.selected = !all));
}

async function toggleStar(row: AuditRow) {
    const newStarred = !row.is_starred;
    await api.setQueryStarred(row.id, newStarred);
    row.is_starred = newStarred ? 1 : 0;
}

// ── Run ───────────────────────────────────────────────────────────────────────
async function runAll() {
    await runRows(auditRows.value);
}

async function runSelected() {
    await runRows(auditRows.value.filter((r) => r.selected));
}

async function runStarred() {
    await runRows(auditRows.value.filter((r) => r.is_starred));
}

async function runRows(rows: AuditRow[]) {
    if (rows.length === 0) return;
    running.value = true;
    stopRequested.value = false;
    doneCount.value = 0;
    totalCount.value = rows.length;

    rows.forEach((r) => {
        r.status = "idle";
        r.row_count = undefined;
        r.elapsed_sec = undefined;
        r.error_msg = "";
        r.result_columns = undefined;
        r.result_rows = undefined;
    });

    for (const row of rows) {
        if (stopRequested.value) {
            row.status = "stopped";
            continue;
        }
        row.status = "running";
        const t0 = performance.now();
        try {
            const result = await api.executeQuery(
                row.sql_text,
                dateFrom.value,
                dateTo.value,
            );
            row.status = "pass";
            row.row_count = result.row_count;
            row.elapsed_sec = result.elapsed_sec;
            row.result_columns = result.columns;
            row.result_rows = result.rows;
            api.logExecution({
                queryId: row.id,
                queryName: row.name,
                mode: "audit",
                dateFrom: dateFrom.value,
                dateTo: dateTo.value,
                rowCount: result.row_count,
                elapsedSec: result.elapsed_sec,
                status: result.row_count > 0 ? "notpass" : "ok",
                errorMsg: "",
            }).catch(() => {});
        } catch (e: unknown) {
            const msg = e instanceof Error ? e.message : String(e);
            row.status = "error";
            row.elapsed_sec = (performance.now() - t0) / 1000;
            row.error_msg = msg;
            api.logExecution({
                queryId: row.id,
                queryName: row.name,
                mode: "audit",
                dateFrom: dateFrom.value,
                dateTo: dateTo.value,
                rowCount: 0,
                elapsedSec: row.elapsed_sec,
                status: "error",
                errorMsg: msg,
            }).catch(() => {});
        }
        doneCount.value++;
    }

    running.value = false;
}

function stop() {
    stopRequested.value = true;
}

function resetAll() {
    auditRows.value.forEach((r) => {
        r.status = "idle";
        r.row_count = undefined;
        r.elapsed_sec = undefined;
        r.error_msg = "";
        r.result_columns = undefined;
        r.result_rows = undefined;
    });
    doneCount.value = 0;
    totalCount.value = 0;
}

function viewResult(row: AuditRow) {
    modalQuery.value = row;
}

// ── Export ────────────────────────────────────────────────────────────────────

/** Build the info header rows: [ ["ชื่อ:", name], ["คำอธิบาย:", description], ["วันที่:", from - to], [] ] */
function buildInfoRows(name: string, description: string): (string | null)[][] {
    return [
        ["ชื่อ:", name],
        ["คำอธิบาย:", description || "-"],
        ["ช่วงวันที่:", `${dateFrom.value} ถึง ${dateTo.value}`],
        [],
    ];
}

/** Export a single row's result to Excel */
async function exportXlsx(row: AuditRow) {
    if (!row.result_columns || !row.result_rows) return;
    exportMsg.value = `กำลังส่งออก "${row.name}"...`;
    exporting.value = true;
    await nextTick();
    await new Promise((r) => setTimeout(r, 50));
    try {
        const wb = utils.book_new();
        const infoRows = buildInfoRows(row.name, row.description);
        const wsData = [...infoRows, row.result_columns, ...row.result_rows];
        const ws = utils.aoa_to_sheet(wsData);
        const sheetName = row.name
            .replace(/[\/\\?*\[\]:]/g, "_")
            .substring(0, 31);
        utils.book_append_sheet(wb, ws, sheetName || "Audit");
        const dateStr =
            dateFrom.value === dateTo.value
                ? dateFrom.value.replace(/-/g, "")
                : `${dateFrom.value.replace(/-/g, "")}-${dateTo.value.replace(/-/g, "")}`;
        const defaultName = `${row.name.replace(/[\s\/\\?*\[\]:]/g, "_")}_${dateStr}.xlsx`;
        const savePath = await saveDialog({
            defaultPath: defaultName,
            filters: [{ name: "Excel", extensions: ["xlsx"] }],
        });
        if (!savePath) {
            exporting.value = false;
            return;
        }
        const xlsxBytes = xlsxWrite(wb, { type: "array", bookType: "xlsx" });
        await tauriWriteFile(savePath, new Uint8Array(xlsxBytes));
        exportMsg.value = `✓ ส่งออก "${row.name}" สำเร็จ`;
    } catch {
        exportMsg.value = "✗ เกิดข้อผิดพลาดในการส่งออก";
    }
    await new Promise((r) => setTimeout(r, 1500));
    exporting.value = false;
}

/** Export ALL not-pass rows to a single Excel file (one sheet per query) */
async function exportAllXlsx() {
    const rows = notPassRows.value;
    if (rows.length === 0) return;
    exportMsg.value = `กำลังส่งออก ${rows.length} รายการ (Not Pass)...`;
    exporting.value = true;
    await nextTick();
    await new Promise((r) => setTimeout(r, 50));
    try {
        const wb = utils.book_new();
        for (const row of rows) {
            const infoRows = buildInfoRows(row.name, row.description);
            const wsData = [
                ...infoRows,
                row.result_columns!,
                ...row.result_rows!,
            ];
            const ws = utils.aoa_to_sheet(wsData);
            const sheetName = row.name
                .replace(/[\/\\?*\[\]:]/g, "_")
                .substring(0, 31);
            utils.book_append_sheet(wb, ws, sheetName);
        }
        const today = new Date().toISOString().slice(0, 10);
        const selectedDept = departments.value.find(
            (d) => d.id === departmentFilter.value,
        );
        const deptSlug = selectedDept
            ? `_${selectedDept.name.replace(/[\/\\?*\[\]: ]/g, "_").replace(/_{2,}/g, "_")}`
            : "";
        const defaultName = `audit_notpass${deptSlug}_${today}.xlsx`;
        const savePath = await saveDialog({
            defaultPath: defaultName,
            filters: [{ name: "Excel", extensions: ["xlsx"] }],
        });
        if (!savePath) {
            exporting.value = false;
            return;
        }
        const xlsxBytes = xlsxWrite(wb, { type: "array", bookType: "xlsx" });
        await tauriWriteFile(savePath, new Uint8Array(xlsxBytes));
        exportMsg.value = `✓ ส่งออก ${rows.length} รายการสำเร็จ`;
    } catch {
        exportMsg.value = "✗ เกิดข้อผิดพลาดในการส่งออก";
    }
    await new Promise((r) => setTimeout(r, 1500));
    exporting.value = false;
}

onMounted(loadData);

// Expose for parent to trigger refresh
defineExpose({ refreshDepartments: loadData });
</script>

<style scoped>
.toast-fade-enter-active,
.toast-fade-leave-active {
    transition:
        opacity 0.25s ease,
        transform 0.25s ease;
}
.toast-fade-enter-from,
.toast-fade-leave-to {
    opacity: 0;
    transform: translateY(8px);
}
</style>

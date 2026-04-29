<template>
    <div class="flex h-full overflow-hidden">
        <!-- Left panel: report list -->
        <div class="w-72 flex flex-col border-r border-[#3A3A3B]">
            <!-- Header -->
            <div
                class="px-4 py-4 border-b border-[#3A3A3B] flex items-center gap-3"
            >
                <BarChart2 :size="22" class="page-header-icon" />
                <div>
                    <h2 class="text-sm font-bold text-[#E9ECEF]">Reports</h2>
                    <p class="text-xs text-[#888] mt-0.5">
                        รายงาน SQL สำหรับ HOSxP
                    </p>
                </div>
            </div>

            <!-- Filter -->
            <div class="px-3 py-2 border-b border-[#3A3A3B] space-y-2">
                <input
                    v-model="searchText"
                    class="input-field text-xs"
                    placeholder="ค้นหารายงาน..."
                />
                <div class="select-wrap w-full">
                    <select
                        v-model="departmentFilter"
                        class="text-xs"
                        @change="onDeptFilterChange"
                    >
                        <option :value="0">All Departments</option>
                        <option
                            v-for="d in departments"
                            :key="d.id"
                            :value="d.id"
                        >
                            {{ d.name }}
                        </option>
                    </select>
                </div>
            </div>

            <!-- Report list -->
            <div class="flex-1 overflow-y-auto">
                <button
                    v-for="report in filteredReports"
                    :key="report.id"
                    :class="[
                        'w-full text-left px-4 py-3 border-b border-[#3A3A3B]/50 transition-colors',
                        selectedReport?.id === report.id
                            ? 'bg-[#FF4D00]/15 border-l-2 border-l-[#FF4D00]'
                            : 'hover:bg-white/5',
                    ]"
                    @click="selectReport(report)"
                >
                    <div class="flex items-center gap-2">
                        <Star
                            v-if="report.is_starred"
                            :size="12"
                            class="text-[#FFB700] flex-shrink-0"
                            fill="#FFB700"
                        />
                        <span
                            class="text-sm font-medium text-[#E9ECEF] truncate"
                            >{{ report.name }}</span
                        >
                    </div>
                    <p
                        v-if="report.description"
                        class="text-xs text-[#888] truncate mt-0.5"
                    >
                        {{ report.description }}
                    </p>
                    <div class="mt-1">
                        <span
                            v-if="report.department_name"
                            class="badge-department text-[9px]"
                            >{{ report.department_name }}</span
                        >
                    </div>
                </button>
                <div
                    v-if="filteredReports.length === 0"
                    class="flex flex-col items-center justify-center h-40 text-[#555] text-xs gap-2"
                >
                    <BarChart2 :size="32" />
                    <span>ไม่พบรายงาน</span>
                </div>
            </div>
        </div>

        <!-- Right panel: result -->
        <div class="flex-1 flex flex-col overflow-hidden">
            <!-- Top bar -->
            <div
                class="px-5 py-4 border-b border-[#3A3A3B] flex items-center gap-3 flex-wrap"
            >
                <div class="flex-1 min-w-0">
                    <h3
                        v-if="selectedReport"
                        class="text-sm font-semibold text-[#E9ECEF] truncate"
                    >
                        {{ selectedReport.name }}
                    </h3>
                    <p v-else class="text-sm text-[#555]">
                        เลือกรายงานจากรายการทางซ้าย
                    </p>
                    <p
                        v-if="selectedReport?.description"
                        class="text-xs text-[#888] truncate"
                    >
                        {{ selectedReport.description }}
                    </p>
                </div>

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

                <!-- Run / Stop -->
                <button
                    v-if="!running"
                    class="btn-primary text-sm"
                    :disabled="!selectedReport || !dbConnected"
                    @click="runReport"
                >
                    <Play :size="14" /> Run Report
                </button>
                <button v-else class="btn-danger text-sm" @click="stop">
                    <Square :size="14" /> Stop
                </button>

                <!-- Export -->
                <button
                    v-if="resultRows.length > 0"
                    class="btn-secondary text-sm"
                    @click="exportCsv"
                >
                    <FileDown :size="14" /> Export Excel
                </button>
            </div>

            <!-- Filter result -->
            <div
                v-if="resultRows.length > 0"
                class="px-4 py-2 border-b border-[#3A3A3B] bg-[#1A1A1B]/30 flex items-center gap-2"
            >
                <Search :size="14" class="text-[#555]" />
                <input
                    v-model="resultFilter"
                    class="input-field flex-1 text-xs"
                    placeholder="Filter results..."
                />
                <span class="text-xs text-[#888]"
                    >{{ filteredResultRows.length }} /
                    {{ resultRows.length }} rows</span
                >
            </div>

            <!-- Status / Progress -->
            <div v-if="running" class="h-1 bg-[#1A1A1B]">
                <div class="h-1 bg-[#FF4D00] animate-pulse w-full"></div>
            </div>

            <div
                v-if="statusMsg"
                :class="[
                    'mx-4 mt-3 px-4 py-2.5 rounded-lg text-sm',
                    statusOk
                        ? 'bg-[#70E000]/15 text-[#70E000]'
                        : 'bg-[#EF233C]/15 text-[#EF233C]',
                ]"
            >
                {{ statusMsg }}
            </div>

            <!-- Result table -->
            <div class="flex-1 overflow-auto p-4">
                <div
                    v-if="!selectedReport"
                    class="flex flex-col items-center justify-center h-full text-[#555] gap-3"
                >
                    <BarChart2 :size="64" />
                    <p>เลือกรายงานเพื่อเริ่มต้น</p>
                </div>
                <div
                    v-else-if="resultRows.length === 0 && !running"
                    class="flex flex-col items-center justify-center h-full text-[#555] gap-3"
                >
                    <Table2 :size="48" />
                    <p>กด Run Report เพื่อดูผลลัพธ์</p>
                </div>
                <div v-else-if="resultRows.length > 0" class="table-container">
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th v-for="col in resultColumns" :key="col">
                                    {{ col }}
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="(row, i) in filteredResultRows" :key="i">
                                <td
                                    v-for="(cell, j) in row"
                                    :key="j"
                                    class="max-w-[250px] truncate"
                                    :title="String(cell ?? '')"
                                >
                                    {{ cell ?? "" }}
                                </td>
                            </tr>
                        </tbody>
                        <tfoot v-if="hasNumericColumns">
                            <tr
                                class="border-t-2 border-[#FF4D00]/60 bg-[#FF4D00]/10 font-semibold text-[#E9ECEF]"
                            >
                                <td
                                    v-for="(_col, j) in resultColumns"
                                    :key="j"
                                    class="px-3 py-2 text-right text-xs whitespace-nowrap"
                                >
                                    <template v-if="j === 0">
                                        รวมทั้งหมด ({{
                                            filteredResultRows.length
                                        }}
                                        แถว)
                                    </template>
                                    <template
                                        v-else-if="columnTotals[j] !== null"
                                    >
                                        {{
                                            columnTotals[j]!.toLocaleString(
                                                "th-TH",
                                                {
                                                    minimumFractionDigits: 2,
                                                    maximumFractionDigits: 2,
                                                },
                                            )
                                        }}
                                    </template>
                                    <template v-else> — </template>
                                </td>
                            </tr>
                        </tfoot>
                    </table>
                </div>
            </div>

            <!-- Status bar -->
            <div
                class="px-6 py-2 border-t border-[#3A3A3B] flex justify-between items-center text-xs text-[#555]"
            >
                <span
                    >พัฒนาโดย: กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์
                    ลพบุรี</span
                >
                <span v-if="lastElapsed"
                    >{{ resultRows.length.toLocaleString() }} rows ·
                    {{ lastElapsed.toFixed(3) }}s</span
                >
            </div>
        </div>

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
    Star,
    BarChart2,
    Play,
    Square,
    FileDown,
    Search,
    CalendarDays,
    Table2,
    Loader2,
} from "lucide-vue-next";
import CustomDatePicker from "./CustomDatePicker.vue";
import { utils as xlsxUtils, write as xlsxWrite } from "xlsx";
import { writeFile as tauriWriteFile } from "@tauri-apps/plugin-fs";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { api } from "../tauri-api";
import type { SqlQuery, Department, DbConfig } from "../types";

defineProps<{ dbConfig: DbConfig | null; dbConnected: boolean }>();

// ── Date helpers ──────────────────────────────────────────────────────────────
const now = new Date();
const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
const endOfMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);

/** Two-way binding for the DatePicker (Date objects) */
const dateFromObj = ref<Date | null>(startOfMonth);
const dateToObj = ref<Date | null>(endOfMonth);

/** Format a Date → "YYYY-MM-DD" string for the API */
function fmtDate(d: Date): string {
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

const dateFrom = computed(() =>
    dateFromObj.value ? fmtDate(dateFromObj.value) : "",
);
const dateTo = computed(() =>
    dateToObj.value ? fmtDate(dateToObj.value) : "",
);

// ── State ─────────────────────────────────────────────────────────────────----
const reports = ref<SqlQuery[]>([]);
const departments = ref<Department[]>([]);
const searchText = ref("");
const departmentFilter = ref<number>(0);
const selectedReport = ref<SqlQuery | null>(null);
const running = ref(false);
const stopRequested = ref(false);
const resultColumns = ref<string[]>([]);
const resultRows = ref<(string | number | null)[][]>([]);
const resultFilter = ref("");
const statusMsg = ref("");
const statusOk = ref(true);
const lastElapsed = ref<number | null>(null);
const exporting = ref(false);
const exportMsg = ref("");

// ── Computed ──────────────────────────────────────────────────────────────────
const filteredReports = computed(() => {
    return reports.value.filter((r) => {
        const matchText =
            !searchText.value ||
            r.name.toLowerCase().includes(searchText.value.toLowerCase());
        // departmentFilter === 0 → show all (including null dept)
        // departmentFilter > 0  → match by department_id
        const matchDept =
            departmentFilter.value === 0 ||
            r.department_id === departmentFilter.value;
        return matchText && matchDept;
    });
});

const filteredResultRows = computed(() => {
    if (!resultFilter.value) return resultRows.value;
    const f = resultFilter.value.toLowerCase();
    return resultRows.value.filter((row) =>
        row.some((cell) =>
            String(cell ?? "")
                .toLowerCase()
                .includes(f),
        ),
    );
});

/*
  Only sum money columns in the footer.
  - isMoneyColumn: lightweight heuristic that detects column names likely to be monetary
  - columnTotals: returns numeric totals only for money columns (null for others)
*/
const isMoneyColumn = (colName: string) => {
    if (!colName) return false;
    const n = colName.toLowerCase();
    return /price|total|amount|amt|cost|fee|nhso|balance|paid/.test(n);
};

const columnTotals = computed<(number | null)[]>(() => {
    if (resultColumns.value.length === 0) return [];
    return resultColumns.value.map((col, j) => {
        if (!isMoneyColumn(col)) return null;
        const values = filteredResultRows.value.map((row) => {
            const v = row[j];
            if (v === null || v === "") return NaN;
            const n = Number(String(v).replace(/,/g, ""));
            return n;
        });
        const allNumeric = values.length > 0 && values.every((n) => !isNaN(n));
        if (!allNumeric) return null;
        return values.reduce<number>((acc, n) => acc + n, 0);
    });
});

const hasNumericColumns = computed(() =>
    columnTotals.value.some((t) => t !== null),
);

// ── Load data ─────────────────────────────────────────────────────────────────
/**
 * Load enabled report queries for the currently selected department.
 * departmentFilter === 0 → returns ALL enabled reports (incl. NULL dept)
 * departmentFilter > 0   → returns only reports for that department
 */
async function loadReports() {
    const [reps, depts] = await Promise.all([
        api.getEnabledQueries("report", departmentFilter.value),
        api.getAllDepartments(),
    ]);
    reports.value = reps;
    departments.value = depts;
}

/** Called when the department dropdown changes */
async function onDeptFilterChange() {
    selectedReport.value = null;
    resultColumns.value = [];
    resultRows.value = [];
    statusMsg.value = "";
    resultFilter.value = "";
    await loadReports();
}

onMounted(loadReports);

// ── Actions ───────────────────────────────────────────────────────────────────
function selectReport(report: SqlQuery) {
    selectedReport.value = report;
    resultColumns.value = [];
    resultRows.value = [];
    statusMsg.value = "";
    resultFilter.value = "";
}

async function runReport() {
    if (!selectedReport.value) return;
    running.value = true;
    stopRequested.value = false;
    statusMsg.value = "";
    resultColumns.value = [];
    resultRows.value = [];

    const t0 = performance.now();
    try {
        const result = await api.executeQuery(
            selectedReport.value.sql_text,
            dateFrom.value,
            dateTo.value,
        );
        if (!stopRequested.value) {
            resultColumns.value = result.columns;
            resultRows.value = result.rows;
            lastElapsed.value = result.elapsed_sec;
            statusMsg.value = `สำเร็จ: ${result.row_count.toLocaleString()} แถว ใช้เวลา ${result.elapsed_sec.toFixed(3)} วินาที`;
            statusOk.value = true;
            api.logExecution({
                queryId: selectedReport.value.id,
                queryName: selectedReport.value.name,
                mode: "report",
                dateFrom: dateFrom.value,
                dateTo: dateTo.value,
                rowCount: result.row_count,
                elapsedSec: result.elapsed_sec,
                status: "ok",
                errorMsg: "",
            }).catch(() => {});
        }
    } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        statusMsg.value = `ข้อผิดพลาด: ${msg}`;
        statusOk.value = false;
        api.logExecution({
            queryId: selectedReport.value!.id,
            queryName: selectedReport.value!.name,
            mode: "report",
            dateFrom: dateFrom.value,
            dateTo: dateTo.value,
            rowCount: 0,
            elapsedSec: (performance.now() - t0) / 1000,
            status: "error",
            errorMsg: msg,
        }).catch(() => {});
    } finally {
        running.value = false;
    }
}

function stop() {
    stopRequested.value = true;
    running.value = false;
}

// ── Export ────────────────────────────────────────────────────────────────────

/** Build info header rows prepended to the Excel sheet */
function buildInfoRows(name: string, description: string): (string | null)[][] {
    return [
        ["ชื่อ:", name],
        ["คำอธิบาย:", description || "-"],
        ["ช่วงวันที่:", `${dateFrom.value} ถึง ${dateTo.value}`],
        [],
    ];
}

async function exportCsv() {
    if (resultColumns.value.length === 0) return;
    const reportName = selectedReport.value?.name ?? "report";
    const reportDesc = selectedReport.value?.description ?? "";
    exportMsg.value = `กำลังส่งออก "${reportName}"...`;
    exporting.value = true;
    await nextTick();
    await new Promise((r) => setTimeout(r, 50));
    try {
        const wb = xlsxUtils.book_new();
        const infoRows = buildInfoRows(reportName, reportDesc);
        const wsData = [...infoRows, resultColumns.value, ...resultRows.value];
        const ws = xlsxUtils.aoa_to_sheet(wsData);
        const sheetName = reportName
            .replace(/[\\/:*?[\]]/g, "")
            .substring(0, 31);
        xlsxUtils.book_append_sheet(wb, ws, sheetName || "Report");
        const dateStr =
            dateFrom.value === dateTo.value
                ? dateFrom.value.replace(/-/g, "")
                : `${dateFrom.value.replace(/-/g, "")}-${dateTo.value.replace(/-/g, "")}`;
        const defaultName = `${reportName.replace(/[\s\/\\?*\[\]:]/g, "_")}_${dateStr}.xlsx`;
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
        exportMsg.value = `✓ ส่งออก "${reportName}" สำเร็จ`;
    } catch {
        exportMsg.value = "✗ เกิดข้อผิดพลาดในการส่งออก";
    }
    await new Promise((r) => setTimeout(r, 1500));
    exporting.value = false;
}

// Expose for parent to trigger refresh
defineExpose({ refreshDepartments: loadReports });
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

<template>
    <div class="flex flex-col h-full">
        <!-- Header -->
        <div class="page-header">
            <Settings2 :size="22" class="page-header-icon" />
            <div>
                <h1 class="text-base font-bold text-[#E9ECEF]">Settings</h1>
                <p class="text-xs text-[#888]">
                    ตั้งค่าการเชื่อมต่อฐานข้อมูล HOSxP
                </p>
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto p-6">
            <div class="max-w-lg">
                <div class="card p-6 space-y-5">
                    <h2
                        class="text-sm font-semibold text-[#FFB700] uppercase tracking-wider flex items-center gap-2"
                    >
                        <Database :size="16" /> Database Connection
                    </h2>

                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-xs text-[#aaa] mb-1.5 font-medium"
                                >Host / IP Address</label
                            >
                            <input
                                v-model="form.host"
                                class="input-field"
                                placeholder="localhost"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-xs text-[#aaa] mb-1.5 font-medium"
                                >Port</label
                            >
                            <input
                                v-model.number="form.port"
                                type="number"
                                class="input-field"
                                placeholder="3306"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-xs text-[#aaa] mb-1.5 font-medium"
                                >Username</label
                            >
                            <input
                                v-model="form.user"
                                class="input-field"
                                placeholder="root"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-xs text-[#aaa] mb-1.5 font-medium"
                                >Password</label
                            >
                            <input
                                v-model="form.password"
                                type="password"
                                class="input-field"
                                placeholder="••••••••"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-xs text-[#aaa] mb-1.5 font-medium"
                                >Database Name</label
                            >
                            <input
                                v-model="form.database"
                                class="input-field"
                                placeholder="hosxp_pcu"
                            />
                        </div>
                    </div>

                    <!-- Test result -->
                    <div
                        v-if="testResult"
                        :class="[
                            'text-sm px-4 py-3 rounded-lg',
                            testOk
                                ? 'bg-[#70E000]/15 text-[#70E000]'
                                : 'bg-[#EF233C]/15 text-[#EF233C]',
                        ]"
                    >
                        {{ testResult }}
                    </div>

                    <!-- Buttons -->
                    <div class="flex gap-3 pt-2">
                        <button
                            class="btn-secondary flex-1"
                            :disabled="testing"
                            @click="testConn"
                        >
                            <Wifi :size="16" />
                            {{ testing ? "กำลังทดสอบ..." : "Test Connection" }}
                        </button>
                        <button class="btn-primary flex-1" @click="save">
                            <Save :size="16" /> Save Settings
                        </button>
                    </div>
                </div>

                <!-- Info card -->
                <div class="mt-4 card p-4">
                    <p class="text-xs text-[#888] leading-relaxed">
                        <span class="text-[#FFB700] font-semibold"
                            >ℹ️ หมายเหตุ:</span
                        >
                        การเชื่อมต่อจะใช้ mysql/mariadb driver โดยตรง
                        รหัสผ่านจะถูกจัดเก็บในเครื่องของคุณ
                    </p>
                </div>

                <!-- Import/Export JSON -->
                <div class="mt-6 card p-6 space-y-5">
                    <h2
                        class="text-sm font-semibold text-[#FFB700] uppercase tracking-wider flex items-center gap-2"
                    >
                        <ArrowLeftRight :size="16" /> นำเข้าและส่งออกข้อมูล
                        (JSON)
                    </h2>

                    <!-- EXPORT -->
                    <div class="space-y-3">
                        <h3
                            class="text-xs font-semibold text-[#E9ECEF] uppercase tracking-wider flex items-center gap-1.5"
                        >
                            <Upload :size="13" class="text-[#FFB700]" />
                            ส่งออกข้อมูล (Export)
                        </h3>

                        <!-- Type selector -->
                        <div class="flex gap-4 flex-wrap">
                            <label
                                class="flex items-center gap-2 cursor-pointer select-none text-xs text-[#ccc]"
                            >
                                <input
                                    type="checkbox"
                                    v-model="exportAudit"
                                    class="accent-[#FFB700] w-3.5 h-3.5"
                                    @change="onExportTypeChange"
                                />
                                <Search :size="13" class="text-[#FFB700]" />
                                Audit Queries
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer select-none text-xs text-[#ccc]"
                            >
                                <input
                                    type="checkbox"
                                    v-model="exportReport"
                                    class="accent-[#4CC9F0] w-3.5 h-3.5"
                                    @change="onExportTypeChange"
                                />
                                <BarChart2 :size="13" class="text-[#4CC9F0]" />
                                Report Queries
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer select-none text-xs text-[#ccc]"
                            >
                                <input
                                    type="checkbox"
                                    v-model="exportDepartments"
                                    class="accent-[#70E000] w-3.5 h-3.5"
                                    @change="onExportTypeChange"
                                />
                                <Database :size="13" class="text-[#70E000]" />
                                Departments
                            </label>
                        </div>

                        <!-- Loading indicator -->
                        <div
                            v-if="loadingExportList"
                            class="text-xs text-[#888] flex items-center gap-2 py-1"
                        >
                            <Loader2 :size="13" class="animate-spin" />
                            กำลังโหลดรายการ...
                        </div>

                        <!-- Query list -->
                        <div
                            v-if="
                                !loadingExportList &&
                                exportableQueries.length > 0
                            "
                            class="border border-[#3A3A3B] rounded-lg overflow-hidden"
                        >
                            <!-- Audit group -->
                            <div v-if="exportAudit && auditQueries.length > 0">
                                <div
                                    class="flex items-center justify-between px-3 py-2 bg-[#2A2A2B] border-b border-[#3A3A3B]"
                                >
                                    <span
                                        class="text-xs font-semibold text-[#FFB700] flex items-center gap-1.5"
                                    >
                                        <Search :size="12" /> Audit ({{
                                            auditQueries.length
                                        }})
                                    </span>
                                    <label
                                        class="flex items-center gap-1.5 text-xs text-[#aaa] cursor-pointer select-none"
                                    >
                                        <input
                                            type="checkbox"
                                            :checked="allAuditSelected"
                                            :indeterminate="
                                                someAuditSelected &&
                                                !allAuditSelected
                                            "
                                            class="accent-[#FFB700] w-3 h-3"
                                            @change="toggleSelectAllAudit"
                                        />
                                        เลือกทั้งหมด
                                    </label>
                                </div>
                                <div class="max-h-45 overflow-y-auto">
                                    <label
                                        v-for="q in auditQueries"
                                        :key="q.id"
                                        class="flex items-center gap-2.5 px-3 py-1.5 hover:bg-white/5 cursor-pointer border-b border-[#2A2A2B] last:border-0"
                                    >
                                        <input
                                            type="checkbox"
                                            :value="q.id"
                                            v-model="selectedExportIds"
                                            class="accent-[#FFB700] w-3 h-3 shrink-0"
                                        />
                                        <span
                                            class="text-xs text-[#ccc] truncate"
                                            >{{ q.name }}</span
                                        >
                                    </label>
                                </div>
                            </div>

                            <!-- Report group -->
                            <div
                                v-if="exportReport && reportQueries.length > 0"
                            >
                                <div
                                    class="flex items-center justify-between px-3 py-2 bg-[#2A2A2B] border-b border-[#3A3A3B]"
                                    :class="{
                                        'border-t':
                                            exportAudit &&
                                            auditQueries.length > 0,
                                    }"
                                >
                                    <span
                                        class="text-xs font-semibold text-[#4CC9F0] flex items-center gap-1.5"
                                    >
                                        <BarChart2 :size="12" /> Report ({{
                                            reportQueries.length
                                        }})
                                    </span>
                                    <label
                                        class="flex items-center gap-1.5 text-xs text-[#aaa] cursor-pointer select-none"
                                    >
                                        <input
                                            type="checkbox"
                                            :checked="allReportSelected"
                                            :indeterminate="
                                                someReportSelected &&
                                                !allReportSelected
                                            "
                                            class="accent-[#4CC9F0] w-3 h-3"
                                            @change="toggleSelectAllReport"
                                        />
                                        เลือกทั้งหมด
                                    </label>
                                </div>
                                <div class="max-h-45 overflow-y-auto">
                                    <label
                                        v-for="q in reportQueries"
                                        :key="q.id"
                                        class="flex items-center gap-2.5 px-3 py-1.5 hover:bg-white/5 cursor-pointer border-b border-[#2A2A2B] last:border-0"
                                    >
                                        <input
                                            type="checkbox"
                                            :value="q.id"
                                            v-model="selectedExportIds"
                                            class="accent-[#4CC9F0] w-3 h-3 shrink-0"
                                        />
                                        <span
                                            class="text-xs text-[#ccc] truncate"
                                            >{{ q.name }}</span
                                        >
                                    </label>
                                </div>
                            </div>
                        </div>

                        <div
                            v-if="
                                !loadingExportList &&
                                (exportAudit || exportReport) &&
                                exportableQueries.length === 0
                            "
                            class="text-xs text-[#888] italic py-1"
                        >
                            ไม่พบข้อมูล Query ที่จะส่งออก
                        </div>

                        <div
                            v-if="exportDepartments"
                            class="text-xs text-[#aaa] leading-relaxed"
                        >
                            Departments จะถูกส่งออกทั้งหมด
                            {{ departments.length }} รายการ
                        </div>

                        <!-- Export result -->
                        <div
                            v-if="exportResult"
                            :class="[
                                'text-sm px-4 py-3 rounded-lg',
                                exportOk
                                    ? 'bg-[#70E000]/15 text-[#70E000]'
                                    : 'bg-[#EF233C]/15 text-[#EF233C]',
                            ]"
                        >
                            {{ exportResult }}
                        </div>

                        <button
                            class="btn-primary text-sm"
                            :disabled="
                                (selectedExportIds.length === 0 &&
                                    !exportDepartments) ||
                                exporting
                            "
                            @click="doExport"
                        >
                            <Upload :size="15" />
                            {{
                                exporting
                                    ? "กำลังส่งออก..."
                                    : `Export (${selectedExportIds.length} Queries${exportDepartments ? ", Departments" : ""})`
                            }}
                        </button>
                    </div>

                    <hr class="border-[#3A3A3B]" />

                    <!-- IMPORT -->
                    <div class="space-y-3">
                        <h3
                            class="text-xs font-semibold text-[#E9ECEF] uppercase tracking-wider flex items-center gap-1.5"
                        >
                            <Download :size="13" class="text-[#FFB700]" />
                            นำเข้าข้อมูล (Import)
                        </h3>

                        <p class="text-xs text-[#888] leading-relaxed">
                            นำเข้า Query และ Departments จากไฟล์ JSON
                            ที่ส่งออกจาก SmartQuery v3.0.0
                        </p>

                        <div class="flex gap-3 items-center">
                            <input
                                v-model="importFilePath"
                                class="input-field flex-1 text-xs"
                                placeholder="เส้นทางไฟล์ .json ..."
                                readonly
                            />
                            <button
                                class="btn-secondary text-sm shrink-0"
                                @click="browseImportFile"
                            >
                                <FolderOpen :size="14" /> Browse
                            </button>
                        </div>

                        <!-- Import result -->
                        <div
                            v-if="importResult"
                            :class="[
                                'text-sm px-4 py-3 rounded-lg',
                                importOk
                                    ? 'bg-[#70E000]/15 text-[#70E000]'
                                    : 'bg-[#EF233C]/15 text-[#EF233C]',
                            ]"
                        >
                            {{ importResult }}
                        </div>

                        <button
                            class="btn-warning text-sm"
                            :disabled="!importFilePath.trim() || importing"
                            @click="doImport"
                        >
                            <Download :size="15" />
                            {{ importing ? "กำลังนำเข้า..." : "Import Data" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Status bar -->
        <div
            class="px-6 py-2 border-t border-[#3A3A3B] flex justify-between items-center text-xs text-[#555]"
        >
            <span>พัฒนาโดย: กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์ ลพบุรี</span>
            <span>SmartQuery for HOSxP v3.0.0</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
    Database,
    Wifi,
    Save,
    FolderOpen,
    Download,
    Upload,
    Search,
    BarChart2,
    ArrowLeftRight,
    Loader2,
    Settings2,
} from "lucide-vue-next";
import {
    open as dialogOpen,
    save as dialogSave,
} from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { api } from "../tauri-api";
import type { DbConfig, SqlQuery, Department } from "../types";

const emit = defineEmits<{ saved: [cfg: DbConfig] }>();

// ---- DB Config ----
const form = ref<DbConfig>({
    host: "localhost",
    port: 3306,
    user: "",
    password: "",
    database: "hosxp_pcu",
});
const testing = ref(false);
const testResult = ref("");
const testOk = ref(false);

// ---- Export ----
const exportAudit = ref(false);
const exportReport = ref(false);
const exportDepartments = ref(false);

const loadingExportList = ref(false);
const auditQueries = ref<SqlQuery[]>([]);
const reportQueries = ref<SqlQuery[]>([]);
const departments = ref<Department[]>([]);
const selectedExportIds = ref<number[]>([]);
const exporting = ref(false);
const exportResult = ref("");
const exportOk = ref(false);

// ---- Import ----
const importFilePath = ref("");
const importing = ref(false);
const importResult = ref("");
const importOk = ref(false);

// ---- Computed ----
const exportableQueries = computed(() => [
    ...auditQueries.value,
    ...reportQueries.value,
]);

const allAuditSelected = computed(
    () =>
        auditQueries.value.length > 0 &&
        auditQueries.value.every((q) => selectedExportIds.value.includes(q.id)),
);
const someAuditSelected = computed(() =>
    auditQueries.value.some((q) => selectedExportIds.value.includes(q.id)),
);
const allReportSelected = computed(
    () =>
        reportQueries.value.length > 0 &&
        reportQueries.value.every((q) =>
            selectedExportIds.value.includes(q.id),
        ),
);
const someReportSelected = computed(() =>
    reportQueries.value.some((q) => selectedExportIds.value.includes(q.id)),
);

// ---- Lifecycle ----
onMounted(async () => {
    try {
        const cfg = await api.loadDbConfig();
        form.value = { ...cfg };
    } catch {
        /* ignore */
    }
});

// ---- DB Config functions ----
async function testConn() {
    testing.value = true;
    testResult.value = "";
    try {
        const msg = await api.testConnection(form.value);
        testResult.value = msg;
        testOk.value = true;
    } catch (e: any) {
        testResult.value = String(e);
        testOk.value = false;
    } finally {
        testing.value = false;
    }
}

async function save() {
    await api.saveDbConfig(form.value);
    try {
        await api.connectDb(form.value);
        testResult.value = "บันทึกและเชื่อมต่อสำเร็จ";
        testOk.value = true;
        emit("saved", { ...form.value });
    } catch (e: any) {
        testResult.value = `บันทึกแล้ว แต่เชื่อมต่อล้มเหลว: ${e}`;
        testOk.value = false;
    }
}

// ---- Export functions ----
async function onExportTypeChange() {
    exportResult.value = "";
    loadingExportList.value = true;
    try {
        const previousAuditIds = auditQueries.value.map((q) => q.id);
        const previousReportIds = reportQueries.value.map((q) => q.id);

        if (exportAudit.value) {
            auditQueries.value = await api.getAllQueries("audit");
        } else {
            auditQueries.value = [];
            selectedExportIds.value = selectedExportIds.value.filter(
                (id) => !previousAuditIds.includes(id),
            );
        }

        if (exportReport.value) {
            reportQueries.value = await api.getAllQueries("report");
        } else {
            reportQueries.value = [];
            selectedExportIds.value = selectedExportIds.value.filter(
                (id) => !previousReportIds.includes(id),
            );
        }

        if (exportDepartments.value) {
            departments.value = await api.getAllDepartments();
        } else {
            departments.value = [];
        }

        const allIds = exportableQueries.value.map((q) => q.id);
        selectedExportIds.value = selectedExportIds.value.filter((id) =>
            allIds.includes(id),
        );
    } catch (e: any) {
        exportResult.value = `โหลดรายการล้มเหลว: ${String(e)}`;
        exportOk.value = false;
    } finally {
        loadingExportList.value = false;
    }
}

function toggleSelectAllAudit(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    const auditIds = auditQueries.value.map((q) => q.id);
    if (checked) {
        const existing = new Set(selectedExportIds.value);
        auditIds.forEach((id) => existing.add(id));
        selectedExportIds.value = Array.from(existing);
    } else {
        selectedExportIds.value = selectedExportIds.value.filter(
            (id) => !auditIds.includes(id),
        );
    }
}

function toggleSelectAllReport(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    const reportIds = reportQueries.value.map((q) => q.id);
    if (checked) {
        const existing = new Set(selectedExportIds.value);
        reportIds.forEach((id) => existing.add(id));
        selectedExportIds.value = Array.from(existing);
    } else {
        selectedExportIds.value = selectedExportIds.value.filter(
            (id) => !reportIds.includes(id),
        );
    }
}

async function doExport() {
    const shouldExportQueries = selectedExportIds.value.length > 0;
    const shouldExportDepartments = exportDepartments.value;
    if (!shouldExportQueries && !shouldExportDepartments) {
        return;
    }

    exporting.value = true;
    exportResult.value = "";
    try {
        const selectedSet = new Set(selectedExportIds.value);
        const queriesToExport = exportableQueries.value
            .filter((q) => selectedSet.has(q.id))
            .map((q) => ({
                id: q.id,
                name: q.name,
                description: q.description,
                sql_text: q.sql_text,
                mode: q.mode,
                department_name: q.department_name ?? "",
                is_starred: q.is_starred,
                enabled: q.enabled,
            }));

        const payload = {
            version: "3.0.0",
            exported_at: new Date().toISOString(),

            departments: shouldExportDepartments
                ? departments.value.map((d) => ({
                      id: d.id,
                      name: d.name,
                      sort_order: d.sort_order,
                  }))
                : [],
            queries: queriesToExport,
        };

        const savePath = await dialogSave({
            defaultPath: "smartquery_export.json",
            filters: [{ name: "JSON File", extensions: ["json"] }],
        });

        if (!savePath) {
            exporting.value = false;
            return;
        }

        await writeTextFile(savePath, JSON.stringify(payload, null, 2));

        const summary = [
            shouldExportQueries ? `${queriesToExport.length} queries` : null,
            shouldExportDepartments
                ? `${payload.departments.length} departments`
                : null,
        ]
            .filter(Boolean)
            .join(", ");

        exportResult.value = `ส่งออกสำเร็จ ${summary} → ${savePath}`;
        exportOk.value = true;
    } catch (e: any) {
        exportResult.value = `ส่งออกล้มเหลว: ${String(e)}`;
        exportOk.value = false;
    } finally {
        exporting.value = false;
    }
}

// ---- Import functions ----
async function browseImportFile() {
    try {
        const selected = await dialogOpen({
            filters: [{ name: "JSON File", extensions: ["json"] }],
            multiple: false,
        });
        if (selected && typeof selected === "string") {
            importFilePath.value = selected;
            importResult.value = "";
        }
    } catch (e) {
        console.error(e);
    }
}

async function doImport() {
    if (!importFilePath.value.trim()) return;
    importing.value = true;
    importResult.value = "";
    try {
        const raw = await readTextFile(importFilePath.value.trim());
        const data = JSON.parse(raw);

        const hasQueries = Array.isArray(data.queries);
        const hasDepartments = Array.isArray(data.departments);
        if (!hasQueries && !hasDepartments) {
            throw new Error(
                "รูปแบบไฟล์ไม่ถูกต้อง: ไม่พบ queries หรือ departments",
            );
        }

        const departmentNameToId = new Map<string, number>();
        let importedDepartments = 0;
        let importedQueries = 0;
        let failedQueries = 0;

        if (hasDepartments) {
            const existingDepartments = await api.getAllDepartments();
            for (const dept of existingDepartments) {
                departmentNameToId.set(dept.name.trim(), dept.id);
            }

            for (const dept of data.departments) {
                const deptName = String(dept?.name ?? "").trim();
                if (!deptName || departmentNameToId.has(deptName)) continue;

                try {
                    const newDeptId = await api.insertDepartment(deptName);
                    departmentNameToId.set(deptName, newDeptId);
                    importedDepartments++;
                } catch {
                    /* ignore duplicate/invalid department */
                }
            }
        }

        if (hasQueries) {
            if (departmentNameToId.size === 0) {
                const existingDepartments = await api.getAllDepartments();
                for (const dept of existingDepartments) {
                    departmentNameToId.set(dept.name.trim(), dept.id);
                }
            }

            for (const q of data.queries) {
                try {
                    const departmentName = String(
                        q.department_name ?? "",
                    ).trim();
                    const departmentId = departmentName
                        ? (departmentNameToId.get(departmentName) ?? null)
                        : null;

                    await api.insertQuery(
                        q.mode ?? "audit",
                        q.name ?? "Untitled",
                        q.description ?? "",
                        q.sql_text ?? "",
                        departmentId,
                        Boolean(q.is_starred),
                    );
                    importedQueries++;
                } catch {
                    failedQueries++;
                }
            }
        }

        importOk.value = failedQueries === 0;
        importResult.value = [
            hasDepartments ? `Departments ${importedDepartments} รายการ` : null,
            hasQueries
                ? failedQueries === 0
                    ? `Queries ${importedQueries} รายการ`
                    : `Queries สำเร็จ ${importedQueries} รายการ, ล้มเหลว ${failedQueries} รายการ`
                : null,
        ]
            .filter(Boolean)
            .join(" | ");
    } catch (e: any) {
        importResult.value = `นำเข้าล้มเหลว: ${e instanceof Error ? e.message : String(e)}`;
        importOk.value = false;
    } finally {
        importing.value = false;
    }
}
</script>

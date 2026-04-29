<template>
    <div class="flex flex-col h-full">
        <!-- Header -->
        <div class="page-header">
            <Database :size="22" class="page-header-icon" />
            <div>
                <h1 class="text-base font-bold text-[#E9ECEF]">
                    SQL Management
                </h1>
                <p class="text-xs text-[#888]">จัดการ Query และ Report</p>
            </div>
            <div class="ml-auto flex items-center gap-2">
                <button class="btn-primary text-sm" @click="addQuery">
                    <Plus :size="15" /> Add Query
                </button>
                <button
                    class="btn-secondary text-sm"
                    @click="manageDepartments"
                >
                    <Building2 :size="15" /> Manage Departments
                </button>
            </div>
        </div>

        <!-- Tabs + Filters -->
        <div
            class="px-4 py-3 border-b border-[#3A3A3B] flex items-center gap-3 flex-wrap"
        >
            <!-- Mode tabs -->
            <div
                class="flex rounded-lg overflow-hidden border border-[#3A3A3B]"
            >
                <button
                    :class="[
                        'px-4 py-1.5 text-sm font-medium transition-colors',
                        mode === 'audit'
                            ? 'bg-[#FF4D00] text-white'
                            : 'text-[#aaa] hover:bg-white/5',
                    ]"
                    @click="setMode('audit')"
                >
                    Audit
                </button>
                <button
                    :class="[
                        'px-4 py-1.5 text-sm font-medium transition-colors',
                        mode === 'report'
                            ? 'bg-[#FF4D00] text-white'
                            : 'text-[#aaa] hover:bg-white/5',
                    ]"
                    @click="setMode('report')"
                >
                    Report
                </button>
            </div>

            <input
                v-model="searchText"
                class="input-field w-56 text-xs"
                placeholder="ค้นหา..."
                @input="reload"
            />

            <!-- Department filter -->
            <div class="select-wrap w-44">
                <select
                    v-model="departmentFilter"
                    class="text-xs"
                    @change="reload"
                >
                    <option value="0">All Departments</option>
                    <option v-for="d in departments" :key="d.id" :value="d.id">
                        {{ d.name }}
                    </option>
                </select>
            </div>

            <div class="select-wrap w-28">
                <select v-model="statusFilter" class="text-xs" @change="reload">
                    <option value="all">All</option>
                    <option value="enabled">Enabled</option>
                    <option value="disabled">Disabled</option>
                </select>
            </div>

            <label
                class="flex items-center gap-2 text-xs text-[#aaa] cursor-pointer"
            >
                <input
                    type="checkbox"
                    v-model="starredOnly"
                    @change="reload"
                    class="accent-[#FFB700]"
                />
                <Star :size="13" class="text-[#FFB700]" /> Starred only
            </label>

            <button class="btn-ghost text-xs ml-auto" @click="clearFilters">
                <X :size="13" /> Clear Filters
            </button>
        </div>

        <!-- Table -->
        <div class="flex-1 overflow-auto">
            <table class="data-table w-full">
                <thead>
                    <tr>
                        <th class="w-8">#</th>
                        <th
                            class="cursor-pointer select-none hover:text-[#E9ECEF] transition-colors"
                            @click="setSort('name')"
                        >
                            <span class="flex items-center gap-1">
                                Name
                                <span class="text-[10px] opacity-60">
                                    <span v-if="sortKey === 'name'">{{
                                        sortDir === "asc" ? "▲" : "▼"
                                    }}</span>
                                    <span v-else class="opacity-40">⇅</span>
                                </span>
                            </span>
                        </th>
                        <th
                            class="cursor-pointer select-none hover:text-[#E9ECEF] transition-colors"
                            @click="setSort('description')"
                        >
                            <span class="flex items-center gap-1">
                                Description
                                <span class="text-[10px] opacity-60">
                                    <span v-if="sortKey === 'description'">{{
                                        sortDir === "asc" ? "▲" : "▼"
                                    }}</span>
                                    <span v-else class="opacity-40">⇅</span>
                                </span>
                            </span>
                        </th>
                        <th
                            class="cursor-pointer select-none hover:text-[#E9ECEF] transition-colors"
                            @click="setSort('department_name')"
                        >
                            <span class="flex items-center gap-1">
                                Department
                                <span class="text-[10px] opacity-60">
                                    <span
                                        v-if="sortKey === 'department_name'"
                                        >{{
                                            sortDir === "asc" ? "▲" : "▼"
                                        }}</span
                                    >
                                    <span v-else class="opacity-40">⇅</span>
                                </span>
                            </span>
                        </th>
                        <th
                            class="w-20 text-center cursor-pointer select-none hover:text-[#E9ECEF] transition-colors"
                            @click="setSort('enabled')"
                        >
                            <span
                                class="flex items-center justify-center gap-1"
                            >
                                Enabled
                                <span class="text-[10px] opacity-60">
                                    <span v-if="sortKey === 'enabled'">{{
                                        sortDir === "asc" ? "▲" : "▼"
                                    }}</span>
                                    <span v-else class="opacity-40">⇅</span>
                                </span>
                            </span>
                        </th>
                        <th
                            class="w-14 text-center cursor-pointer select-none hover:text-[#E9ECEF] transition-colors"
                            @click="setSort('is_starred')"
                        >
                            <span
                                class="flex items-center justify-center gap-1"
                            >
                                Star
                                <span class="text-[10px] opacity-60">
                                    <span v-if="sortKey === 'is_starred'">{{
                                        sortDir === "asc" ? "▲" : "▼"
                                    }}</span>
                                    <span v-else class="opacity-40">⇅</span>
                                </span>
                            </span>
                        </th>
                        <th class="w-40 text-right">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(q, i) in sortedQueries" :key="q.id">
                        <td class="text-[#555] text-xs">{{ i + 1 }}</td>
                        <td
                            class="font-medium max-w-[200px] truncate"
                            :title="q.name"
                        >
                            {{ q.name }}
                        </td>
                        <td
                            class="text-xs text-[#888] max-w-[200px] truncate"
                            :title="q.description"
                        >
                            {{ q.description || "—" }}
                        </td>
                        <td>
                            <span
                                v-if="q.department_name"
                                class="badge-department text-[10px]"
                            >
                                {{ q.department_name }}
                            </span>
                            <span v-else class="text-[#555] text-xs">—</span>
                        </td>
                        <td class="text-center">
                            <button
                                @click="toggleEnabled(q)"
                                :class="
                                    q.enabled ? 'text-[#70E000]' : 'text-[#555]'
                                "
                            >
                                <ToggleRight v-if="q.enabled" :size="20" />
                                <ToggleLeft v-else :size="20" />
                            </button>
                        </td>
                        <td class="text-center">
                            <button
                                @click="toggleStar(q)"
                                :class="
                                    q.is_starred
                                        ? 'text-[#FFB700]'
                                        : 'text-[#555] hover:text-[#FFB700]'
                                "
                            >
                                <Star
                                    :size="15"
                                    :fill="q.is_starred ? '#FFB700' : 'none'"
                                />
                            </button>
                        </td>
                        <td class="text-right">
                            <div class="flex items-center justify-end gap-1">
                                <button
                                    class="btn-ghost text-xs px-2 py-1"
                                    @click="editQuery(q)"
                                >
                                    <Pencil :size="13" /> Edit
                                </button>
                                <button
                                    class="btn-ghost text-xs px-2 py-1"
                                    @click="testQuery(q)"
                                >
                                    <Play :size="13" /> Test
                                </button>
                                <button
                                    class="btn-danger text-xs px-2 py-1"
                                    @click="deleteQuery(q)"
                                >
                                    <Trash2 :size="13" />
                                </button>
                            </div>
                        </td>
                    </tr>
                    <tr v-if="sortedQueries.length === 0">
                        <td colspan="7" class="text-center py-12 text-[#555]">
                            ไม่พบ Query
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- Status bar -->
        <div
            class="px-6 py-2 border-t border-[#3A3A3B] flex justify-between items-center text-xs text-[#555]"
        >
            <span>พัฒนาโดย: กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์ ลพบุรี</span>
            <span>{{ sortedQueries.length }} queries</span>
        </div>

        <!-- Query Editor Modal -->
        <QueryEditorModal
            v-if="showEditor"
            :query="editingQuery"
            :mode="mode"
            :departments="departments"
            :db-config="dbConfig"
            :db-connected="dbConnected"
            @close="showEditor = false"
            @saved="onSaved"
        />

        <!-- Departments Manager Modal -->
        <DepartmentModal
            v-if="showDeptModal"
            :departments="departments"
            @close="closeDeptModal"
            @changed="reloadDepartments"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
    Plus,
    Building2,
    Star,
    X,
    Pencil,
    Trash2,
    Play,
    ToggleLeft,
    ToggleRight,
    Database,
} from "lucide-vue-next";
import QueryEditorModal from "./QueryEditorModal.vue";
import DepartmentModal from "./DepartmentModal.vue";
import { api } from "../tauri-api";
import type { SqlQuery, Department, DbConfig } from "../types";

defineProps<{ dbConfig: DbConfig | null; dbConnected: boolean }>();

const mode = ref<"audit" | "report">("audit");
const queries = ref<SqlQuery[]>([]);
const departments = ref<Department[]>([]);
const searchText = ref("");
const departmentFilter = ref<number>(0);
const statusFilter = ref("all");
const starredOnly = ref(false);
const showEditor = ref(false);
const editingQuery = ref<SqlQuery | null>(null);
const showDeptModal = ref(false);

// Sorting
type SortKey =
    | "name"
    | "description"
    | "department_name"
    | "enabled"
    | "is_starred";
const sortKey = ref<SortKey | "">("");
const sortDir = ref<"asc" | "desc">("asc");

const sortedQueries = computed(() => {
    if (!sortKey.value) return queries.value;
    const key = sortKey.value;
    return [...queries.value].sort((a, b) => {
        let valA: string | number = a[key] ?? "";
        let valB: string | number = b[key] ?? "";
        if (typeof valA === "number" && typeof valB === "number") {
            return sortDir.value === "asc" ? valA - valB : valB - valA;
        }
        valA = String(valA).toLowerCase();
        valB = String(valB).toLowerCase();
        if (valA < valB) return sortDir.value === "asc" ? -1 : 1;
        if (valA > valB) return sortDir.value === "asc" ? 1 : -1;
        return 0;
    });
});

function setSort(key: SortKey) {
    if (sortKey.value === key) {
        sortDir.value = sortDir.value === "asc" ? "desc" : "asc";
    } else {
        sortKey.value = key;
        sortDir.value = "asc";
    }
}

onMounted(async () => {
    await reloadDepartments();
    await reload();
});

async function reload() {
    queries.value = await api.searchQueries(
        mode.value,
        searchText.value,
        statusFilter.value,
        departmentFilter.value,
        starredOnly.value,
    );
}

async function reloadDepartments() {
    departments.value = await api.getAllDepartments();
}

function setMode(m: "audit" | "report") {
    mode.value = m;
    reload();
}

function clearFilters() {
    searchText.value = "";
    departmentFilter.value = 0;
    statusFilter.value = "all";
    starredOnly.value = false;
    reload();
}

async function toggleEnabled(q: SqlQuery) {
    const newVal = !q.enabled;
    await api.setQueryEnabled(q.id, newVal);
    q.enabled = newVal ? 1 : 0;
}

async function toggleStar(q: SqlQuery) {
    const newVal = !q.is_starred;
    await api.setQueryStarred(q.id, newVal);
    q.is_starred = newVal ? 1 : 0;
}

function addQuery() {
    editingQuery.value = null;
    showEditor.value = true;
}

function editQuery(q: SqlQuery) {
    editingQuery.value = { ...q };
    showEditor.value = true;
}

async function deleteQuery(q: SqlQuery) {
    if (!confirm(`ต้องการลบ "${q.name}" หรือไม่?`)) return;
    await api.deleteQuery(q.id);
    await reload();
}

async function testQuery(q: SqlQuery) {
    editingQuery.value = { ...q };
    showEditor.value = true;
}

function manageDepartments() {
    showDeptModal.value = true;
}

function closeDeptModal() {
    showDeptModal.value = false;
}

async function onSaved() {
    showEditor.value = false;
    await reload();
}
</script>

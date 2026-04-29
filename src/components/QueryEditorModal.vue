<template>
    <div
        class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-6"
    >
        <div
            class="bg-[#2D2D2E] rounded-xl border border-[#3A3A3B] w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl"
        >
            <!-- Header -->
            <div
                class="flex items-center gap-3 px-5 py-4 border-b border-[#3A3A3B]"
            >
                <Code2 :size="18" class="text-[#FFB700]" />
                <h2 class="font-semibold text-sm flex-1">
                    {{ query ? "Edit Query" : "Add Query" }}
                </h2>
                <button class="btn-ghost p-1.5" @click="$emit('close')">
                    <X :size="18" />
                </button>
            </div>

            <!-- Form -->
            <div class="flex-1 overflow-y-auto p-5 space-y-4">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label
                            class="block text-xs text-[#aaa] mb-1.5 font-medium"
                            >Name *</label
                        >
                        <input
                            v-model="form.name"
                            class="input-field"
                            placeholder="ชื่อ Query"
                        />
                    </div>
                    <div>
                        <label
                            class="block text-xs text-[#aaa] mb-1.5 font-medium"
                            >Description</label
                        >
                        <input
                            v-model="form.description"
                            class="input-field"
                            placeholder="คำอธิบาย"
                        />
                    </div>
                </div>

                <!-- Department selector -->
                <div>
                    <label class="block text-xs text-[#aaa] mb-1.5 font-medium">
                        Department / แผนก
                    </label>
                    <select v-model="form.department_id" class="input-field">
                        <option :value="null">— ไม่ระบุแผนก —</option>
                        <option
                            v-for="d in departments"
                            :key="d.id"
                            :value="d.id"
                        >
                            {{ d.name }}
                        </option>
                    </select>
                </div>

                <!-- SQL editor -->
                <div>
                    <label class="block text-xs text-[#aaa] mb-1.5 font-medium"
                        >SQL Query *</label
                    >
                    <textarea
                        v-model="form.sql_text"
                        class="input-field font-mono text-xs h-64 resize-none"
                        placeholder="SELECT ..."
                        spellcheck="false"
                    ></textarea>
                </div>

                <!-- Date range for test -->
                <div class="flex items-center gap-3 text-sm">
                    <label
                        class="text-xs text-[#aaa] font-medium whitespace-nowrap"
                        >Test Date Range:</label
                    >
                    <input
                        type="date"
                        v-model="testDateFrom"
                        class="input-field w-36 text-xs"
                    />
                    <span class="text-[#555]">—</span>
                    <input
                        type="date"
                        v-model="testDateTo"
                        class="input-field w-36 text-xs"
                    />
                    <button
                        class="btn-secondary text-xs"
                        :disabled="testing"
                        @click="runTest"
                    >
                        <Play :size="13" />
                        {{ testing ? "Testing..." : "Test SQL" }}
                    </button>
                </div>

                <!-- Test result -->
                <div
                    v-if="testResult"
                    :class="[
                        'text-xs px-4 py-3 rounded-lg font-mono max-h-40 overflow-y-auto',
                        testOk
                            ? 'bg-[#70E000]/15 text-[#70E000]'
                            : 'bg-[#EF233C]/15 text-[#EF233C]',
                    ]"
                >
                    <div v-if="testOk">✓ {{ testResultMsg }}</div>
                    <div v-else class="whitespace-pre-wrap">
                        ✗ {{ testResultMsg }}
                    </div>
                    <!-- Preview first 5 rows -->
                    <div
                        v-if="testOk && testRows.length > 0"
                        class="mt-2 overflow-x-auto"
                    >
                        <table class="text-[10px] border-collapse">
                            <thead>
                                <tr>
                                    <th
                                        v-for="col in testCols"
                                        :key="col"
                                        class="px-2 py-0.5 border border-[#70E000]/30 text-[#70E000]/80"
                                    >
                                        {{ col }}
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr
                                    v-for="(row, i) in testRows.slice(0, 5)"
                                    :key="i"
                                >
                                    <td
                                        v-for="(cell, j) in row"
                                        :key="j"
                                        class="px-2 py-0.5 border border-[#70E000]/20 text-[#E9ECEF]/70"
                                    >
                                        {{ cell ?? "" }}
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                        <p
                            v-if="testRows.length > 5"
                            class="mt-1 text-[#70E000]/60"
                        >
                            ... and {{ testRows.length - 5 }} more rows
                        </p>
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div
                class="px-5 py-4 border-t border-[#3A3A3B] flex items-center gap-3"
            >
                <label class="flex items-center gap-2 text-sm cursor-pointer">
                    <input
                        type="checkbox"
                        v-model="form.enabled"
                        class="accent-[#FF4D00]"
                    />
                    <span class="text-[#aaa]">Enabled</span>
                </label>
                <label class="flex items-center gap-2 text-sm cursor-pointer">
                    <input
                        type="checkbox"
                        v-model="form.is_starred"
                        class="accent-[#FFB700]"
                    />
                    <Star :size="14" class="text-[#FFB700]" /> Starred
                </label>
                <div class="flex-1"></div>
                <button class="btn-secondary" @click="$emit('close')">
                    Cancel
                </button>
                <button
                    class="btn-primary"
                    :disabled="!form.name || !form.sql_text"
                    @click="save"
                >
                    <Save :size="15" /> Save
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { X, Code2, Play, Save, Star } from "lucide-vue-next";
import { api } from "../tauri-api";
import type { SqlQuery, Department, DbConfig } from "../types";

const props = defineProps<{
    query: SqlQuery | null;
    mode: "audit" | "report";
    departments: Department[];
    dbConfig: DbConfig | null;
    dbConnected: boolean;
}>();
const emit = defineEmits(["close", "saved"]);

const now = new Date();
const pad = (n: number) => String(n).padStart(2, "0");
const localISO = (d: Date) =>
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const testDateFrom = ref(
    localISO(new Date(now.getFullYear(), now.getMonth(), 1)),
);
const testDateTo = ref(
    localISO(new Date(now.getFullYear(), now.getMonth() + 1, 0)),
);

const form = reactive({
    name: props.query?.name ?? "",
    description: props.query?.description ?? "",
    sql_text: props.query?.sql_text ?? "",
    department_id: props.query?.department_id ?? null,
    enabled: props.query ? props.query.enabled === 1 : true,
    is_starred: props.query ? props.query.is_starred === 1 : false,
});

const testing = ref(false);
const testResult = ref(false);
const testOk = ref(false);
const testResultMsg = ref("");
const testCols = ref<string[]>([]);
const testRows = ref<(string | number | null)[][]>([]);

async function runTest() {
    if (!form.sql_text.trim()) return;
    testing.value = true;
    testResult.value = false;
    testCols.value = [];
    testRows.value = [];
    try {
        const result = await api.executeQuery(
            form.sql_text,
            testDateFrom.value,
            testDateTo.value,
        );
        testOk.value = true;
        testResultMsg.value = `${result.row_count.toLocaleString()} rows ใน ${result.elapsed_sec.toFixed(3)}s`;
        testCols.value = result.columns;
        testRows.value = result.rows;
    } catch (e: unknown) {
        testOk.value = false;
        testResultMsg.value = e instanceof Error ? e.message : String(e);
    } finally {
        testing.value = false;
        testResult.value = true;
    }
}

async function save() {
    if (!form.name.trim() || !form.sql_text.trim()) return;
    if (props.query) {
        await api.updateQuery(
            props.query.id,
            form.name,
            form.description,
            form.sql_text,
            form.enabled,
            form.department_id,
            form.is_starred,
        );
    } else {
        await api.insertQuery(
            props.mode,
            form.name,
            form.description,
            form.sql_text,
            form.department_id,
            form.is_starred,
        );
    }
    emit("saved");
}
</script>

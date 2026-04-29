<template>
    <div class="flex flex-col h-full">
        <!-- Header -->
        <div class="page-header">
            <ClockFading :size="22" class="page-header-icon" />
            <div>
                <h1 class="text-base font-bold text-[#E9ECEF]">
                    Execution History
                </h1>
                <p class="text-xs text-[#888]">ประวัติการรันคำสั่ง SQL</p>
            </div>
            <div class="ml-auto flex items-center gap-2">
                <!-- Filters -->
                <div class="select-wrap w-28">
                    <select
                        v-model="modeFilter"
                        class="text-xs"
                        @change="reload"
                    >
                        <option value="all">All Mode</option>
                        <option value="audit">Audit</option>
                        <option value="report">Report</option>
                    </select>
                </div>
                <div class="select-wrap w-32">
                    <select
                        v-model="statusFilter"
                        class="text-xs"
                        @change="reload"
                    >
                        <option value="all">All Status</option>
                        <option value="ok">✓ Pass</option>
                        <option value="notpass">✗ Not Pass</option>
                        <option value="error">✗ Error</option>
                    </select>
                </div>
                <button class="btn-danger text-xs" @click="clearHistory">
                    <Trash2 :size="14" /> Clear All
                </button>
            </div>
        </div>

        <!-- Table -->
        <div class="flex-1 overflow-auto p-4">
            <div
                v-if="history.length === 0"
                class="flex flex-col items-center justify-center h-64 text-[#555] gap-3"
            >
                <ClockFading :size="48" />
                <p>ยังไม่มีประวัติการรัน</p>
            </div>
            <div v-else class="table-container">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>#</th>
                            <th>Query Name</th>
                            <th>Mode</th>
                            <th>Date Range</th>
                            <th class="text-right">Rows</th>
                            <th class="text-right">Time (s)</th>
                            <th>Status</th>
                            <th>Error</th>
                            <th>Executed At</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="(h, i) in history" :key="h.id">
                            <td class="text-[#555] text-xs">{{ i + 1 }}</td>
                            <td class="font-medium max-w-[200px] truncate">
                                {{ h.query_name }}
                            </td>
                            <td>
                                <span
                                    :class="
                                        h.mode === 'audit'
                                            ? 'badge-warning'
                                            : 'badge-neutral'
                                    "
                                >
                                    {{ h.mode }}
                                </span>
                            </td>
                            <td class="text-xs text-[#aaa]">
                                {{ h.date_from }} → {{ h.date_to }}
                            </td>
                            <td class="text-right tabular-nums">
                                {{ h.row_count.toLocaleString() }}
                            </td>
                            <td class="text-right tabular-nums text-xs">
                                {{ h.elapsed_sec.toFixed(3) }}
                            </td>
                            <td>
                                <span
                                    :class="
                                        h.status === 'ok'
                                            ? 'badge-success'
                                            : h.status === 'notpass'
                                              ? 'badge-notpass'
                                              : 'badge-error'
                                    "
                                >
                                    {{
                                        h.status === "ok"
                                            ? "✓ Pass"
                                            : h.status === "notpass"
                                              ? "✗ Not Pass"
                                              : "✗ Error"
                                    }}
                                </span>
                            </td>
                            <td
                                class="text-xs text-[#FF4D00] max-w-[200px] truncate"
                            >
                                {{ h.error_msg || "—" }}
                            </td>
                            <td class="text-xs text-[#888]">
                                {{ h.executed_at }}
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Status bar -->
        <div
            class="px-6 py-2 border-t border-[#3A3A3B] flex justify-between items-center text-xs text-[#555]"
        >
            <span>พัฒนาโดย: กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์ ลพบุรี</span>
            <span>{{ history.length }} records</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Trash2, Clock as ClockFading } from "lucide-vue-next";
import { api } from "../tauri-api";
import type { ExecutionHistory } from "../types";

const history = ref<ExecutionHistory[]>([]);
const modeFilter = ref("all");
const statusFilter = ref("all");

onMounted(reload);

async function reload() {
    try {
        history.value = await api.getExecutionHistory(
            500,
            modeFilter.value,
            statusFilter.value,
        );
    } catch {
        /* ignore */
    }
}

async function clearHistory() {
    if (!confirm("ต้องการลบประวัติทั้งหมดหรือไม่?")) return;
    await api.clearExecutionHistory();
    history.value = [];
}
</script>

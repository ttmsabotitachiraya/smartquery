<template>
    <div
        class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-6"
        @mousedown.self="$emit('close')"
    >
        <div
            class="bg-[#2D2D2E] rounded-xl border border-[#3A3A3B] w-full max-w-5xl h-full max-h-[85vh] flex flex-col shadow-2xl"
        >
            <!-- Modal header -->
            <div
                class="flex items-center gap-3 px-5 py-4 border-b border-[#3A3A3B]"
            >
                <Table2 :size="18" class="text-[#FFB700]" />
                <div class="flex-1 min-w-0">
                    <h2 class="font-semibold text-sm text-[#E9ECEF] truncate">
                        {{ query.name }}
                    </h2>
                    <p class="text-xs text-[#888]">
                        {{ query.row_count?.toLocaleString() }} rows
                        <span v-if="query.elapsed_sec !== undefined">
                            · {{ query.elapsed_sec.toFixed(3) }}s
                        </span>
                    </p>
                </div>

                <!-- Filter input -->
                <div class="relative">
                    <Search
                        :size="13"
                        class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[#555]"
                    />
                    <input
                        v-model="filterText"
                        class="input-field w-48 text-xs pl-7"
                        placeholder="Filter results..."
                        autofocus
                    />
                </div>

                <!-- Close button -->
                <button class="btn-ghost p-1.5" @click="$emit('close')">
                    <X :size="18" />
                </button>
            </div>

            <!-- Table body -->
            <div class="flex-1 overflow-auto">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th
                                v-for="col in query.result_columns"
                                :key="col"
                                class="whitespace-nowrap"
                            >
                                {{ col }}
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="(row, i) in pagedRows" :key="i">
                            <td
                                v-for="(cell, j) in row"
                                :key="j"
                                class="max-w-[300px] truncate"
                                :title="formatCellTitle(cell)"
                            >
                                {{ formatCell(cell) }}
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div
                    v-if="filteredRows.length === 0"
                    class="flex flex-col items-center justify-center h-32 text-[#555] gap-2 text-sm"
                >
                    <Database :size="32" />
                    <span>No data</span>
                </div>
            </div>

            <!-- Footer -->
            <div
                class="px-5 py-3 border-t border-[#3A3A3B] flex items-center justify-between text-xs text-[#888]"
            >
                <!-- Row info + pagination -->
                <div class="flex items-center gap-3">
                    <span>
                        แสดง
                        {{ filteredRows.length > 0 ? pageStart + 1 : 0 }}–{{
                            pageEnd
                        }}
                        จาก {{ filteredRows.length.toLocaleString() }}
                        <template v-if="filterText">
                            (กรองจาก
                            {{
                                query.result_rows?.length?.toLocaleString() ?? 0
                            }}
                            rows)
                        </template>
                    </span>

                    <!-- Pagination controls -->
                    <div v-if="totalPages > 1" class="flex items-center gap-1">
                        <button
                            class="btn-ghost px-2 py-0.5 text-xs"
                            :disabled="currentPage === 1"
                            @click="currentPage--"
                        >
                            <ChevronLeft :size="14" />
                        </button>
                        <span class="tabular-nums text-[#aaa]"
                            >{{ currentPage }} / {{ totalPages }}</span
                        >
                        <button
                            class="btn-ghost px-2 py-0.5 text-xs"
                            :disabled="currentPage === totalPages"
                            @click="currentPage++"
                        >
                            <ChevronRight :size="14" />
                        </button>
                    </div>
                </div>

                <!-- Export CSV button -->
                <button class="btn-success text-xs" @click="exportCsv">
                    <FileDown :size="14" /> Export CSV
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import {
    X,
    Table2,
    FileDown,
    Search,
    Database,
    ChevronLeft,
    ChevronRight,
} from "lucide-vue-next";
import type { AuditQueryRow } from "../types";

const PAGE_SIZE = 200;

const props = defineProps<{ query: AuditQueryRow }>();
defineEmits(["close"]);

const filterText = ref("");
const currentPage = ref(1);

// Reset page when filter changes
watch(filterText, () => {
    currentPage.value = 1;
});

// ── Filtered rows ──────────────────────────────────────────────────────────
const filteredRows = computed(() => {
    if (!props.query.result_rows) return [];
    if (!filterText.value) return props.query.result_rows;
    const f = filterText.value.toLowerCase();
    return props.query.result_rows.filter((row) =>
        row.some((cell) =>
            String(cell ?? "")
                .toLowerCase()
                .includes(f),
        ),
    );
});

// ── Pagination ─────────────────────────────────────────────────────────────
const totalPages = computed(() =>
    Math.max(1, Math.ceil(filteredRows.value.length / PAGE_SIZE)),
);
const pageStart = computed(() => (currentPage.value - 1) * PAGE_SIZE);
const pageEnd = computed(() =>
    Math.min(currentPage.value * PAGE_SIZE, filteredRows.value.length),
);
const pagedRows = computed(() =>
    filteredRows.value.slice(pageStart.value, pageEnd.value),
);

// ── Export CSV (Blob download — works in Tauri webview) ────────────────────
function exportCsv() {
    if (!props.query.result_columns || !props.query.result_rows) return;

    const escape = (v: unknown) => `"${String(v ?? "").replace(/"/g, '""')}"`;

    const lines = [
        props.query.result_columns.join(","),
        ...props.query.result_rows.map((r) => r.map(escape).join(",")),
    ];
    const csv = lines.join("\n");
    const blob = new Blob(["\uFEFF" + csv], {
        type: "text/csv;charset=utf-8;",
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${props.query.name.replace(/\s+/g, "_")}.csv`;
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
}

/* Helpers for formatting displayed cells */
const formatCell = (v: unknown): string => {
    if (v === null || v === undefined) return "";
    // numbers
    if (typeof v === "number") {
        // integers with thousands separator, floats with up to 3 decimals (trim trailing zeros)
        if (Number.isInteger(v)) return v.toLocaleString();
        return Number(v)
            .toFixed(3)
            .replace(/\.?0+$/, "")
            .toString();
    }
    const s = String(v);

    // datetime: ISO-like or 'YYYY-MM-DD HH:MM:SS' -> normalize to space-separated
    const dtRegex = /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}/;
    const dateRegex = /^\d{4}-\d{2}-\d{2}$/;
    if (dtRegex.test(s)) {
        return s.replace("T", " ");
    }
    if (dateRegex.test(s)) {
        return s;
    }

    // fallback: return as-is
    return s;
};

const formatCellTitle = (v: unknown): string => {
    if (v === null || v === undefined) return "";
    // keep original string for tooltip (so users can see full precision/unformatted value)
    if (typeof v === "number") {
        return String(v);
    }
    return String(v);
};
</script>

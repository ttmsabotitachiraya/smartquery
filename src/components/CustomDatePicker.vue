<script setup lang="ts">
import {
    ref,
    computed,
    onMounted,
    onUnmounted,
    nextTick,
    type CSSProperties,
} from "vue";
import { Calendar, ChevronDown } from "lucide-vue-next";

// ── Props / Emits ─────────────────────────────────────────────────────────────
const props = withDefaults(
    defineProps<{
        modelValue: Date | null;
        placeholder?: string;
    }>(),
    { placeholder: "เลือกวันที่" },
);

const emit = defineEmits<{
    "update:modelValue": [value: Date | null];
}>();

// ── Thai locale data ──────────────────────────────────────────────────────────
const THAI_MONTHS = [
    "มกราคม",
    "กุมภาพันธ์",
    "มีนาคม",
    "เมษายน",
    "พฤษภาคม",
    "มิถุนายน",
    "กรกฎาคม",
    "สิงหาคม",
    "กันยายน",
    "ตุลาคม",
    "พฤศจิกายน",
    "ธันวาคม",
];

const THAI_MONTHS_SHORT = [
    "ม.ค.",
    "ก.พ.",
    "มี.ค.",
    "เม.ย.",
    "พ.ค.",
    "มิ.ย.",
    "ก.ค.",
    "ส.ค.",
    "ก.ย.",
    "ต.ค.",
    "พ.ย.",
    "ธ.ค.",
];

const THAI_DAYS = ["อา", "จ", "อ", "พ", "พฤ", "ศ", "ส"];

// ── Panel state ───────────────────────────────────────────────────────────────
const isOpen = ref(false);
const showMonthPicker = ref(false);

const triggerRef = ref<HTMLElement | null>(null);
const panelRef = ref<HTMLElement | null>(null);

const panelStyle = ref<CSSProperties>({
    position: "fixed",
    top: "0px",
    left: "0px",
    zIndex: 9999,
});

// Viewing month/year (not necessarily selected)
const viewYear = ref(new Date().getFullYear());
const viewMonth = ref(new Date().getMonth());

// ── Helpers ───────────────────────────────────────────────────────────────────
function fmtDisplay(d: Date): string {
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function sameDay(a: Date, b: Date): boolean {
    return (
        a.getFullYear() === b.getFullYear() &&
        a.getMonth() === b.getMonth() &&
        a.getDate() === b.getDate()
    );
}

// ── Calendar cells ────────────────────────────────────────────────────────────
interface CalCell {
    date: Date;
    day: number;
    isCurrentMonth: boolean;
    key: string;
}

const calendarCells = computed((): CalCell[] => {
    const cells: CalCell[] = [];
    const firstDay = new Date(viewYear.value, viewMonth.value, 1);
    const lastDay = new Date(viewYear.value, viewMonth.value + 1, 0);
    const startDow = firstDay.getDay(); // 0 = Sunday

    // Previous month fill
    for (let i = startDow - 1; i >= 0; i--) {
        const d = new Date(viewYear.value, viewMonth.value, -i);
        cells.push({
            date: d,
            day: d.getDate(),
            isCurrentMonth: false,
            key: `p${i}`,
        });
    }

    // Current month
    for (let d = 1; d <= lastDay.getDate(); d++) {
        cells.push({
            date: new Date(viewYear.value, viewMonth.value, d),
            day: d,
            isCurrentMonth: true,
            key: `c${d}`,
        });
    }

    // Next month fill → always 42 cells (6 rows × 7 cols)
    const remaining = 42 - cells.length;
    for (let d = 1; d <= remaining; d++) {
        cells.push({
            date: new Date(viewYear.value, viewMonth.value + 1, d),
            day: d,
            isCurrentMonth: false,
            key: `n${d}`,
        });
    }

    return cells;
});

function isSelected(d: Date): boolean {
    return !!props.modelValue && sameDay(d, props.modelValue);
}

function isToday(d: Date): boolean {
    return sameDay(d, new Date());
}

// ── Navigation ────────────────────────────────────────────────────────────────
function prevMonth() {
    if (viewMonth.value === 0) {
        viewMonth.value = 11;
        viewYear.value--;
    } else {
        viewMonth.value--;
    }
}

function nextMonth() {
    if (viewMonth.value === 11) {
        viewMonth.value = 0;
        viewYear.value++;
    } else {
        viewMonth.value++;
    }
}

function selectMonthFromPicker(idx: number) {
    viewMonth.value = idx;
    showMonthPicker.value = false;
}

// ── Actions ───────────────────────────────────────────────────────────────────
function pickDate(d: Date) {
    emit("update:modelValue", new Date(d));
    closePanel();
}

function pickToday() {
    const t = new Date();
    viewYear.value = t.getFullYear();
    viewMonth.value = t.getMonth();
    emit("update:modelValue", t);
    closePanel();
}

function clearDate() {
    emit("update:modelValue", null);
    closePanel();
}

// ── Panel open / close ────────────────────────────────────────────────────────
function openPanel() {
    const d = props.modelValue ?? new Date();
    viewYear.value = d.getFullYear();
    viewMonth.value = d.getMonth();
    showMonthPicker.value = false;
    isOpen.value = true;
    nextTick(updatePosition);
}

function closePanel() {
    isOpen.value = false;
    showMonthPicker.value = false;
}

function togglePanel() {
    isOpen.value ? closePanel() : openPanel();
}

function updatePosition() {
    if (!triggerRef.value) return;
    const rect = triggerRef.value.getBoundingClientRect();
    const PANEL_W = 284;
    let left = rect.left;
    if (left + PANEL_W > window.innerWidth - 8) {
        left = window.innerWidth - PANEL_W - 8;
    }
    panelStyle.value = {
        position: "fixed",
        top: `${rect.bottom + 6}px`,
        left: `${Math.max(8, left)}px`,
        zIndex: 9999,
    };
}

function onClickOutside(e: MouseEvent) {
    if (!isOpen.value) return;
    const t = e.target as Node;
    if (triggerRef.value?.contains(t) || panelRef.value?.contains(t)) return;
    closePanel();
}

function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpen.value) closePanel();
}

onMounted(() => {
    document.addEventListener("mousedown", onClickOutside);
    document.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
    document.removeEventListener("mousedown", onClickOutside);
    document.removeEventListener("keydown", onKeydown);
});
</script>

<template>
    <div ref="triggerRef" class="cdp">
        <!-- ── Trigger button ──────────────────────────────────────────────── -->
        <button class="cdp__trigger" type="button" @click="togglePanel">
            <span :class="{ cdp__placeholder: !modelValue }">
                {{ modelValue ? fmtDisplay(modelValue) : placeholder }}
            </span>
            <Calendar :size="14" class="cdp__icon cdp__icon--faded" />
        </button>

        <!-- ── Panel (teleported to body) ────────────────────────────────── -->
        <Teleport to="body">
            <Transition name="cdp-pop">
                <div
                    v-if="isOpen"
                    ref="panelRef"
                    class="cdp__panel"
                    :style="panelStyle"
                >
                    <!-- ── Month / Year picker overlay ─────────────────── -->
                    <div v-if="showMonthPicker">
                        <div class="cdp__mypicker">
                            <!-- Year navigator -->
                            <div class="cdp__year-nav">
                                <button
                                    class="cdp__ynav-btn"
                                    @click="viewYear--"
                                >
                                    ‹
                                </button>
                                <span class="cdp__year-label">{{
                                    viewYear
                                }}</span>
                                <button
                                    class="cdp__ynav-btn"
                                    @click="viewYear++"
                                >
                                    ›
                                </button>
                            </div>

                            <!-- Month grid (3 × 4) -->
                            <div class="cdp__month-grid">
                                <button
                                    v-for="(name, idx) in THAI_MONTHS_SHORT"
                                    :key="idx"
                                    class="cdp__month-cell"
                                    :class="{
                                        'cdp__month-cell--active':
                                            idx === viewMonth,
                                    }"
                                    @click="selectMonthFromPicker(idx)"
                                >
                                    {{ name }}
                                </button>
                            </div>
                        </div>

                        <div class="cdp__footer">
                            <button class="cdp__btn-clear" @click="clearDate">
                                ล้าง
                            </button>
                            <button class="cdp__btn-today" @click="pickToday">
                                วันนี้
                            </button>
                        </div>
                    </div>

                    <!-- ── Calendar view ───────────────────────────────── -->
                    <div v-else>
                        <!-- Header -->
                        <div class="cdp__header">
                            <button
                                class="cdp__month-label"
                                @click="showMonthPicker = true"
                            >
                                {{ THAI_MONTHS[viewMonth] }} {{ viewYear }}
                                <ChevronDown :size="13" class="cdp__chevron" />
                            </button>

                            <div class="cdp__nav">
                                <button
                                    class="cdp__nav-btn"
                                    title="เดือนก่อนหน้า"
                                    @click="prevMonth"
                                >
                                    ↑
                                </button>
                                <button
                                    class="cdp__nav-btn"
                                    title="เดือนถัดไป"
                                    @click="nextMonth"
                                >
                                    ↓
                                </button>
                            </div>
                        </div>

                        <!-- Weekday header row -->
                        <div class="cdp__weekdays">
                            <span v-for="d in THAI_DAYS" :key="d">{{ d }}</span>
                        </div>

                        <!-- Day grid -->
                        <div class="cdp__grid">
                            <button
                                v-for="cell in calendarCells"
                                :key="cell.key"
                                class="cdp__day"
                                :class="{
                                    'cdp__day--other': !cell.isCurrentMonth,
                                    'cdp__day--selected': isSelected(cell.date),
                                    'cdp__day--today':
                                        isToday(cell.date) &&
                                        !isSelected(cell.date),
                                }"
                                @click="pickDate(cell.date)"
                            >
                                {{ cell.day }}
                            </button>
                        </div>

                        <!-- Footer -->
                        <div class="cdp__footer">
                            <button class="cdp__btn-clear" @click="clearDate">
                                ล้าง
                            </button>
                            <button class="cdp__btn-today" @click="pickToday">
                                วันนี้
                            </button>
                        </div>
                    </div>
                </div>
            </Transition>
        </Teleport>
    </div>
</template>

<!-- Non-scoped so the teleported panel can be styled -->
<style>
/* ── Trigger ──────────────────────────────────────────────────────────────── */
.cdp {
    display: inline-block;
}

.cdp__trigger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: #2b2b2b;
    border: 1px solid #3d3d3d;
    color: #f8edeb;
    border-radius: 8px;
    height: 34px;
    padding: 0 10px;
    font-size: 12px;
    font-family: "Tahoma", "Sarabun", sans-serif;
    cursor: pointer;
    width: 132px;
    transition: border-color 0.15s;
    white-space: nowrap;
    overflow: hidden;
}

.cdp__trigger:hover {
    border-color: #e63946;
}

.cdp__icon {
    color: #888;
    flex-shrink: 0;
    margin-left: auto;
}
.cdp__icon--faded {
    color: #666;
    opacity: 0.65;
}

.cdp__placeholder {
    color: #666;
}

/* ── Panel ────────────────────────────────────────────────────────────────── */
.cdp__panel {
    width: 284px;
    background: #1c1c1e;
    border: 1px solid #3a3a3b;
    border-radius: 16px;
    box-shadow:
        0 20px 60px rgba(0, 0, 0, 0.75),
        0 4px 12px rgba(0, 0, 0, 0.4);
    overflow: hidden;
    font-family: "Tahoma", "Sarabun", sans-serif;
    font-size: 14px;
    color: #f8edeb;
}

/* ── Calendar header ──────────────────────────────────────────────────────── */
.cdp__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 14px 8px;
}

.cdp__month-label {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    color: #f8edeb;
    font-size: 15px;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 8px;
    transition: background 0.15s;
    letter-spacing: 0.01em;
}

.cdp__month-label:hover {
    background: rgba(255, 255, 255, 0.08);
}

.cdp__chevron {
    color: #999;
    flex-shrink: 0;
    margin-top: 1px;
}

.cdp__nav {
    display: flex;
    gap: 2px;
}

.cdp__nav-btn {
    background: none;
    border: none;
    color: #999;
    font-size: 17px;
    font-family: inherit;
    cursor: pointer;
    width: 34px;
    height: 34px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
        background 0.15s,
        color 0.15s;
}

.cdp__nav-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #f8edeb;
}

/* ── Weekday header row ───────────────────────────────────────────────────── */
.cdp__weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    padding: 2px 12px 0;
}

.cdp__weekdays span {
    text-align: center;
    font-size: 11px;
    font-weight: 700;
    color: #888;
    padding: 5px 0;
    letter-spacing: 0.03em;
}

/* ── Day grid ─────────────────────────────────────────────────────────────── */
.cdp__grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    padding: 2px 12px 4px;
    gap: 1px;
}

.cdp__day {
    background: none;
    border: none;
    color: #f0f0f0;
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
    height: 34px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.1s;
    position: relative;
}

.cdp__day:hover:not(.cdp__day--selected) {
    background: rgba(255, 255, 255, 0.08);
}

.cdp__day--other {
    color: #505050;
}

.cdp__day--selected {
    background: #e63946 !important;
    color: #fff !important;
    font-weight: 700;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(230, 57, 70, 0.45);
}

.cdp__day--today {
    color: #e63946;
    font-weight: 700;
}

.cdp__day--today::after {
    content: "";
    position: absolute;
    bottom: 4px;
    left: 50%;
    transform: translateX(-50%);
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: #e63946;
}

/* ── Footer ───────────────────────────────────────────────────────────────── */
.cdp__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 18px 14px;
    border-top: 1px solid #2a2a2c;
    margin-top: 6px;
}

.cdp__btn-clear,
.cdp__btn-today {
    background: none;
    border: none;
    color: #e63946;
    font-size: 14px;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
    transition: background 0.15s;
}

.cdp__btn-clear:hover,
.cdp__btn-today:hover {
    background: rgba(230, 57, 70, 0.12);
}

/* ── Month / Year picker overlay ─────────────────────────────────────────── */
.cdp__mypicker {
    padding: 18px 16px 10px;
}

.cdp__year-nav {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
}

.cdp__year-label {
    font-size: 18px;
    font-weight: 700;
    color: #f8edeb;
    min-width: 72px;
    text-align: center;
    letter-spacing: 0.01em;
}

.cdp__ynav-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 22px;
    font-family: inherit;
    cursor: pointer;
    width: 38px;
    height: 38px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
        background 0.15s,
        color 0.15s;
}

.cdp__ynav-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #f8edeb;
}

.cdp__month-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 5px;
}

.cdp__month-cell {
    background: none;
    border: none;
    color: #f0f0f0;
    font-size: 13px;
    font-family: inherit;
    padding: 11px 4px;
    border-radius: 8px;
    cursor: pointer;
    text-align: center;
    transition: background 0.12s;
}

.cdp__month-cell:hover:not(.cdp__month-cell--active) {
    background: rgba(255, 255, 255, 0.08);
}

.cdp__month-cell--active {
    background: #e63946;
    color: #fff;
    font-weight: 700;
    box-shadow: 0 2px 8px rgba(230, 57, 70, 0.4);
}

/* ── Open / close animation ───────────────────────────────────────────────── */
.cdp-pop-enter-active,
.cdp-pop-leave-active {
    transition:
        opacity 0.16s ease,
        transform 0.16s ease;
    transform-origin: top left;
}

.cdp-pop-enter-from,
.cdp-pop-leave-to {
    opacity: 0;
    transform: scaleY(0.93) translateY(-6px);
}
</style>

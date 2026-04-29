<template>
    <div
        class="flex h-screen w-screen overflow-hidden bg-[#1A1A1B] text-[#E9ECEF]"
    >
        <!-- Sidebar -->
        <aside
            class="flex flex-col w-52 min-w-[208px] bg-[#2D2D2E] border-r border-[#3A3A3B]"
        >
            <!-- Logo area -->
            <div
                class="flex items-center gap-3 px-4 py-4 border-b border-[#3A3A3B]"
            >
                <img
                    src="/logo.svg"
                    alt="SmartQuery Logo"
                    class="w-9 h-9 rounded-lg flex-shrink-0"
                />
                <div class="flex flex-col leading-tight">
                    <span class="text-[#E9ECEF] font-bold text-sm"
                        >SmartQuery</span
                    >
                    <span class="text-[#888] text-xs">for HOSxP</span>
                </div>
            </div>

            <!-- Navigation -->
            <nav class="flex-1 overflow-y-auto py-2">
                <div class="section-label">Workspace</div>
                <NavBtn :active="page === 'audit'" @click="page = 'audit'">
                    <Search :size="16" /> Audit
                </NavBtn>
                <NavBtn :active="page === 'report'" @click="page = 'report'">
                    <BarChart2 :size="16" /> Reports
                </NavBtn>
                <NavBtn :active="page === 'history'" @click="navigateHistory">
                    <History :size="16" /> History
                </NavBtn>

                <div class="my-2 mx-3 border-t border-[#3A3A3B]"></div>

                <div class="section-label">System</div>
                <NavBtn
                    :active="page === 'management'"
                    @click="page = 'management'"
                >
                    <Database :size="16" /> SQL Management
                </NavBtn>
                <NavBtn
                    :active="page === 'settings'"
                    @click="page = 'settings'"
                >
                    <Settings :size="16" /> Settings
                </NavBtn>
            </nav>

            <!-- Connection status -->
            <div class="px-3 py-3 border-t border-[#3A3A3B] bg-[#1A1A1B]/50">
                <div
                    class="text-[10px] font-bold uppercase tracking-widest text-[#888] mb-1"
                >
                    Connection
                </div>
                <div
                    v-if="dbConnected"
                    class="text-xs text-[#70E000] flex items-center gap-1.5"
                >
                    <CircleDot :size="10" class="text-[#70E000]" />
                    <span class="truncate"
                        >{{ dbConfig?.host }}/{{ dbConfig?.database }}</span
                    >
                </div>
                <div
                    v-else
                    class="text-xs text-[#888] flex items-center gap-1.5"
                >
                    <CircleDot :size="10" />
                    Not connected
                </div>
                <div class="text-right text-[10px] text-[#555] mt-1">
                    v3.0.0
                </div>
            </div>
        </aside>

        <!-- Main content -->
        <main class="flex-1 overflow-hidden">
            <AuditPage
                ref="auditPageRef"
                v-show="page === 'audit'"
                :db-config="dbConfig"
                :db-connected="dbConnected"
                @connect="handleConnect"
            />
            <ReportPage
                ref="reportPageRef"
                v-show="page === 'report'"
                :db-config="dbConfig"
                :db-connected="dbConnected"
                @connect="handleConnect"
            />
            <ManagementPage
                v-if="page === 'management'"
                :db-config="dbConfig"
                :db-connected="dbConnected"
                @close="onManagementClose"
            />
            <SettingsPage
                v-if="page === 'settings'"
                @saved="handleSettingsSaved"
            />
            <HistoryPage v-if="page === 'history'" />
        </main>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import {
    Search,
    BarChart2,
    History,
    Database,
    Settings,
    CircleDot,
} from "lucide-vue-next";
import NavBtn from "./components/NavBtn.vue";
import AuditPage from "./components/AuditPage.vue";
import ReportPage from "./components/ReportPage.vue";
import ManagementPage from "./components/ManagementPage.vue";
import SettingsPage from "./components/SettingsPage.vue";
import HistoryPage from "./components/HistoryPage.vue";
import { api } from "./tauri-api";
import type { DbConfig } from "./types";

const page = ref<"audit" | "report" | "management" | "settings" | "history">(
    "audit",
);
const dbConfig = ref<DbConfig | null>(null);
const dbConnected = ref(false);

const auditPageRef = ref<InstanceType<typeof AuditPage> | null>(null);
const reportPageRef = ref<InstanceType<typeof ReportPage> | null>(null);

// When leaving management page, refresh departments in audit and report tabs
watch(page, (_newPage, oldPage) => {
    if (oldPage === "management") {
        auditPageRef.value?.refreshDepartments();
        reportPageRef.value?.refreshDepartments();
    }
});

function onManagementClose() {
    page.value = "audit";
}

onMounted(async () => {
    try {
        const cfg = await api.loadDbConfig();
        dbConfig.value = cfg;
        if (cfg.host && cfg.user) {
            const result = await api.connectDb(cfg);
            if (result) dbConnected.value = true;
        }
    } catch {
        // ignore
    }
});

async function handleSettingsSaved(cfg: DbConfig) {
    dbConfig.value = cfg;
    try {
        await api.connectDb(cfg);
        dbConnected.value = true;
    } catch {
        dbConnected.value = false;
    }
}

async function handleConnect(cfg: DbConfig) {
    dbConfig.value = cfg;
    dbConnected.value = true;
}

function navigateHistory() {
    page.value = "history";
}
</script>

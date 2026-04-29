<template>
    <div
        class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-6"
    >
        <div
            class="bg-[#2D2D2E] rounded-xl border border-[#3A3A3B] w-full max-w-md shadow-2xl"
        >
            <!-- Header -->
            <div
                class="flex items-center gap-3 px-5 py-4 border-b border-[#3A3A3B]"
            >
                <Building2 :size="18" class="text-[#FFB700]" />
                <h2 class="font-semibold text-sm flex-1">Manage Departments</h2>
                <button class="btn-ghost p-1.5" @click="$emit('close')">
                    <X :size="18" />
                </button>
            </div>

            <!-- List -->
            <div class="p-4 space-y-2 max-h-72 overflow-y-auto">
                <div
                    v-for="d in departments"
                    :key="d.id"
                    class="flex items-center gap-2 px-3 py-2 bg-[#1A1A1B]/50 rounded-lg"
                >
                    <!-- Normal view -->
                    <template v-if="editingId !== d.id">
                        <span class="flex-1 text-sm text-[#E9ECEF]">{{
                            d.name
                        }}</span>
                        <button
                            class="btn-ghost text-xs px-2 py-1"
                            @click="startEdit(d)"
                            title="เปลี่ยนชื่อ"
                        >
                            <Pencil :size="13" />
                        </button>
                        <button
                            class="btn-danger text-xs px-2 py-1"
                            @click="del(d.id)"
                            title="ลบแผนก"
                        >
                            <Trash2 :size="13" />
                        </button>
                    </template>

                    <!-- Edit view -->
                    <template v-else>
                        <input
                            ref="editInput"
                            v-model="editName"
                            class="input-field flex-1 text-sm py-1"
                            @keydown.enter="saveEdit(d.id)"
                            @keydown.escape="cancelEdit"
                        />
                        <button
                            class="btn-success text-xs px-2 py-1"
                            :disabled="!editName.trim()"
                            @click="saveEdit(d.id)"
                        >
                            <Check :size="13" />
                        </button>
                        <button
                            class="btn-ghost text-xs px-2 py-1"
                            @click="cancelEdit"
                        >
                            <X :size="13" />
                        </button>
                    </template>
                </div>
                <div
                    v-if="departments.length === 0"
                    class="text-center py-6 text-[#555] text-sm"
                >
                    ยังไม่มีแผนก
                </div>
            </div>

            <!-- Add new -->
            <div class="px-4 py-4 border-t border-[#3A3A3B] flex gap-2">
                <input
                    v-model="newName"
                    class="input-field flex-1 text-sm"
                    placeholder="ชื่อแผนกใหม่ เช่น OPD, IPD, ER..."
                    @keydown.enter="add"
                />
                <button
                    class="btn-primary text-sm"
                    :disabled="!newName.trim()"
                    @click="add"
                >
                    <Plus :size="15" /> Add
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import { Building2, X, Trash2, Plus, Pencil, Check } from "lucide-vue-next";
import { api } from "../tauri-api";
import type { Department } from "../types";

defineProps<{ departments: Department[] }>();
const emit = defineEmits(["close", "changed"]);

const newName = ref("");
const editingId = ref<number | null>(null);
const editName = ref("");
const editInput = ref<HTMLInputElement | null>(null);

async function add() {
    if (!newName.value.trim()) return;
    await api.insertDepartment(newName.value.trim());
    newName.value = "";
    emit("changed");
}

function startEdit(d: Department) {
    editingId.value = d.id;
    editName.value = d.name;
    nextTick(() => {
        editInput.value?.focus();
    });
}

function cancelEdit() {
    editingId.value = null;
    editName.value = "";
}

async function saveEdit(id: number) {
    if (!editName.value.trim()) return;
    await api.updateDepartment(id, editName.value.trim());
    editingId.value = null;
    editName.value = "";
    emit("changed");
}

async function del(id: number) {
    if (
        !confirm(
            "ต้องการลบแผนกนี้หรือไม่?\nQuery ที่สังกัดแผนกนี้จะถูกเปลี่ยนเป็นไม่มีแผนก",
        )
    )
        return;
    await api.deleteDepartment(id);
    emit("changed");
}
</script>

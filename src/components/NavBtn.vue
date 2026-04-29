<template>
    <button
        :class="[
            'w-full flex items-center gap-2.5 px-4 py-2.5 text-sm font-medium text-left',
            active
                ? 'bg-[#FF4D00]/20 text-[#FF4D00] border-r-2 border-[#FF4D00]'
                : 'text-[#aaa] hover:bg-white/5 hover:text-[#E9ECEF]',
        ]"
        @click="$emit('click')"
    >
        <slot />
    </button>
</template>

<script setup lang="ts">
defineProps<{ active: boolean }>();
defineEmits(["click"]);
</script>

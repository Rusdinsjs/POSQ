<script lang="ts">
    import { onMount } from 'svelte';
    import BackButton from '$lib/components/BackButton.svelte';
    import { capabilityStore } from '$lib/capabilities/capabilityStore.svelte';

    let selectedPreset = $state('general_flexible');
    let isProcessing = $state(false);
    let successMessage = $state('');
    let errorMessage = $state('');

    onMount(async () => {
        await capabilityStore.fetchAvailablePresets();
        await capabilityStore.loadCapabilities('default_outlet');
        selectedPreset = capabilityStore.activePreset;
    });

    async function handleSelectPreset(code: string) {
        isProcessing = true;
        successMessage = '';
        errorMessage = '';
        try {
            await capabilityStore.changePreset('default_outlet', code);
            selectedPreset = capabilityStore.activePreset;
            successMessage = `Mode bisnis berhasil diubah ke preset "${code}". Navigasi aplikasi telah disesuaikan!`;
        } catch (e: any) {
            errorMessage = String(e);
        } finally {
            isProcessing = false;
        }
    }

    function getPresetIcon(code: string) {
        switch (code) {
            case 'general_flexible': return '⚡';
            case 'retail_standard': return '🛍️';
            case 'retail_serialized': return '📱';
            case 'fnb_quick_service': return '☕';
            case 'fnb_table_service': return '🍔';
            case 'nonprofit_donation': return '💚';
            case 'cooperative_member_store': return '🏢';
            case 'public_service_fee': return '🏛️';
            case 'internal_issue': return '📦';
            case 'school_campus': return '🏫';
            case 'parking': return '🚗';
            case 'fuel_energy': return '⛽';
            default: return '🏬';
        }
    }
</script>

<div class="p-8 max-w-6xl mx-auto space-y-6">
    <BackButton />

    <div class="flex items-center justify-between bg-slate-800 p-6 rounded-2xl border border-slate-700 text-white shadow-xl">
        <div>
            <h1 class="text-3xl font-black tracking-tight flex items-center gap-3">
                <span>🏬</span> Pengaturan DNA Mode Bisnis
            </h1>
            <p class="text-sm text-slate-400 mt-1">
                Pilih preset DNA bisnis outlet Anda. Sistem akan menyesuaikan kapabilitas backend, alur kasir, dan bilah navigasi secara otomatis.
            </p>
        </div>
        {#if capabilityStore.isLoading}
            <span class="px-4 py-2 bg-indigo-500/20 text-indigo-400 font-bold rounded-xl text-xs flex items-center gap-2 border border-indigo-500/30">
                <span>⏳</span> Memuat Kapabilitas...
            </span>
        {/if}
    </div>

    {#if successMessage}
        <div class="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-xl text-emerald-400 text-sm font-semibold flex items-center justify-between">
            <span>✅ {successMessage}</span>
            <button class="text-xs underline hover:text-white" onclick={() => successMessage = ''}>Tutup</button>
        </div>
    {/if}

    {#if errorMessage}
        <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-sm font-semibold flex items-center justify-between">
            <span>⚠️ {errorMessage}</span>
            <button class="text-xs underline hover:text-white" onclick={() => errorMessage = ''}>Tutup</button>
        </div>
    {/if}

    <!-- Grid Preset DNA Bisnis -->
    <div class="space-y-4">
        <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
            <span>🧬</span> Daftar Preset DNA Bisnis Terdaftar
        </h2>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {#each capabilityStore.availablePresets as preset}
                {@const isActive = capabilityStore.activePreset === preset.code}
                <div 
                    class="bg-white rounded-2xl p-6 border-2 shadow-sm hover:shadow-md cursor-pointer transition-all flex flex-col justify-between space-y-4 relative overflow-hidden {isActive ? 'border-indigo-600 ring-2 ring-indigo-500/20 bg-indigo-50/20' : 'border-slate-200 hover:border-indigo-300'}"
                    onclick={() => handleSelectPreset(preset.code)}
                >
                    {#if isActive}
                        <div class="absolute top-0 right-0 bg-indigo-600 text-white text-[10px] font-black uppercase tracking-widest px-3 py-1 rounded-bl-xl shadow">
                            Preset Aktif
                        </div>
                    {/if}

                    <div class="space-y-3">
                        <div class="flex items-center gap-3">
                            <span class="text-3xl p-2 bg-slate-100 rounded-xl">{getPresetIcon(preset.code)}</span>
                            <div>
                                <h3 class="font-bold text-lg text-slate-900 leading-tight">{preset.name}</h3>
                                <span class="text-[11px] font-mono text-slate-400 block mt-0.5">{preset.code}</span>
                            </div>
                        </div>

                        <p class="text-xs text-slate-600 leading-relaxed">
                            {preset.description}
                        </p>
                    </div>

                    <!-- Kapabilitas Bawaan -->
                    <div class="border-t border-slate-100 pt-3 space-y-2">
                        <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">Kapabilitas Bawaan:</span>
                        <div class="flex flex-wrap gap-1">
                            {#each preset.default_capabilities as cap}
                                <span class="px-2 py-0.5 bg-slate-100 text-slate-600 rounded text-[10px] font-mono border border-slate-200">
                                    {cap}
                                </span>
                            {/each}
                        </div>
                    </div>

                    <button 
                        class="w-full py-2.5 rounded-xl font-bold text-xs transition-all flex items-center justify-center gap-2 {isActive ? 'bg-indigo-600 text-white shadow-md' : 'bg-slate-100 hover:bg-indigo-50 text-slate-700 hover:text-indigo-600'}"
                        disabled={isProcessing}
                    >
                        {#if isActive}
                            <span>✓</span> Digunakan Saat Ini
                        {:else}
                            <span>👉</span> Aktifkan Preset Ini
                        {/if}
                    </button>
                </div>
            {/each}
        </div>
    </div>
</div>

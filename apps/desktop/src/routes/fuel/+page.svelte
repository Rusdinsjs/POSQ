<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let pumpId = $state('PUMP-01');
  let fuelType = $state('pertalite');
  let initialMeter = $state<number>(0);
  let finalMeter = $state<number>(0);
  let shiftId = $state('SHIFT-PAGI');
  let loading = $state(false);
  let readingResult = $state<any>(null);
  let errorMessage = $state('');

  let calculatedVolume = $derived(
    finalMeter >= initialMeter ? finalMeter - initialMeter : 0
  );

  async function handleRecordMeter() {
    if (finalMeter < initialMeter) {
      errorMessage = 'Angka meter akhir tidak boleh lebih kecil dari meter awal!';
      return;
    }
    loading = true;
    errorMessage = '';
    try {
      const res = await invoke('record_fuel_pump_reading_cmd', {
        pumpId,
        fuelType,
        initialMeter: Number(initialMeter),
        finalMeter: Number(finalMeter),
        shiftId: shiftId ? shiftId : null
      });
      readingResult = res;
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="p-6 max-w-5xl mx-auto space-y-6">
  <!-- Header -->
  <div class="bg-slate-800 p-5 rounded-2xl border border-slate-700 text-white shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black tracking-tight flex items-center gap-3">
        <span class="p-2 bg-amber-500 rounded-xl text-slate-900">⛽</span> SPBU & EV Charging Dispenser Metering
      </h1>
      <p class="text-xs text-slate-400 mt-1">Pencatatan angka meter awal/akhir dispenser dan rekonsiliasi stok BBM per shift</p>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-sm font-semibold flex items-center justify-between">
      <span>⚠️ {errorMessage}</span>
      <button class="text-xs underline hover:text-white" onclick={() => errorMessage = ''}>Tutup</button>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- Form Input Meter -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-5 text-white shadow-lg">
      <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
        <span>📋</span> Form Catat Meter Dispenser
      </h2>

      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-1.5">
          <label for="pump-id-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">ID Pompa / Dispenser</label>
          <input 
            id="pump-id-input"
            type="text" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-amber-500"
            bind:value={pumpId}
          />
        </div>

        <div class="space-y-1.5">
          <label for="shift-id-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Shift</label>
          <input 
            id="shift-id-input"
            type="text" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-amber-500"
            bind:value={shiftId}
          />
        </div>
      </div>

      <div class="space-y-1.5">
        <label for="fuel-type-select" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Jenis Bahan Bakar / Listrik</label>
        <select 
          id="fuel-type-select"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-white font-bold focus:outline-none focus:border-amber-500"
          bind:value={fuelType}
        >
          <option value="pertalite">Pertalite (RON 90)</option>
          <option value="pertamax">Pertamax (RON 92)</option>
          <option value="solar">Biosolar / Dexlite</option>
          <option value="ev_kwh">EV Charging (kWh)</option>
        </select>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-1.5">
          <label for="initial-meter-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Meter Awal Shift (Liter)</label>
          <input 
            id="initial-meter-input"
            type="number" 
            step="0.1"
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-white font-mono font-bold text-lg focus:outline-none focus:border-amber-500"
            bind:value={initialMeter}
          />
        </div>

        <div class="space-y-1.5">
          <label for="final-meter-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Meter Akhir Shift (Liter)</label>
          <input 
            id="final-meter-input"
            type="number" 
            step="0.1"
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-emerald-400 font-mono font-bold text-lg focus:outline-none focus:border-amber-500"
            bind:value={finalMeter}
          />
        </div>
      </div>

      <!-- Preview Volume -->
      <div class="bg-amber-500/10 border border-amber-500/30 p-4 rounded-xl flex justify-between items-center">
        <span class="text-xs font-bold text-amber-400 uppercase tracking-wider">Volume Terdispensasi:</span>
        <span class="text-2xl font-mono font-black text-amber-400">{calculatedVolume.toFixed(2)} Liter</span>
      </div>

      <button 
        class="w-full py-4 bg-amber-500 hover:bg-amber-400 text-slate-950 font-black rounded-xl shadow-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 text-base"
        disabled={loading}
        onclick={handleRecordMeter}
      >
        {#if loading}
          <span>⏳</span> Menyimpan Meter...
        {:else}
          <span>💾</span> Simpan & Rekonsiliasi Meter Shift
        {/if}
      </button>
    </div>

    <!-- Ringkasan Hasil Meter -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl flex flex-col justify-between text-white shadow-lg">
      {#if readingResult}
        <div class="space-y-6">
          <div class="border-b border-slate-800 pb-3">
            <span class="text-xs font-bold text-amber-400 uppercase tracking-widest block">Hasil Rekonsiliasi Meter Berhasil</span>
            <h3 class="text-xl font-mono font-bold mt-1">ID: {readingResult.id.substring(0, 8)}</h3>
          </div>

          <div class="space-y-3">
            <div class="flex justify-between bg-slate-800/50 p-3 rounded-xl">
              <span class="text-slate-400 text-sm">Pompa:</span>
              <span class="font-bold">{readingResult.pump_id}</span>
            </div>
            <div class="flex justify-between bg-slate-800/50 p-3 rounded-xl">
              <span class="text-slate-400 text-sm">Jenis BBM:</span>
              <span class="font-bold uppercase text-amber-400">{readingResult.fuel_type}</span>
            </div>
            <div class="flex justify-between bg-slate-800/50 p-3 rounded-xl">
              <span class="text-slate-400 text-sm">Meter Awal:</span>
              <span class="font-mono font-bold">{readingResult.initial_meter}</span>
            </div>
            <div class="flex justify-between bg-slate-800/50 p-3 rounded-xl">
              <span class="text-slate-400 text-sm">Meter Akhir:</span>
              <span class="font-mono font-bold text-emerald-400">{readingResult.final_meter}</span>
            </div>
          </div>

          <div class="bg-emerald-500/10 border border-emerald-500/30 p-5 rounded-2xl text-center space-y-1">
            <span class="text-xs text-emerald-400 uppercase font-bold tracking-wider block">Total BBM Terpenuhi</span>
            <span class="text-3xl font-black text-emerald-400">{readingResult.volume_liters.toFixed(2)} Liter</span>
          </div>
        </div>

        <button 
          class="w-full py-3.5 bg-slate-800 hover:bg-slate-700 text-white font-bold rounded-xl transition-all border border-slate-700 mt-6"
          onclick={() => readingResult = null}
        >
          Selesai & Catat Pompa Lain
        </button>
      {:else}
        <div class="h-full flex flex-col items-center justify-center text-slate-500 space-y-2 py-12">
          <span class="text-5xl block">📊</span>
          <p class="text-sm font-semibold">Hasil rekonsiliasi meter dispenser akan tampil di sini</p>
        </div>
      {/if}
    </div>
  </div>
</div>

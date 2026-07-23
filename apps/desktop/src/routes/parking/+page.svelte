<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let activeTab: 'entrance' | 'exit' = $state('entrance');
  
  // Gate Entrance State
  let vehicleType = $state('car');
  let licensePlate = $state('');
  let entranceTicket = $state<any>(null);
  let entranceLoading = $state(false);

  // Gate Exit State
  let ticketCodeScan = $state('');
  let isLostTicket = $state(false);
  let exitResult = $state<any>(null);
  let exitLoading = $state(false);
  let errorMessage = $state('');

  async function handleIssueTicket() {
    entranceLoading = true;
    errorMessage = '';
    try {
      const res = await invoke('issue_parking_ticket_cmd', {
        vehicleType,
        licensePlate: licensePlate ? licensePlate : null
      });
      entranceTicket = res;
      licensePlate = '';
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      entranceLoading = false;
    }
  }

  async function handleProcessExit() {
    if (!ticketCodeScan) return;
    exitLoading = true;
    errorMessage = '';
    try {
      const res = await invoke('process_parking_exit_cmd', {
        ticketCode: ticketCodeScan,
        isLostTicket
      });
      exitResult = res;
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      exitLoading = false;
    }
  }

  function formatRp(val: number) {
    return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0 }).format(val);
  }
</script>

<div class="p-6 max-w-6xl mx-auto space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between bg-slate-800 p-5 rounded-2xl border border-slate-700 text-white shadow-xl">
    <div>
      <h1 class="text-2xl font-black tracking-tight flex items-center gap-3">
        <span class="p-2 bg-indigo-600 rounded-xl">🚗</span> Operational Parkir & Sistem Gate
      </h1>
      <p class="text-xs text-slate-400 mt-1">Manajemen palang masuk/keluar, cetak tiket barcode, dan kalkulasi tarif progresif</p>
    </div>
    <div class="flex gap-2 bg-slate-900/80 p-1.5 rounded-xl border border-slate-700">
      <button 
        class="px-5 py-2.5 rounded-lg font-bold text-sm transition-all flex items-center gap-2 {activeTab === 'entrance' ? 'bg-indigo-600 text-white shadow-lg' : 'text-slate-400 hover:text-white'}"
        onclick={() => { activeTab = 'entrance'; errorMessage = ''; }}
      >
        <span>🟢</span> Gate Masuk (Entry)
      </button>
      <button 
        class="px-5 py-2.5 rounded-lg font-bold text-sm transition-all flex items-center gap-2 {activeTab === 'exit' ? 'bg-emerald-600 text-white shadow-lg' : 'text-slate-400 hover:text-white'}"
        onclick={() => { activeTab = 'exit'; errorMessage = ''; }}
      >
        <span>🔴</span> Gate Keluar (Exit & Pay)
      </button>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-sm font-semibold flex items-center justify-between">
      <span>⚠️ {errorMessage}</span>
      <button class="text-xs underline hover:text-white" onclick={() => errorMessage = ''}>Tutup</button>
    </div>
  {/if}

  {#if activeTab === 'entrance'}
    <!-- GATE MASUK -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-5 text-white shadow-lg">
        <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
          <span>📝</span> Cetak Tiket Gate Masuk
        </h2>

        <div class="space-y-2">
          <label for="vehicle-type-select" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Pilih Jenis Kendaraan</label>
          <div class="grid grid-cols-3 gap-3">
            <button 
              class="p-4 rounded-xl border-2 flex flex-col items-center gap-2 font-bold text-sm transition-all {vehicleType === 'motorcycle' ? 'border-indigo-500 bg-indigo-500/10 text-indigo-400' : 'border-slate-800 bg-slate-800/50 text-slate-400 hover:border-slate-700'}"
              onclick={() => vehicleType = 'motorcycle'}
            >
              <span class="text-2xl">🛵</span> Motor
            </button>
            <button 
              class="p-4 rounded-xl border-2 flex flex-col items-center gap-2 font-bold text-sm transition-all {vehicleType === 'car' ? 'border-indigo-500 bg-indigo-500/10 text-indigo-400' : 'border-slate-800 bg-slate-800/50 text-slate-400 hover:border-slate-700'}"
              onclick={() => vehicleType = 'car'}
            >
              <span class="text-2xl">🚗</span> Mobil
            </button>
            <button 
              class="p-4 rounded-xl border-2 flex flex-col items-center gap-2 font-bold text-sm transition-all {vehicleType === 'bus_truck' ? 'border-indigo-500 bg-indigo-500/10 text-indigo-400' : 'border-slate-800 bg-slate-800/50 text-slate-400 hover:border-slate-700'}"
              onclick={() => vehicleType = 'bus_truck'}
            >
              <span class="text-2xl">🚚</span> Bus / Truk
            </button>
          </div>
        </div>

        <div class="space-y-2">
          <label for="license-plate-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nomor Polisi (Opsional)</label>
          <input 
            id="license-plate-input"
            type="text" 
            placeholder="Contoh: B 1234 ABC" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-white font-mono uppercase tracking-widest text-lg focus:outline-none focus:border-indigo-500"
            bind:value={licensePlate}
          />
        </div>

        <button 
          class="w-full py-4 bg-indigo-600 hover:bg-indigo-500 text-white font-black rounded-xl shadow-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 text-base"
          disabled={entranceLoading}
          onclick={handleIssueTicket}
        >
          {#if entranceLoading}
            <span>⏳</span> Memproses Gate...
          {:else}
            <span>🖨️</span> Cetak Tiket & Buka Palang
          {/if}
        </button>
      </div>

      <!-- TIKET HASIL CETAK -->
      <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl flex flex-col items-center justify-center text-white shadow-lg min-h-[300px]">
        {#if entranceTicket}
          <div class="w-full max-w-sm bg-white text-slate-900 p-6 rounded-xl shadow-2xl space-y-4 font-mono text-center border-2 border-dashed border-slate-300">
            <div class="border-b border-slate-300 pb-2">
              <h3 class="text-xl font-black tracking-widest">POSQ PARKING</h3>
              <p class="text-xs text-slate-500">Struk Masuk Kendaraan</p>
            </div>

            <div class="py-2">
              <span class="text-3xl font-black tracking-widest text-indigo-600 block">{entranceTicket.ticket_code}</span>
              <p class="text-xs text-slate-500 mt-1">Scan Barcode Ini Saat Keluar</p>
            </div>

            <div class="text-xs text-left space-y-1 bg-slate-100 p-3 rounded-lg">
              <div class="flex justify-between"><span>Jenis:</span> <span class="font-bold uppercase">{entranceTicket.vehicle_type}</span></div>
              <div class="flex justify-between"><span>NoPol:</span> <span class="font-bold">{entranceTicket.license_plate || '-'}</span></div>
              <div class="flex justify-between"><span>Masuk:</span> <span class="font-bold">{new Date(entranceTicket.entry_time).toLocaleTimeString('id-ID')}</span></div>
            </div>

            <div class="text-[10px] text-slate-400 italic pt-2">
              Simpan tiket ini dengan baik. Hilang tiket dikenakan denda resmi.
            </div>
          </div>
        {:else}
          <div class="text-center text-slate-500 space-y-2">
            <span class="text-5xl block">🎟️</span>
            <p class="text-sm font-semibold">Struk tiket masuk akan muncul di sini setelah dicetak</p>
          </div>
        {/if}
      </div>
    </div>

  {:else}
    <!-- GATE KELUAR -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-5 text-white shadow-lg">
        <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
          <span>🔍</span> Scan Tiket & Pembayaran Gate Keluar
        </h2>

        <div class="space-y-2">
          <label for="ticket-code-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Scan Barcode / Ketik Kode Tiket</label>
          <input 
            id="ticket-code-input"
            type="text" 
            placeholder="Contoh: PRK-A1B2C3D4" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-emerald-400 font-mono uppercase tracking-widest text-xl font-bold focus:outline-none focus:border-emerald-500"
            bind:value={ticketCodeScan}
          />
        </div>

        <div class="flex items-center gap-3 bg-rose-500/10 border border-rose-500/30 p-3 rounded-xl">
          <input 
            type="checkbox" 
            id="lostTicket" 
            class="w-5 h-5 accent-rose-500 rounded cursor-pointer"
            bind:checked={isLostTicket}
          />
          <label for="lostTicket" class="text-sm font-semibold text-rose-300 cursor-pointer">
            Tiket Hilang (Dikenakan Denda Resmi)
          </label>
        </div>

        <button 
          class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 text-white font-black rounded-xl shadow-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 text-base"
          disabled={exitLoading || !ticketCodeScan}
          onclick={handleProcessExit}
        >
          {#if exitLoading}
            <span>⏳</span> Memproses Tarif...
          {:else}
            <span>💳</span> Kalkulasi Tarif & Layani Pembayaran
          {/if}
        </button>
      </div>

      <!-- HASIL CALCULASI TARIF -->
      <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl flex flex-col justify-between text-white shadow-lg">
        {#if exitResult}
          <div class="space-y-6">
            <div class="flex justify-between items-start border-b border-slate-800 pb-4">
              <div>
                <span class="text-xs font-bold text-emerald-400 uppercase tracking-widest block">Ringkasan Biaya Parkir</span>
                <h3 class="text-2xl font-mono font-black">{exitResult.ticket_code}</h3>
              </div>
              <span class="px-3 py-1 bg-emerald-500/20 text-emerald-400 rounded-full text-xs font-bold uppercase">{exitResult.status}</span>
            </div>

            <div class="grid grid-cols-2 gap-4 text-sm">
              <div class="bg-slate-800/50 p-3 rounded-xl">
                <span class="text-slate-400 text-xs block">Jenis Kendaraan</span>
                <span class="font-bold text-base uppercase">{exitResult.vehicle_type}</span>
              </div>
              <div class="bg-slate-800/50 p-3 rounded-xl">
                <span class="text-slate-400 text-xs block">Total Durasi</span>
                <span class="font-bold text-base">{exitResult.duration_minutes} Menit</span>
              </div>
            </div>

            <div class="bg-emerald-500/10 border border-emerald-500/30 p-6 rounded-2xl text-center space-y-1">
              <span class="text-xs text-emerald-400 uppercase font-bold tracking-wider block">Total Tarif Yang Harus Dibayar</span>
              <span class="text-4xl font-black text-emerald-400">{formatRp(exitResult.total_fee)}</span>
            </div>
          </div>

          <button 
            class="w-full py-3.5 bg-slate-800 hover:bg-slate-700 text-white font-bold rounded-xl transition-all border border-slate-700 mt-6"
            onclick={() => { exitResult = null; ticketCodeScan = ''; isLostTicket = false; }}
          >
            Selesai & Buka Palang Keluar
          </button>
        {:else}
          <div class="h-full flex flex-col items-center justify-center text-slate-500 space-y-2 py-12">
            <span class="text-5xl block">🚗💨</span>
            <p class="text-sm font-semibold">Hasil perhitungan tarif progresif akan tampil di sini</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

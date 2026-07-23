<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let customerName = $state('');
  let customerPhone = $state('');
  let assetName = $state('iPhone 13 Pro');
  let serialImei = $state('358921098471203');
  let problemDescription = $state('Layar pecah & baterai bocor');

  let ticketList = $state<any[]>([]);
  let loading = $state(false);
  let errorMessage = $state('');

  async function handleCreateTicket() {
    if (!customerName || !assetName) return;
    loading = true;
    errorMessage = '';
    try {
      const res = await invoke('create_repair_ticket_cmd', {
        customerName,
        customerPhone: customerPhone ? customerPhone : null,
        assetName,
        serialImei: serialImei ? serialImei : null,
        problemDescription
      });
      ticketList = [res, ...ticketList];
      customerName = '';
      customerPhone = '';
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      loading = false;
    }
  }

  function getStatusBadge(status: string) {
    switch (status) {
      case 'Intake': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
      case 'Diagnosing': return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
      case 'InRepair': return 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30';
      case 'ReadyForPickup': return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
      default: return 'bg-slate-700 text-slate-300 border-slate-600';
    }
  }
</script>

<div class="p-6 max-w-6xl mx-auto space-y-6">
  <!-- Header -->
  <div class="bg-slate-800 p-5 rounded-2xl border border-slate-700 text-white shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black tracking-tight flex items-center gap-3">
        <span class="p-2 bg-indigo-600 rounded-xl">🔧</span> Servis & Bengkel Maintenance Ticket
      </h1>
      <p class="text-xs text-slate-400 mt-1">Intake aset pelanggan, pelacakan IMEI/Serial, dan alur status pengerjaan servis</p>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-sm font-semibold flex items-center justify-between">
      <span>⚠️ {errorMessage}</span>
      <button class="text-xs underline hover:text-white" onclick={() => errorMessage = ''}>Tutup</button>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Form Intake Servis -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-4 text-white shadow-lg md:col-span-1">
      <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
        <span>📥</span> Penerimaan Aset Servis Baru
      </h2>

      <div class="space-y-1">
        <label for="customer-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nama Pelanggan</label>
        <input 
          id="customer-name-input"
          type="text" 
          placeholder="Nama Pemilik"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-indigo-500"
          bind:value={customerName}
        />
      </div>

      <div class="space-y-1">
        <label for="customer-phone-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">No. HP / WA</label>
        <input 
          id="customer-phone-input"
          type="text" 
          placeholder="08123456789"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-mono focus:outline-none focus:border-indigo-500"
          bind:value={customerPhone}
        />
      </div>

      <div class="space-y-1">
        <label for="asset-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nama Barang / Perangkat</label>
        <input 
          id="asset-name-input"
          type="text" 
          placeholder="Contoh: MacBook Pro 2021 / Honda Vario"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-indigo-500"
          bind:value={assetName}
        />
      </div>

      <div class="space-y-1">
        <label for="serial-imei-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">No. Seri / IMEI / Plat Nomor</label>
        <input 
          id="serial-imei-input"
          type="text" 
          placeholder="SN / IMEI / Plat"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-indigo-400 font-mono font-bold focus:outline-none focus:border-indigo-500"
          bind:value={serialImei}
        />
      </div>

      <div class="space-y-1">
        <label for="problem-desc-textarea" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Keluhan / Kerusakan</label>
        <textarea 
          id="problem-desc-textarea"
          rows="3"
          placeholder="Jelaskan kendala..."
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white text-sm focus:outline-none focus:border-indigo-500"
          bind:value={problemDescription}
        ></textarea>
      </div>

      <button 
        class="w-full py-3.5 bg-indigo-600 hover:bg-indigo-500 text-white font-black rounded-xl shadow-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 text-sm"
        disabled={loading || !customerName || !assetName}
        onclick={handleCreateTicket}
      >
        {#if loading}
          <span>⏳</span> Membuat Tiket...
        {:else}
          <span>🎟️</span> Buat Tiket Servis & Cetak Tanda Terima
        {/if}
      </button>
    </div>

    <!-- Daftar Tiket Servis Aktif -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-4 text-white shadow-lg md:col-span-2">
      <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
        <span>📋</span> Tiket Servis Aktif
      </h2>

      {#if ticketList.length === 0}
        <div class="text-center text-slate-500 py-16 space-y-2">
          <span class="text-5xl block">🛠️</span>
          <p class="text-sm font-semibold">Belum ada tiket servis yang dibuat hari ini</p>
        </div>
      {:else}
        <div class="space-y-3">
          {#each ticketList as ticket}
            <div class="bg-slate-800/60 border border-slate-700/60 p-4 rounded-xl space-y-3">
              <div class="flex justify-between items-start">
                <div>
                  <span class="text-xs font-mono font-bold text-indigo-400 block">{ticket.ticket_number}</span>
                  <h3 class="text-base font-bold">{ticket.asset_name}</h3>
                  <p class="text-xs text-slate-400">Pemilik: <span class="text-white font-semibold">{ticket.customer_name}</span> ({ticket.customer_phone || '-'})</p>
                </div>
                <span class="px-3 py-1 rounded-full text-xs font-bold border {getStatusBadge(ticket.status)}">
                  {ticket.status}
                </span>
              </div>

              <p class="text-xs bg-slate-900/60 p-2.5 rounded-lg text-slate-300 border border-slate-800">
                💬 <span class="italic">{ticket.problem_description}</span>
              </p>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

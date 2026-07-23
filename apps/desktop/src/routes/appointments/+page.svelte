<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let customerName = $state('');
  let customerPhone = $state('');
  let staffName = $state('Stylist Rina');
  let serviceName = $state('Hair Styling & Treatment');
  let appointmentDate = $state(new Date().toISOString().substring(0, 10));
  let appointmentTime = $state('14:00');

  let appointments = $state<any[]>([]);
  let loading = $state(false);
  let errorMessage = $state('');

  async function handleCreateAppointment() {
    if (!customerName) return;
    loading = true;
    errorMessage = '';
    try {
      const res = await invoke('create_appointment_cmd', {
        customerName,
        customerPhone: customerPhone ? customerPhone : null,
        staffName,
        serviceName,
        appointmentTime: `${appointmentDate}T${appointmentTime}:00Z`
      });
      appointments = [res, ...appointments];
      customerName = '';
      customerPhone = '';
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="p-6 max-w-6xl mx-auto space-y-6">
  <!-- Header -->
  <div class="bg-slate-800 p-5 rounded-2xl border border-slate-700 text-white shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black tracking-tight flex items-center gap-3">
        <span class="p-2 bg-purple-600 rounded-xl">📅</span> Janji Temu Salon, Spa & Klinik
      </h1>
      <p class="text-xs text-slate-400 mt-1">Penjadwalan reservasi pelanggan, alokasi staf/kapster, dan manajemen slot waktu</p>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-sm font-semibold flex items-center justify-between">
      <span>⚠️ {errorMessage}</span>
      <button class="text-xs underline hover:text-white" onclick={() => errorMessage = ''}>Tutup</button>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Form Reservasi -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-4 text-white shadow-lg md:col-span-1">
      <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
        <span>✍️</span> Buat Reservasi Baru
      </h2>

      <div class="space-y-1">
        <label for="customer-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nama Pelanggan</label>
        <input 
          id="customer-name-input"
          type="text" 
          placeholder="Nama Pelanggan"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-purple-500"
          bind:value={customerName}
        />
      </div>

      <div class="space-y-1">
        <label for="customer-phone-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">No. HP / WA</label>
        <input 
          id="customer-phone-input"
          type="text" 
          placeholder="08123456789"
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-mono focus:outline-none focus:border-purple-500"
          bind:value={customerPhone}
        />
      </div>

      <div class="space-y-1">
        <label for="service-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Layanan / Treatment</label>
        <input 
          id="service-name-input"
          type="text" 
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-purple-500"
          bind:value={serviceName}
        />
      </div>

      <div class="space-y-1">
        <label for="staff-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Staf / Stylist / Terapis</label>
        <input 
          id="staff-name-input"
          type="text" 
          class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-purple-400 font-bold focus:outline-none focus:border-purple-500"
          bind:value={staffName}
        />
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label for="appointment-date-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Tanggal</label>
          <input 
            id="appointment-date-input"
            type="date" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-3 py-2 text-white font-bold text-xs focus:outline-none focus:border-purple-500"
            bind:value={appointmentDate}
          />
        </div>
        <div class="space-y-1">
          <label for="appointment-time-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Jam</label>
          <input 
            id="appointment-time-input"
            type="time" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-3 py-2 text-white font-bold text-xs focus:outline-none focus:border-purple-500"
            bind:value={appointmentTime}
          />
        </div>
      </div>

      <button 
        class="w-full py-3.5 bg-purple-600 hover:bg-purple-500 text-white font-black rounded-xl shadow-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 text-sm"
        disabled={loading || !customerName}
        onclick={handleCreateAppointment}
      >
        {#if loading}
          <span>⏳</span> Menyimpan...
        {:else}
          <span>📆</span> Simpan Reservasi Janji Temu
        {/if}
      </button>
    </div>

    <!-- Daftar Reservasi Hari Ini -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-4 text-white shadow-lg md:col-span-2">
      <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
        <span>📋</span> Jadwal Janji Temu Aktif
      </h2>

      {#if appointments.length === 0}
        <div class="text-center text-slate-500 py-16 space-y-2">
          <span class="text-5xl block">💇‍♀️</span>
          <p class="text-sm font-semibold">Belum ada reservasi janji temu yang didaftarkan</p>
        </div>
      {:else}
        <div class="space-y-3">
          {#each appointments as appt}
            <div class="bg-slate-800/60 border border-slate-700/60 p-4 rounded-xl flex justify-between items-center">
              <div>
                <span class="text-xs font-bold text-purple-400 block">{appt.service_name}</span>
                <h3 class="text-base font-bold">{appt.customer_name} ({appt.customer_phone || '-'})</h3>
                <p class="text-xs text-slate-400 mt-0.5">Staf: <span class="text-white font-semibold">{appt.staff_name}</span></p>
              </div>
              <div class="text-right">
                <span class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-full text-xs font-bold border border-purple-500/30 block">
                  {new Date(appt.appointment_time).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' })}
                </span>
                <span class="text-[10px] text-slate-400 uppercase mt-1 block">{appt.status}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

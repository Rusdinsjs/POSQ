<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let activeTab: 'donation' | 'retribusi' = $state('donation');

  // Donation State
  let donorName = $state('');
  let donorPhone = $state('');
  let campaignName = $state('Pembangunan Masjid');
  let fundType = $state('zakat');
  let amount = $state<number>(100000);
  let donationResult = $state<any>(null);
  let donationLoading = $state(false);

  // Retribusi State
  let serviceName = $state('Retribusi Pasar / Parkir Umum');
  let retribusiAmount = $state<number>(50000);
  let discountAttempt = $state<number>(0);
  let retribusiError = $state('');

  let errorMessage = $state('');

  async function handleRecordDonation() {
    if (!donorName || amount <= 0) return;
    donationLoading = true;
    errorMessage = '';
    try {
      const res = await invoke('record_donation_cmd', {
        orderId: 'DON-' + Date.now(),
        donorName,
        donorPhone: donorPhone ? donorPhone : null,
        campaignName,
        fundType,
        amount: Number(amount)
      });
      donationResult = res;
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      donationLoading = false;
    }
  }

  function handleRetribusiSubmit() {
    if (discountAttempt > 0) {
      retribusiError = 'NO_DISCOUNT_ALLOWED: Mode Layanan Publik / Retribusi dilarang memberikan diskon!';
      return;
    }
    retribusiError = '';
    alert('Penerimaan Retribusi Resmi Berhasil Diproses Tanpa Diskon!');
  }

  function formatRp(val: number) {
    return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0 }).format(val);
  }
</script>

<div class="p-6 max-w-5xl mx-auto space-y-6">
  <!-- Header -->
  <div class="bg-slate-800 p-5 rounded-2xl border border-slate-700 text-white shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black tracking-tight flex items-center gap-3">
        <span class="p-2 bg-rose-500 rounded-xl">💚</span> Donasi, Yayasan & Retribusi Publik
      </h1>
      <p class="text-xs text-slate-400 mt-1">Penerimaan dana donasi (tanpa HPP) & penerimaan retribusi dengan kebijakan larangan diskon</p>
    </div>
    <div class="flex gap-2 bg-slate-900/80 p-1.5 rounded-xl border border-slate-700">
      <button 
        class="px-4 py-2 rounded-lg font-bold text-xs transition-all {activeTab === 'donation' ? 'bg-rose-600 text-white shadow-lg' : 'text-slate-400 hover:text-white'}"
        onclick={() => activeTab = 'donation'}
      >
        ❤️ Donasi & Zakat
      </button>
      <button 
        class="px-4 py-2 rounded-lg font-bold text-xs transition-all {activeTab === 'retribusi' ? 'bg-blue-600 text-white shadow-lg' : 'text-slate-400 hover:text-white'}"
        onclick={() => activeTab = 'retribusi'}
      >
        🏛️ Retribusi Publik
      </button>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-sm font-semibold flex items-center justify-between">
      <span>⚠️ {errorMessage}</span>
      <button class="text-xs underline hover:text-white" onclick={() => errorMessage = ''}>Tutup</button>
    </div>
  {/if}

  {#if activeTab === 'donation'}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-5 text-white shadow-lg">
        <h2 class="text-lg font-bold border-b border-slate-800 pb-3 flex items-center gap-2">
          <span>✍️</span> Form Kuitansi Donatur
        </h2>

        <div class="space-y-1.5">
          <label for="donor-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nama Donatur / Hamba Allah</label>
          <input 
            id="donor-name-input"
            type="text" 
            placeholder="Contoh: H. Ahmad"
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-white font-bold focus:outline-none focus:border-rose-500"
            bind:value={donorName}
          />
        </div>

        <div class="space-y-1.5">
          <label for="donor-phone-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nomor Telepon (Opsional)</label>
          <input 
            id="donor-phone-input"
            type="text" 
            placeholder="08123456789"
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-mono focus:outline-none focus:border-rose-500"
            bind:value={donorPhone}
          />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label for="campaign-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Kampanye / Program</label>
            <input 
              id="campaign-name-input"
              type="text" 
              class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-rose-500"
              bind:value={campaignName}
            />
          </div>

          <div class="space-y-1.5">
            <label for="fund-type-select" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Jenis Dana</label>
            <select 
              id="fund-type-select"
              class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-white font-bold focus:outline-none focus:border-rose-500"
              bind:value={fundType}
            >
              <option value="zakat">Zakat</option>
              <option value="infaq">Infaq / Sedekah</option>
              <option value="unrestricted">Donasi Umum</option>
              <option value="wakaf">Wakaf</option>
            </select>
          </div>
        </div>

        <div class="space-y-1.5">
          <label for="donation-amount-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nominal Donasi (Rp)</label>
          <input 
            id="donation-amount-input"
            type="number" 
            step="10000"
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-rose-400 font-mono font-black text-2xl focus:outline-none focus:border-rose-500"
            bind:value={amount}
          />
        </div>

        <button 
          class="w-full py-4 bg-rose-600 hover:bg-rose-500 text-white font-black rounded-xl shadow-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 text-base"
          disabled={donationLoading || !donorName}
          onclick={handleRecordDonation}
        >
          {#if donationLoading}
            <span>⏳</span> Memproses...
          {:else}
            <span>🤝</span> Terima Donasi & Cetak Kuitansi
          {/if}
        </button>
      </div>

      <!-- Kuitansi Donasi -->
      <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl flex flex-col items-center justify-center text-white shadow-lg">
        {#if donationResult}
          <div class="w-full max-w-sm bg-white text-slate-900 p-6 rounded-xl shadow-2xl space-y-4 font-mono text-center border-2 border-dashed border-rose-300">
            <div class="border-b border-slate-300 pb-2">
              <h3 class="text-xl font-black text-rose-600">YAYASAN / POSQ CARE</h3>
              <p class="text-xs text-slate-500">Kuitansi Penerimaan Donasi</p>
            </div>

            <div class="text-xs text-left space-y-2 bg-rose-50 p-3 rounded-lg border border-rose-100">
              <div class="flex justify-between"><span>Donatur:</span> <span class="font-bold">{donationResult.donor_name}</span></div>
              <div class="flex justify-between"><span>Program:</span> <span class="font-bold">{donationResult.campaign_name}</span></div>
              <div class="flex justify-between"><span>Jenis:</span> <span class="font-bold uppercase text-rose-700">{donationResult.fund_type}</span></div>
              <div class="flex justify-between border-t border-rose-200 pt-1 text-sm"><span class="font-bold">Jumlah:</span> <span class="font-black text-rose-600">{formatRp(donationResult.amount)}</span></div>
            </div>

            <p class="text-[10px] text-slate-400 italic">Jazakallah Khairan atas bantuan yang diberikan.</p>
          </div>
        {:else}
          <div class="text-center text-slate-500 space-y-2">
            <span class="text-5xl block">📜</span>
            <p class="text-sm font-semibold">Kuitansi Donatur resmi akan muncul di sini</p>
          </div>
        {/if}
      </div>
    </div>

  {:else}
    <!-- RETRIBUSI PUBLIK -->
    <div class="bg-slate-900 border border-slate-800 p-6 rounded-2xl space-y-6 text-white shadow-lg max-w-2xl mx-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h2 class="text-lg font-bold flex items-center gap-2">
          <span>🏛️</span> Layanan Retribusi Resmi Pemerintah
        </h2>
        <span class="px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full text-xs font-bold border border-blue-500/30">
          🔒 NO_DISCOUNT_ALLOWED
        </span>
      </div>

      {#if retribusiError}
        <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-xl text-rose-400 text-xs font-bold">
          ⚠️ {retribusiError}
        </div>
      {/if}

      <div class="space-y-4">
        <div class="space-y-1.5">
          <label for="service-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Jenis Layanan / Permohonan</label>
          <input 
            id="service-name-input"
            type="text" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-white font-bold focus:outline-none"
            bind:value={serviceName}
          />
        </div>

        <div class="space-y-1.5">
          <label for="retribusi-amount-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Tarif Resmi Perda (Rp)</label>
          <input 
            id="retribusi-amount-input"
            type="number" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-3 text-blue-400 font-mono font-black text-2xl focus:outline-none"
            bind:value={retribusiAmount}
          />
        </div>

        <div class="space-y-1.5">
          <label for="discount-attempt-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Uji Input Diskon (Dilarang dalam Retribusi)</label>
          <input 
            id="discount-attempt-input"
            type="number" 
            class="w-full bg-slate-800 border border-slate-700 rounded-xl px-4 py-2.5 text-rose-400 font-mono focus:outline-none"
            bind:value={discountAttempt}
          />
        </div>
      </div>

      <button 
        class="w-full py-4 bg-blue-600 hover:bg-blue-500 text-white font-black rounded-xl shadow-lg transition-all"
        onclick={handleRetribusiSubmit}
      >
        🏛️ Diproses & Cetak Kuitansi Retribusi Resmi
      </button>
    </div>
  {/if}
</div>

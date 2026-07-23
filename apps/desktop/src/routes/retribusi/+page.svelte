<script lang="ts">
  let searchQuery = $state('');
  let activeTab: 'loket' | 'services' | 'taxpayers' | 'applications' = $state('loket');

  let services = $state([
    { code: 'RET-01', name: 'Retribusi Kebersihan & Pasar', rate: 25000, category: 'Pasar', discountAllowed: false },
    { code: 'RET-02', name: 'Retribusi Parkir Umum Tepi Jalan', rate: 50000, category: 'Perhubungan', discountAllowed: false },
    { code: 'RET-03', name: 'Retribusi Izin Mendirikan Bangunan (PBG)', rate: 250000, category: 'Tata Ruang', discountAllowed: false },
    { code: 'RET-04', name: 'Retribusi Pemakaian Layanan Gedung Serbaguna', rate: 500000, category: 'Aset Daerah', discountAllowed: false },
  ]);

  let queues = $state([
    { queueNo: 'A-012', applicant: 'Budi Santoso', service: 'Retribusi Kebersihan & Pasar', amount: 25000, status: 'Menunggu Pembayaran' },
    { queueNo: 'A-013', applicant: 'PT Mulia Sejahtera', service: 'Retribusi Pemakaian Layanan Gedung', amount: 500000, status: 'Lunas' },
  ]);

  function formatRp(val: number) {
    return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0 }).format(val);
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- Header Banner -->
  <div class="bg-slate-900 text-white p-6 rounded-2xl border border-slate-800 shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black flex items-center gap-3">
        <span class="p-2 bg-blue-600 rounded-xl">🏛️</span> Loket Retribusi & Layanan Publik
      </h1>
      <p class="text-xs text-slate-400 mt-1">H. Penerimaan Retribusi Resmi Pemda/Instansi, Wajib Bayar, Cetak Karcis & Kuitansi Berurutan</p>
    </div>
    <span class="px-3.5 py-1.5 bg-blue-500/20 text-blue-400 border border-blue-500/30 rounded-xl text-xs font-bold uppercase tracking-wider">
      🔒 NO_DISCOUNT_ALLOWED
    </span>
  </div>

  <!-- Metric Cards -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Loket Aktif</span>
      <span class="text-2xl font-black text-slate-900">Loket 01 (Utama)</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Antrean Diproses Hari Ini</span>
      <span class="text-2xl font-black text-blue-600">{queues.length} Permohonan</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Penerimaan Retribusi Hari Ini</span>
      <span class="text-2xl font-black text-emerald-600">{formatRp(525000)}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Status Setoran Petugas</span>
      <span class="text-2xl font-black text-amber-600">Terbuka / Shift 1</span>
    </div>
  </div>

  <!-- Main Content Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Form Penerimaan Retribusi -->
    <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-4 md:col-span-1">
      <h2 class="text-lg font-bold border-b border-slate-100 pb-3 flex items-center gap-2 text-slate-900">
        <span>✍️</span> Loket Pembayaran Karcis
      </h2>

      <div class="space-y-1.5">
        <label for="applicant-name-input" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Nama Wajib Bayar / Pemohon</label>
        <input id="applicant-name-input" type="text" placeholder="Nama Wajib Bayar" class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm font-bold text-slate-900 focus:outline-none focus:border-blue-500" />
      </div>

      <div class="space-y-1.5">
        <label for="retribusi-service-select" class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Pilih Jenis Layanan Retribusi</label>
        <select id="retribusi-service-select" class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm font-bold text-slate-900 focus:outline-none focus:border-blue-500">
          {#each services as item}
            <option value={item.code}>{item.name} - {formatRp(item.rate)}</option>
          {/each}
        </select>
      </div>

      <div class="p-3 bg-rose-50 border border-rose-200 rounded-xl text-xs font-bold text-rose-700">
        ⚠️ Kebijakan Hukum Perda: Diskon/Potongan Harga dilarang keras untuk retribusi publik.
      </div>

      <button class="w-full py-3.5 bg-blue-600 hover:bg-blue-500 text-white font-black rounded-xl shadow transition text-sm flex items-center justify-center gap-2">
        <span>🖨️</span> Cetak Karcis Retribusi Resmi
      </button>
    </div>

    <!-- Tabel Jenis Layanan & Antrean -->
    <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-4 md:col-span-2">
      <h2 class="text-lg font-bold border-b border-slate-100 pb-3 flex items-center justify-between text-slate-900">
        <span class="flex items-center gap-2"><span>📋</span> Daftar Tarip & Layanan Retribusi Resmi</span>
      </h2>

      <div class="overflow-hidden rounded-xl border border-slate-200">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr class="bg-slate-50 border-b border-slate-200 text-xs font-bold text-slate-500 uppercase tracking-wider">
              <th class="p-3">Kode</th>
              <th class="p-3">Nama Layanan</th>
              <th class="p-3">Kategori</th>
              <th class="p-3 text-right">Tarif Resmi</th>
              <th class="p-3 text-center">Diskon</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100 text-sm font-medium text-slate-700">
            {#each services as service}
              <tr class="hover:bg-slate-50/80 transition">
                <td class="p-3 font-mono font-bold text-blue-600">{service.code}</td>
                <td class="p-3 font-bold text-slate-900">{service.name}</td>
                <td class="p-3 text-slate-500">{service.category}</td>
                <td class="p-3 text-right font-mono font-bold text-slate-900">{formatRp(service.rate)}</td>
                <td class="p-3 text-center">
                  <span class="px-2 py-0.5 bg-rose-100 text-rose-700 rounded text-[10px] font-black uppercase">Dilarang</span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>
</div>

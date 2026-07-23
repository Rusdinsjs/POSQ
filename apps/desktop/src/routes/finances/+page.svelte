<script lang="ts">
  let searchQuery = $state('');
  let activeTab: 'expenses' | 'purchasing' | 'disbursement' | 'reconciliation' = $state('expenses');

  let transactions = $state([
    { id: 'FIN-1001', type: 'Pengeluaran Operasional', category: 'Listrik & Air', amount: 1250000, recipient: 'PLN Persero', date: '2026-07-22', status: 'Lunas' },
    { id: 'FIN-1002', type: 'Pembelian Stok Supplier', category: 'Restock Pangan', amount: 8500000, recipient: 'PT Sembako Nusantara', date: '2026-07-21', status: 'Lunas' },
    { id: 'FIN-1003', type: 'Penyaluran Donasi', category: 'Program Pendidikan', amount: 5000000, recipient: 'Mustahik Beasiswa', date: '2026-07-20', status: 'Disalurkan' },
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
        <span class="p-2 bg-emerald-600 rounded-xl">💰</span> Pengeluaran, Pembelian & Keuangan
      </h1>
      <p class="text-xs text-slate-400 mt-1">Manajemen Pengeluaran Operasional, Pembelian Stok, Hutang/Piutang & Rekonsiliasi Bank</p>
    </div>
    <div class="flex gap-2 bg-slate-800 p-1.5 rounded-xl border border-slate-700">
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'expenses' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'expenses'}>💸 Pengeluaran</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'purchasing' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'purchasing'}>🛒 Pembelian Stok</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'disbursement' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'disbursement'}>🤝 Penyaluran Dana</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'reconciliation' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'reconciliation'}>🏦 Rekonsiliasi Bank</button>
    </div>
  </div>

  <!-- Metric Cards -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Pengeluaran Bulan Ini</span>
      <span class="text-2xl font-black text-rose-600">{formatRp(14750000)}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Pembelian Stok</span>
      <span class="text-2xl font-black text-slate-900">{formatRp(8500000)}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Hutang Supplier</span>
      <span class="text-2xl font-black text-amber-600">{formatRp(0)}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Saldo Rekening Bank</span>
      <span class="text-2xl font-black text-emerald-600">{formatRp(45200000)}</span>
    </div>
  </div>

  <!-- Actions & Table -->
  <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-4">
    <div class="flex items-center justify-between gap-4">
      <div class="relative flex-grow max-w-md">
        <input 
          type="text" 
          placeholder="Cari No. Transaksi, Kategori, atau Penerima..." 
          class="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm font-semibold focus:outline-none focus:border-emerald-500"
          bind:value={searchQuery}
        />
        <span class="absolute left-3.5 top-3 text-slate-400">🔍</span>
      </div>
      <button class="px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-black rounded-xl shadow transition text-sm flex items-center gap-2">
        <span>➕</span> Catat Pengeluaran Baru
      </button>
    </div>

    <div class="overflow-hidden rounded-xl border border-slate-200">
      <table class="w-full text-left border-collapse">
        <thead>
          <tr class="bg-slate-50 border-b border-slate-200 text-xs font-bold text-slate-500 uppercase tracking-wider">
            <th class="p-4">No. Transaksi</th>
            <th class="p-4">Jenis & Kategori</th>
            <th class="p-4">Penerima / Keterangan</th>
            <th class="p-4">Tanggal</th>
            <th class="p-4 text-right">Jumlah</th>
            <th class="p-4 text-center">Status</th>
            <th class="p-4 text-center">Aksi</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100 text-sm font-medium text-slate-700">
          {#each transactions as item}
            <tr class="hover:bg-slate-50/80 transition">
              <td class="p-4 font-mono font-bold text-emerald-600">{item.id}</td>
              <td class="p-4 font-bold text-slate-900">{item.type} <span class="text-xs text-slate-400 block font-normal">{item.category}</span></td>
              <td class="p-4 text-slate-600">{item.recipient}</td>
              <td class="p-4 font-mono text-xs text-slate-500">{item.date}</td>
              <td class="p-4 text-right font-mono font-black text-rose-600">{formatRp(item.amount)}</td>
              <td class="p-4 text-center">
                <span class="px-3 py-1 bg-emerald-100 text-emerald-800 rounded-full text-xs font-bold">{item.status}</span>
              </td>
              <td class="p-4 text-center">
                <button class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-bold transition">
                  Kuitansi
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

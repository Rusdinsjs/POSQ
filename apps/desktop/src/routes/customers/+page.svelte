<script lang="ts">
  let searchQuery = $state('');
  let activeCategory: 'all' | 'member' | 'donor' | 'taxpayer' | 'supplier' = $state('all');

  let customers = $state([
    { id: 'CUST-001', name: 'Bambang Sudirman', phone: '08123456789', category: 'member', typeLabel: 'Member Loyalty', points: 450, debt: 150000, status: 'Aktif' },
    { id: 'DON-102', name: 'H. Ahmad Syarif', phone: '08198765432', category: 'donor', typeLabel: 'Donatur Yayasan', points: 0, debt: 0, status: 'Aktif' },
    { id: 'TAX-501', name: 'CV Rekarsa Jaya', phone: '08215554433', category: 'taxpayer', typeLabel: 'Wajib Bayar Retribusi', points: 0, debt: 500000, status: 'Aktif' },
    { id: 'SUP-901', name: 'PT Sembako Nusantara', phone: '02177788899', category: 'supplier', typeLabel: 'Supplier Pangan', points: 0, debt: 0, status: 'Aktif' },
  ]);

  let filteredCustomers = $derived(
    customers.filter(c => {
      const matchQuery = c.name.toLowerCase().includes(searchQuery.toLowerCase()) || c.id.toLowerCase().includes(searchQuery.toLowerCase()) || c.phone.includes(searchQuery);
      if (activeCategory === 'all') return matchQuery;
      return matchQuery && c.category === activeCategory;
    })
  );

  function formatRp(val: number) {
    return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0 }).format(val);
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- Header Banner -->
  <div class="bg-slate-900 text-white p-6 rounded-2xl border border-slate-800 shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black flex items-center gap-3">
        <span class="p-2 bg-indigo-600 rounded-xl">👥</span> Pelanggan, Donatur, Member & Supplier
      </h1>
      <p class="text-xs text-slate-400 mt-1">Manajemen Entitas Pelanggan, Poin Loyalty, Donatur Yayasan, Wajib Bayar & Supplier</p>
    </div>
    <div class="flex gap-2 bg-slate-800 p-1.5 rounded-xl border border-slate-700">
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeCategory === 'all' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeCategory = 'all'}>Semua</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeCategory === 'member' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeCategory = 'member'}>Member</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeCategory === 'donor' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeCategory = 'donor'}>Donatur</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeCategory === 'taxpayer' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeCategory = 'taxpayer'}>Wajib Bayar</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeCategory === 'supplier' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeCategory = 'supplier'}>Supplier</button>
    </div>
  </div>

  <!-- Metric Cards -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Entitas Terdaftar</span>
      <span class="text-2xl font-black text-slate-900">{customers.length} Entitas</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Poin Member</span>
      <span class="text-2xl font-black text-amber-600">450 Poin</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Piutang Berjalan</span>
      <span class="text-2xl font-black text-rose-600">{formatRp(customers.reduce((a, b) => a + b.debt, 0))}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Supplier Aktif</span>
      <span class="text-2xl font-black text-indigo-600">1 Supplier</span>
    </div>
  </div>

  <!-- Search & Actions -->
  <div class="bg-white p-4 rounded-2xl border border-slate-200 shadow-sm flex items-center justify-between gap-4">
    <div class="relative flex-grow max-w-md">
      <input 
        type="text" 
        placeholder="Cari ID, Nama, No. Telepon..." 
        class="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm font-semibold focus:outline-none focus:border-indigo-500"
        bind:value={searchQuery}
      />
      <span class="absolute left-3.5 top-3 text-slate-400">🔍</span>
    </div>
    <button class="px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-black rounded-xl shadow transition text-sm flex items-center gap-2">
      <span>➕</span> Tambah Entitas Baru
    </button>
  </div>

  <!-- Table -->
  <div class="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
    <table class="w-full text-left border-collapse">
      <thead>
        <tr class="bg-slate-50 border-b border-slate-200 text-xs font-bold text-slate-500 uppercase tracking-wider">
          <th class="p-4">ID</th>
          <th class="p-4">Nama Lengkap</th>
          <th class="p-4">No. Telepon</th>
          <th class="p-4">Kategori / Tipe</th>
          <th class="p-4 text-center">Poin</th>
          <th class="p-4 text-right">Piutang</th>
          <th class="p-4 text-center">Status</th>
          <th class="p-4 text-center">Aksi</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-100 text-sm font-medium text-slate-700">
        {#each filteredCustomers as cust}
          <tr class="hover:bg-slate-50/80 transition">
            <td class="p-4 font-mono font-bold text-indigo-600">{cust.id}</td>
            <td class="p-4 font-bold text-slate-900">{cust.name}</td>
            <td class="p-4 text-slate-500 font-mono text-xs">{cust.phone}</td>
            <td class="p-4">
              <span class="px-2.5 py-1 bg-slate-100 text-slate-700 rounded-lg text-xs font-bold">{cust.typeLabel}</span>
            </td>
            <td class="p-4 text-center font-mono font-bold text-amber-600">{cust.points}</td>
            <td class="p-4 text-right font-mono font-bold text-rose-600">{formatRp(cust.debt)}</td>
            <td class="p-4 text-center">
              <span class="px-3 py-1 bg-emerald-100 text-emerald-800 rounded-full text-xs font-bold">{cust.status}</span>
            </td>
            <td class="p-4 text-center">
              <button class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-bold transition">
                Edit
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<script lang="ts">
  let searchQuery = $state('');
  let activeTab: 'members' | 'shu' | 'savings' | 'credit' = $state('members');
  let isLoading = $state(false);

  let members = $state([
    { id: 'KOP-001', name: 'Bambang Sudirman', nip: '1982031001', department: 'Divisi Operasional', limit: 2000000, usedCredit: 450000, savings: 5000000, shuEstimate: 750000 },
    { id: 'KOP-002', name: 'Siti Nurhaliza', nip: '1989051202', department: 'Divisi Keuangan', limit: 3000000, usedCredit: 1200000, savings: 8500000, shuEstimate: 1250000 },
    { id: 'KOP-003', name: 'Rudi Hermawan', nip: '1991072203', department: 'Divisi IT', limit: 2500000, usedCredit: 0, savings: 3200000, shuEstimate: 480000 },
    { id: 'KOP-004', name: 'Dewi Lestari', nip: '1995110404', department: 'Divisi HRD', limit: 2000000, usedCredit: 850000, savings: 4100000, shuEstimate: 620000 },
  ]);

  let filteredMembers = $derived(
    members.filter(m => m.name.toLowerCase().includes(searchQuery.toLowerCase()) || m.id.toLowerCase().includes(searchQuery.toLowerCase()) || m.department.toLowerCase().includes(searchQuery.toLowerCase()))
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
        <span class="p-2 bg-amber-500 text-slate-950 rounded-xl">🏢</span> Toko Koperasi Anggota & SHU
      </h1>
      <p class="text-xs text-slate-400 mt-1">G. Manajemen Anggota Koperasi, Simpanan, Saldo Kredit, Potong Gaji & Kalkulasi SHU</p>
    </div>
    <div class="flex gap-2 bg-slate-800 p-1.5 rounded-xl border border-slate-700">
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'members' ? 'bg-amber-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'members'}>👥 Data Anggota</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'shu' ? 'bg-amber-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'shu'}>📊 Poin & SHU</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'credit' ? 'bg-amber-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'credit'}>💳 Limit & Potong Gaji</button>
    </div>
  </div>

  <!-- Metric Cards -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Anggota Aktif</span>
      <span class="text-2xl font-black text-slate-900">{members.length} Orang</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Simpanan Pokok/Wajib</span>
      <span class="text-2xl font-black text-emerald-600">{formatRp(members.reduce((a, b) => a + b.savings, 0))}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Piutang Belanja</span>
      <span class="text-2xl font-black text-rose-600">{formatRp(members.reduce((a, b) => a + b.usedCredit, 0))}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Estimasi SHU Tahun Ini</span>
      <span class="text-2xl font-black text-amber-600">{formatRp(members.reduce((a, b) => a + b.shuEstimate, 0))}</span>
    </div>
  </div>

  <!-- Search & Actions -->
  <div class="bg-white p-4 rounded-2xl border border-slate-200 shadow-sm flex items-center justify-between gap-4">
    <div class="relative flex-grow max-w-md">
      <input 
        type="text" 
        placeholder="Cari NAMA, ID Anggota, NIP, atau Departemen..." 
        class="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm font-semibold focus:outline-none focus:border-amber-500"
        bind:value={searchQuery}
      />
      <span class="absolute left-3.5 top-3 text-slate-400">🔍</span>
    </div>
    <button class="px-5 py-2.5 bg-amber-500 hover:bg-amber-400 text-slate-950 font-black rounded-xl shadow transition text-sm flex items-center gap-2">
      <span>➕</span> Tambah Anggota Koperasi
    </button>
  </div>

  <!-- Data Table -->
  <div class="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
    <table class="w-full text-left border-collapse">
      <thead>
        <tr class="bg-slate-50 border-b border-slate-200 text-xs font-bold text-slate-500 uppercase tracking-wider">
          <th class="p-4">ID / NIP</th>
          <th class="p-4">Nama Anggota</th>
          <th class="p-4">Departemen</th>
          <th class="p-4 text-right">Limit Belanja</th>
          <th class="p-4 text-right">Piutang / Terpakai</th>
          <th class="p-4 text-right">Simpanan Total</th>
          <th class="p-4 text-right">Estimasi SHU</th>
          <th class="p-4 text-center">Aksi</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-100 text-sm font-medium text-slate-700">
        {#each filteredMembers as member}
          <tr class="hover:bg-slate-50/80 transition">
            <td class="p-4 font-mono font-bold text-amber-600">{member.id}</td>
            <td class="p-4 font-bold text-slate-900">{member.name}</td>
            <td class="p-4 text-slate-500">{member.department}</td>
            <td class="p-4 text-right font-mono">{formatRp(member.limit)}</td>
            <td class="p-4 text-right font-mono text-rose-600 font-bold">{formatRp(member.usedCredit)}</td>
            <td class="p-4 text-right font-mono text-emerald-600 font-bold">{formatRp(member.savings)}</td>
            <td class="p-4 text-right font-mono text-amber-600 font-bold">{formatRp(member.shuEstimate)}</td>
            <td class="p-4 text-center">
              <button class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-bold transition">
                Kelola
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

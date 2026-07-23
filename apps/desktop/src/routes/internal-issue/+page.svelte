<script lang="ts">
  let searchQuery = $state('');
  let activeTab: 'requests' | 'inventory' | 'transfers' | 'departments' = $state('requests');

  let internalRequests = $state([
    { reqNo: 'REQ-2026-001', department: 'Divisi HRD & GA', requester: 'Siti Rahma', item: 'Kertas HVS A4 70gr', qty: 10, unit: 'Rim', costCenter: 'CC-HRD-01', status: 'Disetujui' },
    { reqNo: 'REQ-2026-002', department: 'Divisi Operasional', requester: 'Budi Santoso', item: 'Tinta Printer Epson Black', qty: 2, unit: 'Botol', costCenter: 'CC-OPS-02', status: 'Menunggu Approval' },
    { reqNo: 'REQ-2026-003', department: 'Pantry Kantor', requester: 'Sri Lestari', item: 'Kopi Kapal Api 165g', qty: 5, unit: 'Bungkus', costCenter: 'CC-PANTRY-01', status: 'Selesai Serah Terima' },
  ]);

  function getStatusBadge(status: string) {
    switch (status) {
      case 'Disetujui': return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'Selesai Serah Terima': return 'bg-emerald-100 text-emerald-800 border-emerald-200';
      default: return 'bg-amber-100 text-amber-800 border-amber-200';
    }
  }
</script>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- Header Banner -->
  <div class="bg-slate-900 text-white p-6 rounded-2xl border border-slate-800 shadow-xl flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-black flex items-center gap-3">
        <span class="p-2 bg-indigo-600 rounded-xl">📦</span> Disposisi Gudang & Pengeluaran Internal
      </h1>
      <p class="text-xs text-slate-400 mt-1">I. Pengeluaran Barang Pantry/Kantor, Cost Center, Transfer Antar Gudang & Tanpa Pendapatan</p>
    </div>
    <span class="px-3.5 py-1.5 bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 rounded-xl text-xs font-bold uppercase tracking-wider">
      Cost Center Internal Mode
    </span>
  </div>

  <!-- Metric Cards -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Permintaan Bulan Ini</span>
      <span class="text-2xl font-black text-slate-900">48 Permintaan</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Menunggu Persetujuan</span>
      <span class="text-2xl font-black text-amber-600">3 Dokumen</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Pengeluaran Cost Center</span>
      <span class="text-2xl font-black text-indigo-600">Rp 12.450.000</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Gudang Utama</span>
      <span class="text-2xl font-black text-emerald-600">Gudang A (Pusat)</span>
    </div>
  </div>

  <!-- Actions & Table -->
  <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-4">
    <div class="flex items-center justify-between gap-4">
      <div class="relative flex-grow max-w-md">
        <input 
          type="text" 
          placeholder="Cari No. Permintaan, Departemen, atau Pemohon..." 
          class="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm font-semibold focus:outline-none focus:border-indigo-500"
          bind:value={searchQuery}
        />
        <span class="absolute left-3.5 top-3 text-slate-400">🔍</span>
      </div>
      <button class="px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-black rounded-xl shadow transition text-sm flex items-center gap-2">
        <span>➕</span> Buat Permintaan Barang Internal Baru
      </button>
    </div>

    <div class="overflow-hidden rounded-xl border border-slate-200">
      <table class="w-full text-left border-collapse">
        <thead>
          <tr class="bg-slate-50 border-b border-slate-200 text-xs font-bold text-slate-500 uppercase tracking-wider">
            <th class="p-4">No. Permintaan</th>
            <th class="p-4">Departemen / Cost Center</th>
            <th class="p-4">Pemohon</th>
            <th class="p-4">Barang</th>
            <th class="p-4 text-center">Jumlah</th>
            <th class="p-4 text-center">Status</th>
            <th class="p-4 text-center">Aksi</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100 text-sm font-medium text-slate-700">
          {#each internalRequests as req}
            <tr class="hover:bg-slate-50/80 transition">
              <td class="p-4 font-mono font-bold text-indigo-600">{req.reqNo}</td>
              <td class="p-4 font-bold text-slate-900">{req.department} <span class="text-xs text-slate-400 block font-mono">{req.costCenter}</span></td>
              <td class="p-4 text-slate-600">{req.requester}</td>
              <td class="p-4 font-bold text-slate-800">{req.item}</td>
              <td class="p-4 text-center font-mono font-bold">{req.qty} {req.unit}</td>
              <td class="p-4 text-center">
                <span class="px-3 py-1 rounded-full text-xs font-bold border {getStatusBadge(req.status)}">{req.status}</span>
              </td>
              <td class="p-4 text-center">
                <button class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-bold transition">
                  Detail & Serah Terima
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

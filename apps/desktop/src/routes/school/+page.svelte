<script lang="ts">
  let searchQuery = $state('');
  let activeTab: 'students' | 'preorder' | 'limit' | 'cards' = $state('students');

  let students = $state([
    { nis: '20261001', name: 'Ahmad Fauzi', class: 'X IPA 1', walletBalance: 150000, dailyLimit: 50000, rfidCard: 'RFID-992102', status: 'Aktif' },
    { nis: '20261002', name: 'Nabila Putri', class: 'X IPS 2', walletBalance: 85000, dailyLimit: 30000, rfidCard: 'RFID-992103', status: 'Aktif' },
    { nis: '20261003', name: 'Kevin Pratama', class: 'XI IPA 3', walletBalance: 200000, dailyLimit: 50000, rfidCard: 'RFID-992104', status: 'Aktif' },
  ]);

  let filteredStudents = $derived(
    students.filter(s => s.name.toLowerCase().includes(searchQuery.toLowerCase()) || s.nis.includes(searchQuery) || s.class.toLowerCase().includes(searchQuery.toLowerCase()))
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
        <span class="p-2 bg-emerald-600 rounded-xl">🏫</span> Kantin & Toko Sekolah (Wallet Siswa)
      </h1>
      <p class="text-xs text-slate-400 mt-1">J. Saldo Wallet RFID Siswa, Limit Belanja Harian, Pre-Order & Harga Khusus Siswa</p>
    </div>
    <div class="flex gap-2 bg-slate-800 p-1.5 rounded-xl border border-slate-700">
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'students' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'students'}>👨‍🎓 Siswa & Wallet</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'preorder' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'preorder'}>🍱 Pesanan Pre-Order</button>
      <button class="px-4 py-2 rounded-lg font-bold text-xs transition {activeTab === 'limit' ? 'bg-emerald-500 text-slate-950 shadow' : 'text-slate-400 hover:text-white'}" onclick={() => activeTab = 'limit'}>🔒 Limit Belanja Harian</button>
    </div>
  </div>

  <!-- Metric Cards -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Siswa Terdaftar</span>
      <span class="text-2xl font-black text-slate-900">{students.length} Siswa</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Total Saldo Wallet Kantin</span>
      <span class="text-2xl font-black text-emerald-600">{formatRp(students.reduce((a, b) => a + b.walletBalance, 0))}</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Transaksi Kantin Hari Ini</span>
      <span class="text-2xl font-black text-blue-600">142 Transaksi</span>
    </div>
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-1">
      <span class="text-xs font-bold text-slate-400 uppercase tracking-wider block">Topup Hari Ini</span>
      <span class="text-2xl font-black text-amber-600">{formatRp(1250000)}</span>
    </div>
  </div>

  <!-- Search & Actions -->
  <div class="bg-white p-4 rounded-2xl border border-slate-200 shadow-sm flex items-center justify-between gap-4">
    <div class="relative flex-grow max-w-md">
      <input 
        type="text" 
        placeholder="Cari Nama Siswa, NIS, RFID, atau Kelas..." 
        class="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm font-semibold focus:outline-none focus:border-emerald-500"
        bind:value={searchQuery}
      />
      <span class="absolute left-3.5 top-3 text-slate-400">🔍</span>
    </div>
    <button class="px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-black rounded-xl shadow transition text-sm flex items-center gap-2">
      <span>💳</span> Topup Saldo Wallet Siswa
    </button>
  </div>

  <!-- Table -->
  <div class="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
    <table class="w-full text-left border-collapse">
      <thead>
        <tr class="bg-slate-50 border-b border-slate-200 text-xs font-bold text-slate-500 uppercase tracking-wider">
          <th class="p-4">NIS</th>
          <th class="p-4">Nama Siswa</th>
          <th class="p-4">Kelas</th>
          <th class="p-4">ID Kartu RFID</th>
          <th class="p-4 text-right">Saldo Wallet</th>
          <th class="p-4 text-right">Limit Harian</th>
          <th class="p-4 text-center">Status</th>
          <th class="p-4 text-center">Aksi</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-100 text-sm font-medium text-slate-700">
        {#each filteredStudents as student}
          <tr class="hover:bg-slate-50/80 transition">
            <td class="p-4 font-mono font-bold text-emerald-600">{student.nis}</td>
            <td class="p-4 font-bold text-slate-900">{student.name}</td>
            <td class="p-4 text-slate-500">{student.class}</td>
            <td class="p-4 font-mono text-xs text-slate-600">{student.rfidCard}</td>
            <td class="p-4 text-right font-mono font-black text-emerald-600">{formatRp(student.walletBalance)}</td>
            <td class="p-4 text-right font-mono text-amber-600 font-bold">{formatRp(student.dailyLimit)}</td>
            <td class="p-4 text-center">
              <span class="px-3 py-1 bg-emerald-100 text-emerald-800 rounded-full text-xs font-bold">{student.status}</span>
            </td>
            <td class="p-4 text-center">
              <button class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-bold transition">
                Topup / Limit
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

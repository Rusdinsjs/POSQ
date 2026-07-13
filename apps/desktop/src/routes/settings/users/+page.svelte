<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { authState } from '$lib/auth.svelte';
    import BackButton from '$lib/components/BackButton.svelte';

    // ── Types ──────────────────────────────────────────────────────────────
    interface UserListItem {
        id: string;
        name: string;
        status: string;
        outlet_id: string | null;
        roles: string[];
        created_at: string;
    }
    interface UserDetail {
        id: string;
        name: string;
        status: string;
        outlet_id: string | null;
        failed_login_attempts: number;
        locked_until: string | null;
        created_at: string;
        roles: RoleAssignment[];
    }
    interface RoleAssignment {
        id: string;
        role_id: string;
        role_name: string;
        outlet_id: string;
        status: string;
        valid_from: string;
        valid_until: string;
        assigned_by: string;
    }
    interface RoleItem {
        id: string;
        name: string;
        system_role: boolean;
    }

    // ── State ──────────────────────────────────────────────────────────────
    let users = $state<UserListItem[]>([]);
    let roles = $state<RoleItem[]>([]);
    let isLoading = $state(true);
    let accessDenied = $state(false);
    let toastMessage = $state('');
    let toastType = $state<'success' | 'error'>('success');

    // Selected user detail panel
    let selectedUser = $state<UserDetail | null>(null);
    let isLoadingDetail = $state(false);

    // Create user modal
    let showCreateModal = $state(false);
    let newUserName = $state('');
    let newUserPin = $state('');
    let newUserRoleId = $state('');
    let isCreating = $state(false);

    // Edit name modal
    let showEditNameModal = $state(false);
    let editNameValue = $state('');

    // Reset PIN modal
    let showResetPinModal = $state(false);
    let resetPinValue = $state('');
    let isResettingPin = $state(false);

    // Assign role modal
    let showAssignRoleModal = $state(false);
    let assignRoleId = $state('');
    let isAssigningRole = $state(false);

    let isUpdating = $state(false);

    // Filter / search
    let searchQuery = $state('');
    let filterStatus = $state<'all' | 'active' | 'inactive'>('all');

    // ── Computed ───────────────────────────────────────────────────────────
    let filteredUsers = $derived(users.filter(u => {
        const matchSearch = u.name.toLowerCase().includes(searchQuery.toLowerCase());
        const matchStatus = filterStatus === 'all' || u.status === filterStatus;
        return matchSearch && matchStatus;
    }));

    // ── Helpers ────────────────────────────────────────────────────────────
    function showToast(msg: string, type: 'success' | 'error' = 'success') {
        toastMessage = msg;
        toastType = type;
        setTimeout(() => { toastMessage = ''; }, 3000);
    }

    function formatDate(dt: string) {
        try {
            return new Intl.DateTimeFormat('id-ID', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(dt));
        } catch { return dt; }
    }

    function roleBadgeColor(name: string) {
        const map: Record<string, string> = {
            owner:      'bg-violet-100 text-violet-700 border-violet-200',
            manager:    'bg-blue-100 text-blue-700 border-blue-200',
            supervisor: 'bg-amber-100 text-amber-700 border-amber-200',
            cashier:    'bg-emerald-100 text-emerald-700 border-emerald-200',
            inventory:  'bg-cyan-100 text-cyan-700 border-cyan-200',
            finance:    'bg-rose-100 text-rose-700 border-rose-200',
        };
        return map[name] ?? 'bg-slate-100 text-slate-700 border-slate-200';
    }

    function statusColor(s: string) {
        return s === 'active'
            ? 'bg-emerald-100 text-emerald-700 border-emerald-200'
            : 'bg-rose-100 text-rose-600 border-rose-200';
    }

    // ── Data Loading ───────────────────────────────────────────────────────
    async function loadAll() {
        isLoading = true;
        try {
            [users, roles] = await Promise.all([
                invoke<UserListItem[]>('list_users'),
                invoke<RoleItem[]>('list_roles'),
            ]);
            if (roles.length > 0) assignRoleId = roles[0].id;
        } catch (e: any) {
            if (String(e).includes('Akses ditolak')) {
                accessDenied = true;
            } else {
                showToast('Gagal memuat data: ' + e, 'error');
            }
        } finally {
            isLoading = false;
        }
    }

    async function loadUserDetail(userId: string) {
        isLoadingDetail = true;
        selectedUser = null;
        try {
            selectedUser = await invoke<UserDetail>('get_user_detail', { userId });
        } catch (e: any) {
            showToast('Gagal memuat detail: ' + e, 'error');
        } finally {
            isLoadingDetail = false;
        }
    }

    onMount(loadAll);

    // ── Actions ────────────────────────────────────────────────────────────
    async function handleCreateUser() {
        if (!newUserName.trim() || !newUserPin || !newUserRoleId) {
            showToast('Lengkapi semua field terlebih dahulu', 'error');
            return;
        }
        isCreating = true;
        try {
            await invoke('create_user', {
                name: newUserName.trim(),
                initialPin: newUserPin,
                roleId: newUserRoleId,
            });
            showToast(`Pengguna '${newUserName}' berhasil dibuat`, 'success');
            showCreateModal = false;
            newUserName = ''; newUserPin = ''; newUserRoleId = roles[0]?.id ?? '';
            await loadAll();
        } catch (e: any) {
            showToast('Gagal: ' + e, 'error');
        } finally {
            isCreating = false;
        }
    }

    async function handleToggleStatus() {
        if (!selectedUser) return;
        const next = selectedUser.status === 'active' ? 'inactive' : 'active';
        isUpdating = true;
        try {
            await invoke('update_user_status', { userId: selectedUser.id, newStatus: next });
            showToast(`Status berhasil diubah menjadi '${next}'`, 'success');
            await loadAll();
            await loadUserDetail(selectedUser.id);
        } catch (e: any) {
            showToast('Gagal: ' + e, 'error');
        } finally {
            isUpdating = false;
        }
    }

    async function handleUnlock() {
        if (!selectedUser) return;
        isUpdating = true;
        try {
            await invoke('unlock_user', { userId: selectedUser.id });
            showToast('Akun berhasil dibuka', 'success');
            await loadUserDetail(selectedUser.id);
        } catch (e: any) {
            showToast('Gagal: ' + e, 'error');
        } finally {
            isUpdating = false;
        }
    }

    async function handleEditName() {
        if (!selectedUser || !editNameValue.trim()) return;
        isUpdating = true;
        try {
            await invoke('update_user_name', { userId: selectedUser.id, newName: editNameValue.trim() });
            showToast('Nama berhasil diperbarui', 'success');
            showEditNameModal = false;
            await loadAll();
            await loadUserDetail(selectedUser.id);
        } catch (e: any) {
            showToast('Gagal: ' + e, 'error');
        } finally {
            isUpdating = false;
        }
    }

    async function handleResetPin() {
        if (!selectedUser || resetPinValue.length < 4) {
            showToast('PIN minimal 4 digit', 'error');
            return;
        }
        isResettingPin = true;
        try {
            await invoke('reset_user_pin', { userId: selectedUser.id, newPin: resetPinValue });
            showToast('PIN berhasil direset', 'success');
            showResetPinModal = false;
            resetPinValue = '';
        } catch (e: any) {
            showToast('Gagal reset PIN: ' + e, 'error');
        } finally {
            isResettingPin = false;
        }
    }

    async function handleAssignRole() {
        if (!selectedUser || !assignRoleId) return;
        isAssigningRole = true;
        try {
            await invoke('assign_user_role', { userId: selectedUser.id, roleId: assignRoleId });
            showToast('Role berhasil di-assign', 'success');
            showAssignRoleModal = false;
            await loadUserDetail(selectedUser.id);
            await loadAll();
        } catch (e: any) {
            showToast('Gagal: ' + e, 'error');
        } finally {
            isAssigningRole = false;
        }
    }

    async function handleRevokeRole(uorId: string) {
        if (!selectedUser) return;
        if (!confirm('Cabut role ini dari pengguna?')) return;
        try {
            await invoke('revoke_user_role', { userOutletRoleId: uorId });
            showToast('Role berhasil dicabut', 'success');
            await loadUserDetail(selectedUser.id);
            await loadAll();
        } catch (e: any) {
            showToast('Gagal: ' + e, 'error');
        }
    }
</script>

<svelte:head>
    <title>Manajemen Pengguna — POSQ</title>
    <meta name="description" content="Kelola pengguna, role, dan hak akses di sistem POSQ." />
</svelte:head>

<!-- Toast -->
{#if toastMessage}
    <div class="fixed bottom-6 right-6 z-[100] px-5 py-3.5 rounded-xl shadow-2xl text-sm font-bold border flex items-center gap-2 animate-in slide-in-from-bottom-3 duration-200
        {toastType === 'success' ? 'bg-emerald-50 text-emerald-800 border-emerald-200' : 'bg-rose-50 text-rose-800 border-rose-200'}">
        <span>{toastType === 'success' ? '✅' : '❌'}</span>
        {toastMessage}
    </div>
{/if}

<div class="min-h-screen bg-slate-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <!-- Page Header -->
        <div class="mb-8 flex items-center gap-4">
            <BackButton to="/settings" label="Kembali ke Pengaturan" />
            <div>
                <h1 class="text-2xl font-black text-slate-900 tracking-tight">Manajemen Pengguna</h1>
                <p class="text-sm text-slate-500 mt-0.5">Kelola pengguna, role, dan hak akses kasir</p>
            </div>
        </div>

        <!-- Access Denied -->
        {#if accessDenied}
            <div class="flex flex-col items-center justify-center py-24 gap-4 text-center">
                <div class="w-16 h-16 rounded-2xl bg-rose-100 flex items-center justify-center text-3xl">🔒</div>
                <h2 class="text-xl font-bold text-slate-800">Akses Ditolak</h2>
                <p class="text-sm text-slate-500 max-w-sm">Anda tidak memiliki izin untuk mengakses manajemen pengguna. Hubungi Owner atau Manager.</p>
            </div>

        {:else if isLoading}
            <div class="flex items-center justify-center py-24">
                <div class="w-10 h-10 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
            </div>

        {:else}
            <!-- Main Layout: List + Detail Panel -->
            <div class="flex gap-6 items-start">

                <!-- LEFT: User List -->
                <div class="flex-1 min-w-0 bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
                    <!-- List Header -->
                    <div class="px-5 py-4 border-b border-slate-100 flex items-center justify-between gap-3 flex-wrap">
                        <div class="flex items-center gap-3 flex-1">
                            <div class="relative flex-1 max-w-xs">
                                <span class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 text-sm">🔍</span>
                                <input
                                    id="user-search"
                                    type="text"
                                    bind:value={searchQuery}
                                    placeholder="Cari nama pengguna..."
                                    class="w-full pl-9 pr-3 py-2 text-sm border border-slate-200 rounded-xl bg-slate-50 focus:outline-none focus:ring-2 focus:ring-blue-300 focus:border-transparent"
                                />
                            </div>
                            <select
                                id="filter-status"
                                bind:value={filterStatus}
                                class="text-xs font-bold border border-slate-200 rounded-xl px-3 py-2 bg-slate-50 focus:outline-none focus:ring-2 focus:ring-blue-300 cursor-pointer"
                            >
                                <option value="all">Semua Status</option>
                                <option value="active">Aktif</option>
                                <option value="inactive">Nonaktif</option>
                            </select>
                        </div>
                        <button
                            id="btn-create-user"
                            type="button"
                            onclick={() => { showCreateModal = true; newUserRoleId = roles[0]?.id ?? ''; }}
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-bold rounded-xl transition-colors shadow-sm cursor-pointer flex items-center gap-1.5"
                        >
                            <span>➕</span> Tambah Pengguna
                        </button>
                    </div>

                    <!-- User count -->
                    <div class="px-5 py-2 bg-slate-50 border-b border-slate-100">
                        <span class="text-xs text-slate-500 font-semibold">{filteredUsers.length} pengguna ditemukan</span>
                    </div>

                    <!-- User Rows -->
                    <div class="divide-y divide-slate-50">
                        {#each filteredUsers as user (user.id)}
                            <button
                                type="button"
                                id="user-row-{user.id}"
                                onclick={() => { loadUserDetail(user.id); }}
                                class="w-full text-left px-5 py-4 hover:bg-blue-50/60 transition-colors cursor-pointer
                                       {selectedUser?.id === user.id ? 'bg-blue-50 border-l-2 border-l-blue-500' : 'border-l-2 border-l-transparent'}"
                            >
                                <div class="flex items-center justify-between gap-3">
                                    <div class="flex items-center gap-3 min-w-0">
                                        <!-- Avatar -->
                                        <div class="w-9 h-9 rounded-full flex items-center justify-center text-sm font-black text-white flex-shrink-0
                                            {user.status === 'active' ? 'bg-gradient-to-br from-blue-500 to-violet-600' : 'bg-slate-300'}">
                                            {user.name.charAt(0).toUpperCase()}
                                        </div>
                                        <div class="min-w-0">
                                            <div class="text-sm font-bold text-slate-800 truncate">{user.name}</div>
                                            <div class="flex flex-wrap gap-1 mt-1">
                                                {#each user.roles as role}
                                                    <span class="px-1.5 py-0.5 rounded-md text-[10px] font-bold border {roleBadgeColor(role)}">{role}</span>
                                                {/each}
                                                {#if user.roles.length === 0}
                                                    <span class="text-[10px] text-slate-400">— tanpa role</span>
                                                {/if}
                                            </div>
                                        </div>
                                    </div>
                                    <span class="text-[10px] font-bold px-2 py-0.5 rounded-lg border flex-shrink-0 {statusColor(user.status)}">
                                        {user.status === 'active' ? 'Aktif' : 'Nonaktif'}
                                    </span>
                                </div>
                            </button>
                        {/each}

                        {#if filteredUsers.length === 0}
                            <div class="px-5 py-12 text-center text-slate-400 text-sm">
                                <div class="text-3xl mb-2">👤</div>
                                Tidak ada pengguna ditemukan
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- RIGHT: Detail Panel -->
                <div class="w-96 flex-shrink-0">
                    {#if isLoadingDetail}
                        <div class="bg-white rounded-2xl border border-slate-200 shadow-sm p-10 flex justify-center">
                            <div class="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                        </div>

                    {:else if selectedUser}
                        <div class="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
                            <!-- User Header -->
                            <div class="bg-gradient-to-br from-blue-600 to-violet-700 px-6 py-6 text-white">
                                <div class="flex items-center gap-4">
                                    <div class="w-14 h-14 rounded-2xl bg-white/20 flex items-center justify-center text-2xl font-black">
                                        {selectedUser.name.charAt(0).toUpperCase()}
                                    </div>
                                    <div>
                                        <div class="text-lg font-black">{selectedUser.name}</div>
                                        <div class="text-xs text-blue-200 mt-0.5">
                                            Bergabung {formatDate(selectedUser.created_at)}
                                        </div>
                                        <div class="flex items-center gap-1.5 mt-2">
                                            <span class="px-2 py-0.5 rounded-lg text-[10px] font-bold
                                                {selectedUser.status === 'active' ? 'bg-emerald-400/30 text-emerald-100' : 'bg-rose-400/30 text-rose-100'}">
                                                {selectedUser.status === 'active' ? '● Aktif' : '● Nonaktif'}
                                            </span>
                                            {#if selectedUser.locked_until}
                                                <span class="px-2 py-0.5 rounded-lg text-[10px] font-bold bg-amber-400/30 text-amber-100">🔒 Terkunci</span>
                                            {/if}
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="p-5 space-y-4">
                                <!-- Login Stats -->
                                {#if selectedUser.failed_login_attempts > 0}
                                    <div class="bg-amber-50 border border-amber-200 rounded-xl p-3 text-xs text-amber-800 font-semibold flex items-center gap-2">
                                        <span>⚠️</span>
                                        {selectedUser.failed_login_attempts}x gagal login
                                        {#if selectedUser.locked_until}
                                            — terkunci hingga {formatDate(selectedUser.locked_until)}
                                        {/if}
                                    </div>
                                {/if}

                                <!-- Actions -->
                                <div class="grid grid-cols-2 gap-2">
                                    <button
                                        id="btn-edit-name"
                                        type="button"
                                        onclick={() => { editNameValue = selectedUser!.name; showEditNameModal = true; }}
                                        class="px-3 py-2.5 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl text-xs font-bold text-slate-700 transition cursor-pointer"
                                    >
                                        ✏️ Edit Nama
                                    </button>
                                    <button
                                        id="btn-reset-pin"
                                        type="button"
                                        onclick={() => { resetPinValue = ''; showResetPinModal = true; }}
                                        class="px-3 py-2.5 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl text-xs font-bold text-slate-700 transition cursor-pointer"
                                    >
                                        🔑 Reset PIN
                                    </button>
                                    {#if selectedUser.locked_until || selectedUser.failed_login_attempts > 0}
                                        <button
                                            id="btn-unlock"
                                            type="button"
                                            disabled={isUpdating}
                                            onclick={handleUnlock}
                                            class="px-3 py-2.5 bg-amber-50 hover:bg-amber-100 border border-amber-200 rounded-xl text-xs font-bold text-amber-700 transition cursor-pointer disabled:opacity-50"
                                        >
                                            🔓 Buka Kunci
                                        </button>
                                    {/if}
                                    <button
                                        id="btn-toggle-status"
                                        type="button"
                                        disabled={isUpdating}
                                        onclick={handleToggleStatus}
                                        class="px-3 py-2.5 rounded-xl text-xs font-bold transition cursor-pointer disabled:opacity-50 border
                                            {selectedUser.status === 'active'
                                                ? 'bg-rose-50 hover:bg-rose-100 border-rose-200 text-rose-700'
                                                : 'bg-emerald-50 hover:bg-emerald-100 border-emerald-200 text-emerald-700'}"
                                    >
                                        {selectedUser.status === 'active' ? '🚫 Nonaktifkan' : '✅ Aktifkan'}
                                    </button>
                                </div>

                                <!-- Roles Section -->
                                <div>
                                    <div class="flex items-center justify-between mb-2">
                                        <span class="text-xs font-black text-slate-700 uppercase tracking-wider">Role</span>
                                        <button
                                            id="btn-assign-role"
                                            type="button"
                                            onclick={() => { showAssignRoleModal = true; }}
                                            class="text-[10px] font-bold text-blue-600 hover:text-blue-700 cursor-pointer transition"
                                        >
                                            + Assign Role
                                        </button>
                                    </div>

                                    <div class="space-y-2">
                                        {#each selectedUser.roles as ra (ra.id)}
                                            <div class="flex items-center justify-between bg-slate-50 rounded-xl px-3 py-2.5 border border-slate-100">
                                                <div class="flex flex-col gap-0.5">
                                                    <span class="text-xs font-bold text-slate-800 flex items-center gap-1.5">
                                                        <span class="w-2 h-2 rounded-full {ra.status === 'ACTIVE' ? 'bg-emerald-400' : 'bg-slate-300'}"></span>
                                                        {ra.role_name}
                                                    </span>
                                                    <span class="text-[10px] text-slate-400">
                                                        Berlaku hingga {formatDate(ra.valid_until)}
                                                    </span>
                                                </div>
                                                {#if ra.status === 'ACTIVE'}
                                                    <button
                                                        id="btn-revoke-role-{ra.id}"
                                                        type="button"
                                                        onclick={() => handleRevokeRole(ra.id)}
                                                        class="text-[10px] text-rose-500 hover:text-rose-700 font-bold cursor-pointer transition"
                                                    >
                                                        Cabut
                                                    </button>
                                                {/if}
                                            </div>
                                        {/each}
                                        {#if selectedUser.roles.filter(r => r.status === 'ACTIVE').length === 0}
                                            <div class="text-xs text-slate-400 text-center py-3">Tidak ada role aktif</div>
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        </div>

                    {:else}
                        <!-- Empty State -->
                        <div class="bg-white rounded-2xl border border-slate-200 shadow-sm p-10 text-center text-slate-400">
                            <div class="text-4xl mb-3">👈</div>
                            <p class="text-sm font-semibold">Pilih pengguna untuk melihat detail</p>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</div>

<!-- ── Modal: Tambah Pengguna ──────────────────────────────────────────────── -->
{#if showCreateModal}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-xs p-4">
        <div class="bg-white rounded-2xl shadow-2xl border border-slate-200 w-full max-w-md overflow-hidden">
            <div class="bg-gradient-to-br from-blue-600 to-violet-700 px-6 py-5 text-white">
                <h2 class="text-base font-black">Tambah Pengguna Baru</h2>
                <p class="text-xs text-blue-200 mt-0.5">Buat akun kasir, supervisor, atau staf lainnya</p>
            </div>
            <div class="p-6 space-y-4">
                <div class="flex flex-col gap-1.5">
                    <label for="new-user-name" class="text-xs font-bold text-slate-600 uppercase tracking-wider">Nama Pengguna</label>
                    <input
                        id="new-user-name"
                        type="text"
                        bind:value={newUserName}
                        placeholder="contoh: Budi Kasir 1"
                        class="w-full px-3 py-2.5 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-300 bg-slate-50"
                    />
                </div>
                <div class="flex flex-col gap-1.5">
                    <label for="new-user-pin" class="text-xs font-bold text-slate-600 uppercase tracking-wider">PIN Awal (min. 4 digit)</label>
                    <input
                        id="new-user-pin"
                        type="password"
                        bind:value={newUserPin}
                        placeholder="••••••"
                        maxlength="12"
                        class="w-full px-3 py-2.5 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-300 bg-slate-50"
                    />
                    <p class="text-[10px] text-amber-600 font-semibold">⚠️ PIN ini hanya ditampilkan sekali. Catat sebelum menyimpan.</p>
                </div>
                <div class="flex flex-col gap-1.5">
                    <label for="new-user-role" class="text-xs font-bold text-slate-600 uppercase tracking-wider">Role</label>
                    <select
                        id="new-user-role"
                        bind:value={newUserRoleId}
                        class="w-full px-3 py-2.5 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-300 bg-slate-50 cursor-pointer"
                    >
                        {#each roles as role}
                            <option value={role.id}>{role.name}</option>
                        {/each}
                    </select>
                </div>
            </div>
            <div class="px-6 pb-6 flex gap-3 justify-end">
                <button
                    id="btn-cancel-create"
                    type="button"
                    onclick={() => showCreateModal = false}
                    class="px-4 py-2.5 text-xs font-bold text-slate-600 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl transition cursor-pointer"
                >
                    Batal
                </button>
                <button
                    id="btn-confirm-create"
                    type="button"
                    disabled={isCreating}
                    onclick={handleCreateUser}
                    class="px-5 py-2.5 text-xs font-bold text-white bg-blue-600 hover:bg-blue-700 rounded-xl transition shadow-sm cursor-pointer disabled:opacity-60 flex items-center gap-2"
                >
                    {#if isCreating}
                        <span class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                    {/if}
                    Simpan
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- ── Modal: Edit Nama ────────────────────────────────────────────────────── -->
{#if showEditNameModal && selectedUser}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-xs p-4">
        <div class="bg-white rounded-2xl shadow-2xl border border-slate-200 w-full max-w-sm overflow-hidden">
            <div class="px-6 py-5 border-b border-slate-100">
                <h2 class="text-sm font-black text-slate-800">Edit Nama Pengguna</h2>
            </div>
            <div class="p-6">
                <div class="flex flex-col gap-1.5">
                    <label for="edit-name-input" class="text-xs font-bold text-slate-600 uppercase tracking-wider">Nama Baru</label>
                    <input
                        id="edit-name-input"
                        type="text"
                        bind:value={editNameValue}
                        class="w-full px-3 py-2.5 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-300 bg-slate-50"
                    />
                </div>
            </div>
            <div class="px-6 pb-6 flex gap-3 justify-end">
                <button type="button" onclick={() => showEditNameModal = false} id="btn-cancel-editname"
                    class="px-4 py-2.5 text-xs font-bold text-slate-600 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl cursor-pointer transition">
                    Batal
                </button>
                <button type="button" onclick={handleEditName} disabled={isUpdating} id="btn-confirm-editname"
                    class="px-5 py-2.5 text-xs font-bold text-white bg-blue-600 hover:bg-blue-700 rounded-xl cursor-pointer transition disabled:opacity-60">
                    Simpan
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- ── Modal: Reset PIN ────────────────────────────────────────────────────── -->
{#if showResetPinModal && selectedUser}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-xs p-4">
        <div class="bg-white rounded-2xl shadow-2xl border border-slate-200 w-full max-w-sm overflow-hidden">
            <div class="px-6 py-5 border-b border-slate-100">
                <h2 class="text-sm font-black text-slate-800">Reset PIN — {selectedUser.name}</h2>
            </div>
            <div class="p-6">
                <div class="flex flex-col gap-1.5">
                    <label for="reset-pin-input" class="text-xs font-bold text-slate-600 uppercase tracking-wider">PIN Baru (min. 4 digit)</label>
                    <input
                        id="reset-pin-input"
                        type="password"
                        bind:value={resetPinValue}
                        placeholder="••••••"
                        maxlength="12"
                        class="w-full px-3 py-2.5 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-300 bg-slate-50"
                    />
                </div>
            </div>
            <div class="px-6 pb-6 flex gap-3 justify-end">
                <button type="button" onclick={() => showResetPinModal = false} id="btn-cancel-resetpin"
                    class="px-4 py-2.5 text-xs font-bold text-slate-600 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl cursor-pointer transition">
                    Batal
                </button>
                <button type="button" onclick={handleResetPin} disabled={isResettingPin} id="btn-confirm-resetpin"
                    class="px-5 py-2.5 text-xs font-bold text-white bg-amber-600 hover:bg-amber-700 rounded-xl cursor-pointer transition disabled:opacity-60 flex items-center gap-2">
                    {#if isResettingPin}<span class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>{/if}
                    Reset PIN
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- ── Modal: Assign Role ──────────────────────────────────────────────────── -->
{#if showAssignRoleModal && selectedUser}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-xs p-4">
        <div class="bg-white rounded-2xl shadow-2xl border border-slate-200 w-full max-w-sm overflow-hidden">
            <div class="px-6 py-5 border-b border-slate-100">
                <h2 class="text-sm font-black text-slate-800">Assign Role — {selectedUser.name}</h2>
            </div>
            <div class="p-6">
                <div class="flex flex-col gap-1.5">
                    <label for="assign-role-select" class="text-xs font-bold text-slate-600 uppercase tracking-wider">Pilih Role</label>
                    <select
                        id="assign-role-select"
                        bind:value={assignRoleId}
                        class="w-full px-3 py-2.5 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-300 bg-slate-50 cursor-pointer"
                    >
                        {#each roles as role}
                            <option value={role.id}>{role.name}</option>
                        {/each}
                    </select>
                </div>
            </div>
            <div class="px-6 pb-6 flex gap-3 justify-end">
                <button type="button" onclick={() => showAssignRoleModal = false} id="btn-cancel-assignrole"
                    class="px-4 py-2.5 text-xs font-bold text-slate-600 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl cursor-pointer transition">
                    Batal
                </button>
                <button type="button" onclick={handleAssignRole} disabled={isAssigningRole} id="btn-confirm-assignrole"
                    class="px-5 py-2.5 text-xs font-bold text-white bg-violet-600 hover:bg-violet-700 rounded-xl cursor-pointer transition disabled:opacity-60 flex items-center gap-2">
                    {#if isAssigningRole}<span class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>{/if}
                    Assign
                </button>
            </div>
        </div>
    </div>
{/if}

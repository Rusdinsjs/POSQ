<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  
  let { children } = $props();
  
  let isAuthenticated = $state(false);
  let currentUser = $state(null);
  
  $effect(() => {
    const token = localStorage.getItem('admin_token');
    if (token) {
      isAuthenticated = true;
    } else if ($page.url.pathname !== '/login') {
      goto('/login');
    }
  });
  
  async function logout() {
    localStorage.removeItem('admin_token');
    isAuthenticated = false;
    goto('/login');
  }
</script>

<div class="min-h-screen bg-gray-100">
  {#if isAuthenticated && $page.url.pathname !== '/login'}
    <div class="fixed inset-y-0 left-0 z-50 w-64 bg-white shadow-lg">
      <div class="flex h-16 items-center justify-center border-b border-gray-200">
        <h1 class="text-xl font-bold text-blue-600">POSQ Admin</h1>
      </div>
      
      <nav class="mt-6 px-4">
        <a href="/" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">📊</span>
          Dashboard
        </a>
        <a href="/merchants" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">🏪</span>
          Merchants
        </a>
        <a href="/devices" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">📱</span>
          Devices
        </a>
        <a href="/subscriptions" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">💳</span>
          Subscriptions
        </a>
        <a href="/updates" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">🔄</span>
          Updates
        </a>
        <a href="/backups" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">💾</span>
          Backups
        </a>
        <a href="/audit" class="flex items-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg">
          <span class="mr-3">📋</span>
          Audit Logs
        </a>
      </nav>
      
      <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-200">
        <button
          onclick={logout}
          class="w-full flex items-center justify-center px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg"
        >
          <span class="mr-3">🚪</span>
          Logout
        </button>
      </div>
    </div>
    
    <div class="ml-64 p-8">
      {@render children()}
    </div>
  {:else}
    {@render children()}
  {/if}
</div>

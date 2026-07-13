<script lang="ts">
  import { onMount } from 'svelte';
  
  let stats = $state({
    totalMerchants: 0,
    activeDevices: 0,
    activeSubscriptions: 0,
    recentBackups: 0,
  });
  
  let isLoading = $state(true);
  
  onMount(async () => {
    try {
      const token = localStorage.getItem('admin_token');
      const apiUrl = import.meta.env.PUBLIC_API_URL || 'http://localhost:3000';
      
      // Fetch stats
      const [merchantsRes, devicesRes] = await Promise.all([
        fetch(`${apiUrl}/api/v1/admin/merchants`, {
          headers: { 'Authorization': `Bearer ${token}` }
        }),
        // Add more API calls as needed
      ]);
      
      if (merchantsRes.ok) {
        const merchantsData = await merchantsRes.json();
        stats.totalMerchants = merchantsData.total || 0;
      }
      
      // Set mock data for demo
      stats.activeDevices = 12;
      stats.activeSubscriptions = 8;
      stats.recentBackups = 5;
      
    } catch (e) {
      console.error('Failed to fetch stats:', e);
    } finally {
      isLoading = false;
    }
  });
</script>

<div class="space-y-6">
  <h1 class="text-2xl font-bold text-gray-800">Dashboard</h1>
  
  {#if isLoading}
    <div class="flex items-center justify-center h-64">
      <div class="text-gray-500">Loading...</div>
    </div>
  {:else}
    <!-- Stats Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center">
          <div class="p-3 bg-blue-100 rounded-full">
            <span class="text-2xl">🏪</span>
          </div>
          <div class="ml-4">
            <p class="text-sm text-gray-600">Total Merchants</p>
            <p class="text-2xl font-bold text-gray-800">{stats.totalMerchants}</p>
          </div>
        </div>
      </div>
      
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center">
          <div class="p-3 bg-green-100 rounded-full">
            <span class="text-2xl">📱</span>
          </div>
          <div class="ml-4">
            <p class="text-sm text-gray-600">Active Devices</p>
            <p class="text-2xl font-bold text-gray-800">{stats.activeDevices}</p>
          </div>
        </div>
      </div>
      
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center">
          <div class="p-3 bg-purple-100 rounded-full">
            <span class="text-2xl">💳</span>
          </div>
          <div class="ml-4">
            <p class="text-sm text-gray-600">Active Subscriptions</p>
            <p class="text-2xl font-bold text-gray-800">{stats.activeSubscriptions}</p>
          </div>
        </div>
      </div>
      
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center">
          <div class="p-3 bg-yellow-100 rounded-full">
            <span class="text-2xl">💾</span>
          </div>
          <div class="ml-4">
            <p class="text-sm text-gray-600">Recent Backups</p>
            <p class="text-2xl font-bold text-gray-800">{stats.recentBackups}</p>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Quick Actions -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-800 mb-4">Quick Actions</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <a href="/merchants" class="block p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
          <div class="flex items-center">
            <span class="text-2xl mr-3">🏪</span>
            <div>
              <p class="font-medium text-gray-800">Manage Merchants</p>
              <p class="text-sm text-gray-600">View and manage merchant accounts</p>
            </div>
          </div>
        </a>
        
        <a href="/devices" class="block p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
          <div class="flex items-center">
            <span class="text-2xl mr-3">📱</span>
            <div>
              <p class="font-medium text-gray-800">Manage Devices</p>
              <p class="text-sm text-gray-600">View and revoke device access</p>
            </div>
          </div>
        </a>
        
        <a href="/subscriptions" class="block p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
          <div class="flex items-center">
            <span class="text-2xl mr-3">💳</span>
            <div>
              <p class="font-medium text-gray-800">Manage Subscriptions</p>
              <p class="text-sm text-gray-600">Extend and manage subscriptions</p>
            </div>
          </div>
        </a>
      </div>
    </div>
    
    <!-- Recent Activity -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-800 mb-4">Recent Activity</h2>
      <div class="space-y-4">
        <div class="flex items-center p-3 bg-gray-50 rounded-lg">
          <span class="text-xl mr-3">✅</span>
          <div>
            <p class="font-medium text-gray-800">Device activated</p>
            <p class="text-sm text-gray-600">Merchant #1234 activated a new device</p>
          </div>
          <span class="ml-auto text-sm text-gray-500">2 hours ago</span>
        </div>
        
        <div class="flex items-center p-3 bg-gray-50 rounded-lg">
          <span class="text-xl mr-3">💳</span>
          <div>
            <p class="font-medium text-gray-800">Subscription renewed</p>
            <p class="text-sm text-gray-600">Merchant #5678 renewed their subscription</p>
          </div>
          <span class="ml-auto text-sm text-gray-500">5 hours ago</span>
        </div>
        
        <div class="flex items-center p-3 bg-gray-50 rounded-lg">
          <span class="text-xl mr-3">💾</span>
          <div>
            <p class="font-medium text-gray-800">Backup uploaded</p>
            <p class="text-sm text-gray-600">Merchant #9012 uploaded a backup</p>
          </div>
          <span class="ml-auto text-sm text-gray-500">1 day ago</span>
        </div>
      </div>
    </div>
  {/if}
</div>

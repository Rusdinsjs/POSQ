<script lang="ts">
  import { onMount } from 'svelte';
  
  let merchants = $state([]);
  let isLoading = $state(true);
  let error = $state('');
  
  onMount(async () => {
    try {
      const token = localStorage.getItem('admin_token');
      const apiUrl = import.meta.env.PUBLIC_API_URL || 'http://localhost:3000';
      
      const response = await fetch(`${apiUrl}/api/v1/admin/merchants`, {
        headers: { 'Authorization': `Bearer ${token}` }
      });
      
      if (!response.ok) {
        throw new Error('Failed to fetch merchants');
      }
      
      const data = await response.json();
      merchants = data.merchants || [];
      
    } catch (e: any) {
      error = e.message;
    } finally {
      isLoading = false;
    }
  });
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-bold text-gray-800">Merchants</h1>
    <a
      href="/merchants/new"
      class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg"
    >
      Add Merchant
    </a>
  </div>
  
  {#if isLoading}
    <div class="flex items-center justify-center h-64">
      <div class="text-gray-500">Loading merchants...</div>
    </div>
  {:else if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {:else if merchants.length === 0}
    <div class="bg-white rounded-lg shadow p-8 text-center">
      <p class="text-gray-500">No merchants found</p>
    </div>
  {:else}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Name
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Slug
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Email
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Created
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each merchants as merchant}
            <tr>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm font-medium text-gray-900">{merchant.name}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm text-gray-500">{merchant.slug}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm text-gray-500">{merchant.email || '-'}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span
                  class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {merchant.active ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}"
                >
                  {merchant.active ? 'Active' : 'Inactive'}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {new Date(merchant.created_at).toLocaleDateString()}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <a href="/merchants/{merchant.id}" class="text-blue-600 hover:text-blue-900">
                  View
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

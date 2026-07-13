<script lang="ts">
  import { goto } from '$app/navigation';
  
  let email = $state('');
  let password = $state('');
  let isLoading = $state(false);
  let error = $state('');
  
  async function handleLogin() {
    isLoading = true;
    error = '';
    
    try {
      const response = await fetch(`${import.meta.env.PUBLIC_API_URL || 'http://localhost:3000'}/api/v1/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      });
      
      const data = await response.json();
      
      if (!response.ok) {
        throw new Error(data.error?.message || 'Login failed');
      }
      
      // Store token
      localStorage.setItem('admin_token', data.access_token);
      
      // Redirect to dashboard
      goto('/');
    } catch (e: any) {
      error = e.message;
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
  <div class="max-w-md w-full bg-white rounded-lg shadow-md p-8">
    <div class="text-center mb-8">
      <h1 class="text-2xl font-bold text-gray-800">POSQ Admin Dashboard</h1>
      <p class="text-gray-600 mt-2">Sign in to manage your POS system</p>
    </div>
    
    <form onsubmit={handleLogin}>
      {#if error}
        <div class="mb-4 p-4 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      {/if}
      
      <div class="mb-4">
        <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
          Email
        </label>
        <input
          type="email"
          id="email"
          bind:value={email}
          required
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="admin@posq.id"
        />
      </div>
      
      <div class="mb-6">
        <label for="password" class="block text-sm font-medium text-gray-700 mb-2">
          Password
        </label>
        <input
          type="password"
          id="password"
          bind:value={password}
          required
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="••••••••"
        />
      </div>
      
      <button
        type="submit"
        disabled={isLoading}
        class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg disabled:opacity-50"
      >
        {isLoading ? 'Signing in...' : 'Sign in'}
      </button>
    </form>
  </div>
</div>

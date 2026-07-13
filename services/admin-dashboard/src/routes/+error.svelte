<script lang="ts">
  import { page } from '$app/stores';
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
  <div class="text-center">
    <h1 class="text-6xl font-bold text-gray-800">{page.status}</h1>
    <p class="text-xl text-gray-600 mt-4">{page.error?.message || 'An error occurred'}</p>
    <a href="/" class="inline-block mt-6 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
      Go back home
    </a>
  </div>
</div>

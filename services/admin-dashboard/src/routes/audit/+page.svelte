<script lang="ts">
  const { title, description } = $props();
</script>

<div class="space-y-6">
  <h1 class="text-2xl font-bold text-gray-800">{title}</h1>
  <div class="bg-white rounded-lg shadow p-8 text-center">
    <p class="text-gray-500 text-lg mb-4">{description}</p>
    <p class="text-gray-400 text-sm">Coming soon</p>
  </div>
</div>

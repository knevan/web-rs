<script lang="ts">
    let {children} = $props();
</script>

<div class="flex min-h-screen">
    <main class="max-w-7xl p-4 flex-1 mx-auto">
        {@render children()}
    </main>
</div>
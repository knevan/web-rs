<script lang="ts">
    let {children} = $props();
</script>

<main class=" flex-1 lg:max-w-5xl mx-auto p-4 flex h-screen w-full items-center justify-center px-4">
    {@render children()}
</main>


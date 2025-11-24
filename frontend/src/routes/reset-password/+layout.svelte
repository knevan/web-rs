<script lang="ts">
    let {children} = $props();
</script>


<main class="w-full flex-1 lg:max-w-5xl mx-auto p-4 flex items-center justify-center px-4">
    {@render children()}
</main>

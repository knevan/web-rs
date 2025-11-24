<script lang="ts">
    let {children} = $props();
</script>

<main class="max-w-4xl lg:max-w-7xl p-4 flex-1 mx-auto w-full">
    {@render children()}
</main>
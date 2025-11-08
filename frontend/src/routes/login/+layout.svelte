<script lang="ts">
    let {children} = $props();
</script>

<div class="flex flex-col w-full flex-1 lg:max-w-5xl mx-auto p-4 items-center justify-center px-4">
    {@render children()}
</div>
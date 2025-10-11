<script lang="ts">
    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">

    <main class="w-full flex-1 mx-auto p-4">
        {@render children()}
    </main>

</div>
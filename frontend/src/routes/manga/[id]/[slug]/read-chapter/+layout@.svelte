<script lang="ts">
    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">

    <main class="flex-1 mx-auto">
        {@render children()}
    </main>

</div>
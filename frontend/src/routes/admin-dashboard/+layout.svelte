<link rel="stylesheet" href="../../app.css">
<script>
    let {children} = $props();
</script>

<div class="admin-panel">

    <main class="flex-grow container mx-auto px-1">
        {@render children()}
    </main>

</div>

<style>
    .admin-panel {
        background-color: var(--secondary);
        color: var(--primary-foreground);
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        width: 100%;
    }
</style>

<script>
    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">

    <main class="flex-grow container max-w-7xl mx-auto px-1">
        {@render children()}
    </main>

</div>

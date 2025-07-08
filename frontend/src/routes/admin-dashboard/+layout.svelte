<script>
    let {children} = $props();
</script>

<div class="flex flex-col min-h-screen bg-gray-50">

    <main class="flex-grow container mx-auto px-4">
        {@render children()}
    </main>

</div>

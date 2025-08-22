<!--<link rel="stylesheet" href="../../app.css">-->
<script>
    import Header from "$lib/components/Header.svelte";

    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">
    <Header/>

    <main class="flex-grow container max-w-7xl mx-auto px-1">
        {@render children()}
    </main>

</div>

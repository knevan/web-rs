<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";

    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">
    <Header/>

    <main class="flex-1 mx-auto">
        {@render children()}
    </main>

    <Footer/>
</div>
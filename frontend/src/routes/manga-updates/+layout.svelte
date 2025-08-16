<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";
    import type {Snippet} from "svelte";


    let {children}: { children: Snippet } = $props();
</script>

<div class="flex min-h-screen flex-col">
    <Header/>

    <main class="w-full flex-1 lg:max-w-6xl mx-auto p-4">
        {@render children()}
    </main>

    <Footer/>
</div>
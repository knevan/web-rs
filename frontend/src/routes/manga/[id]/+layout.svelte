<script lang="ts">
    import Footer from "$lib/components/Footer.svelte";

    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">
    <main class="max-w-[1440px] flex-1 lg:max-w-4xl mx-auto md:p-4">
        {@render children()}
    </main>

    <Footer/>
</div>
<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";

    let {children} = $props();
</script>

<div class="flex min-h-screen flex-col">
    
    <main class="w-full flex-1 mx-auto p-4">
        {@render children()}
    </main>

</div>
<script lang="ts">
    import Header from '$lib/components/Header.svelte';
    import '../app.css'
    import {auth} from "$lib/store/auth.js";
    import {Toaster} from "$lib/components/ui/sonner/index.js";
    import {ModeWatcher} from "mode-watcher";
    import Footer from "$lib/components/Footer.svelte";
    import {search} from '$lib/store/searchStore.svelte';
    import SearchSeries from '$lib/components/SearchSeries.svelte';
    import {NuqsAdapter} from "nuqs-svelte/adapters/svelte-kit";

    let {data, children} = $props();

    // It will sync the `api` store whenever the `data` from `load` function change
    $effect(() => {
        if (data.user) {
            auth.set({
                isAuthenticated: true,
                user: data.user,
                error: null,
            });
        } else {
            // If no user data, set the store to its initial unauthenticated state.
            auth.set({
                isAuthenticated: false,
                user: null,
                error: null,
            })
        }
    })
</script>

<ModeWatcher/>

<Toaster
        richColors
        closeButton
        class="![--width:clamp(280px,340px,100%)] xl:[--width:clamp(280px,660px,100%)]"
        toastOptions={{
            classes: {
                toast: 'flex items-center w-full px-5 py-3 rounded-lg shadow-lg bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-500 text-zinc-800 dark:text-zinc-200',
                title: 'font-medium text-sm'
            }
        }}
/>

<Header/>
{#if search.isOpen}
    <div class="w-full bg-[--background] px-2">
        <SearchSeries/>
    </div>
{/if}
<NuqsAdapter>
    {@render children()}
</NuqsAdapter>
<Footer/>
<script lang="ts">
    import Header from '$lib/components/Header.svelte';
    import '../app.css'
    import {auth} from "$lib/store/auth.js";
    import {Toaster} from "$lib/components/ui/sonner/index.js";
    import {ModeWatcher} from "mode-watcher";
    import Footer from "$lib/components/Footer.svelte";
	import { beforeNavigate } from '$app/navigation';
	import { search } from '$lib/store/searchStore.svelte';
	import SearchSeries from '$lib/components/SearchSeries.svelte';

    let {data, children} = $props();
    let scrollYToRestore: number | undefined;

    beforeNavigate(({ from, to }) => {
        console.log(`DEBUG: SvelteKit is navigating from '${from?.url.pathname}' to '${to?.url.pathname}'.`);
        console.log('DEBUG: Calling `search.close()` due to navigation.');
        search.close();
    });

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

    /* Save the scroll position before navigating
    beforeNavigate((navigation) => {
        if (navigation.from?.url.pathname === navigation.to?.url.pathname) {
            return;
        }
        savePosition(navigation.from?.url.pathname ?? '/', window.scrollY)
    })

    afterNavigate(async (navigation) => {
        if (navigation.type === 'popstate') {
            scrollYToRestore = getPosition(navigation.to?.url.pathname ?? '/');
        } else {
            window.scrollTo({top: 0});
            scrollYToRestore = undefined;
        }

        if (scrollYToRestore !== undefined) {
            await tick();
            requestAnimationFrame(() => {
                window.scrollTo({top: scrollYToRestore});
            });
        }
    })*/

    $effect(() => {
        console.log(
            'DEBUG: `+layout.svelte` detected a change in `search.isOpen`. New value:',
            search.isOpen
        );
    });
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
    <div class="w-full bg-[--background] px-6">
        <SearchSeries />
    </div>
{/if}
{@render children()}
<Footer/>
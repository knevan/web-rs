<script>
    import {goto} from "$app/navigation";
    import {auth} from "$lib/store/auth.js";
    import {tick} from "svelte";

    let {children} = $props();

    $effect(() => {
        // Checking the session.
        if ($auth.isVerifying) {
            return;
        }

        // If session verified (isVerifying = false), user not logged in, redirect to login page
        if ($auth.isAuthenticated === false) {
            tick().then(() => {
                goto(`/login?redirectTo=/admin-dashboard`)
            });
            // If session verified, user logged in, but role not admin
        } else if ($auth.isAuthenticated === true && $auth.user?.role !== 'admin') {
            tick().then(() => {
                goto(`/?error=unauthorized`)
            });
        }
    });

    // Check if user is admin render admin-dashboard user on header
    let isAdmin = $derived($auth.isAuthenticated && $auth.user?.role === 'admin');
</script>

{#if isAdmin}
    <main class="flex-grow container max-w-7xl mx-auto px-1">
        {@render children()}
    </main>
{:else}
    <main class="flex-grow container max-w-7xl mx-auto px-1">
        <div class="flex h-[calc(100vh-10rem)] items-center justify-center">
            <p class="text-lg font-medium">Verify admin access...</p>
        </div>
    </main>
{/if}


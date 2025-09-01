<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";
    import {auth, login} from "$lib/store/auth";
    import {page} from "$app/state";
    import {Eye} from "@lucide/svelte";
    import {EyeOff} from "@lucide/svelte";
    import {onMount} from "svelte";

    // props are now passed via a prop object in Svelte 5
    let {id = 'login-form'} = $props();

    let email = $state('');
    let password = $state('');
    let isLoading = $state(false);
    let showPassword = $state(false);

    // Subscribe to the api store to get the latest state, including errors
    const authState = $derived($auth);

    const redirectTo = $derived(page.url.searchParams.get('redirectTo'));

    async function handleLogin() {
        isLoading = true;
        await login(email, password, redirectTo);
        isLoading = false;
    }

    onMount(() => {
        if (authState.error) {
            auth.update((store) => ({...store, error: null}));
        }
    });

    // const id = $props.id();
</script>

<Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header class="text-center">
        <Card.Title class="text-2xl md:text-4xl">Login</Card.Title>
        <Card.Description class="text-xs md:text-base">Enter your email or username below to login to your account
        </Card.Description>
    </Card.Header>
    <Card.Content>
        <form onsubmit={handleLogin}>
            <div class="grid gap-4">
                {#if authState.error}
                    <div class="text-red-500 text-xs">{authState.error}</div>
                {/if}
                <div class="grid gap-2">
                    <Label for="email-{id}" class="text-md">Credential</Label>
                    <Input bind:value={email} id="email-{id}" type="text"
                           class="text-xs md:text-lg"
                           placeholder="ex@gmail.com or username"
                           required/>
                </div>
                <div class="grid gap-2">
                    <div class="flex items-center">
                        <Label for="password-{id}" class="text-base md:text-lg">Password</Label>
                        <a href="/reset-password"
                           class="ml-auto inline-block text-xs md:text-base underline">
                            Forgot your password?
                        </a>
                    </div>
                    <div class="relative">
                        <Input bind:value={password} id="password-{id}"
                               type={showPassword ? 'text' : 'password'}
                               required/>
                        <!-- Toggle button for password visibility -->
                        <button
                                type="button"
                                onclick={() => (showPassword = !showPassword)}
                                class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-300 hover:text-gray-400"
                                aria-label="Toggle password visibility"
                        >
                            {#if showPassword}
                                <!-- Eye Icon -->
                                <Eye/>
                            {:else}
                                <!-- Eye Off Icon -->
                                <EyeOff/>
                            {/if}
                        </button>
                    </div>
                </div>
                <Button type="submit" class="w-full text-base md:text-lg" disabled={isLoading}>
                    {isLoading ? 'Logging in...' : 'Login'}
                </Button>
            </div>
        </form>
        <div class="mt-4 text-center text-base md:text-lg">
            Don't have an account?
            <a href="/signup" class="underline text-base md:text-lg"> Sign up </a>
        </div>
    </Card.Content>
</Card.Root>

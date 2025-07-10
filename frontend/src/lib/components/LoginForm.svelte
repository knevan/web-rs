<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";
    import {auth, login} from "$lib/store/auth";

    // props are now passed via a prop object in Svelte 5
    let {id = 'login-form'} = $props();

    let email = $state('');
    let password = $state('');
    let isLoading = $state(false);
    let showPassword = $state(false);

    // Subscribe to the auth store to get the latest state, including errors
    const authState = $derived($auth);

    async function handleLogin() {
        isLoading = true;
        // The login function from the store will handle all logic,
        // including updating the store with errors or success.
        await login(email, password);
        isLoading = false;
    }


    // const id = $props.id();
</script>

<Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header class="text-center">
        <Card.Title class="text-3xl">Login</Card.Title>
        <Card.Description>Enter your email or username below to login to your account
        </Card.Description>
        <Card.Description class="text-xs pt-2">
            Admin: admin@admin.com / admin123 <br/>
            User: user@admin.com / user123
        </Card.Description>
    </Card.Header>
    <Card.Content>
        <form onsubmit={handleLogin}>
            <div class="grid gap-4">
                {#if authState.error}
                    <div class="text-red-500 text-xs">{authState.error}</div>
                {/if}
                <div class="grid gap-2">
                    <Label for="email-{id}">Credential</Label>
                    <Input bind:value={email} id="email-{id}" type="text"
                           placeholder="m@example.com or username"
                           required/>
                </div>
                <div class="grid gap-2">
                    <div class="flex items-center">
                        <Label for="password-{id}">Password</Label>
                        <a href="/reset-password"
                           class="ml-auto inline-block text-sm underline">
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
                                <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                >
                                    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
                                    <circle
                                            cx="12"
                                            cy="12"
                                            r="3"
                                    />
                                </svg>
                            {:else}
                                <!-- Eye Off Icon -->
                                <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                >
                                    <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"/>
                                    <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/>
                                    <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/>
                                    <line x1="2" x2="22" y1="2" y2="22"/>
                                </svg>
                            {/if}
                        </button>
                    </div>
                </div>
                <Button type="submit" class="w-full" disabled={isLoading}>
                    {isLoading ? 'Logging in...' : 'login'}
                </Button>
            </div>
        </form>
        <div class="mt-4 text-center text-sm">
            Don't have an account?
            <a href="/signup" class="underline"> Sign up </a>
        </div>
    </Card.Content>
</Card.Root>

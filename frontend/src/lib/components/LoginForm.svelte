<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";
    import {auth, login} from "$lib/store/auth";
    import {goto} from "$app/navigation";

    // props are now passed via a prop object in Svelte 5
    let {id = 'login-form'} = $props();

    let email = $state('');
    let password = $state('');
    let errorMessage = $state('');
    let isLoading = $state(false);

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
        <Card.Description>Enter your email or username below to login to your account</Card.Description>
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
                    <Label for="email-{id}">Email</Label>
                    <Input bind:value={email} id="email-{id}" type="email" placeholder="m@example.com or username"
                           required/>
                </div>
                <div class="grid gap-2">
                    <div class="flex items-center">
                        <Label for="password-{id}">Password</Label>
                        <a href="/reset-password" class="ml-auto inline-block text-sm underline">
                            Forgot your password?
                        </a>
                    </div>
                    <Input bind:value={password} id="password-{id}" type="password" required/>
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

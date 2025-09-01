<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";
    import {checkUsernameAvailability, register} from "$lib/store/auth";
    import {goto} from "$app/navigation";
    import {Eye} from "@lucide/svelte";
    import {EyeOff} from "@lucide/svelte";
    import {LoaderCircle} from "@lucide/svelte";

    const id = $props.id();

    // State for form input and feedback
    let username = $state('');
    let email = $state('');
    let password = $state('');
    let confirmPassword = $state('');
    let error = $state<string | null>(null);
    let successMessage = $state<string | null>(null);
    let isLoading = $state(false);
    let showPassword = $state(false);
    let showConfirmPassword = $state(false);

    // Username validation state
    let isCheckingUsername = $state(false);
    let usernameAvailable = $state<boolean | null>(null);
    let usernameCheckMessage = $state('');
    let debounceTimer = $state<any>(null); // To hold set timeout ID

    // Realtime username validation state
    type UsernameRequirementKey = 'length';
    const usernameRequirementsList = [
        {key: 'length' as UsernameRequirementKey, text: 'Minimum 4 characters long'},
    ];
    let usernameRequirement = $derived({
        length: username.trim().length >= 4,
    });
    let allUsernameRequirementMet = $derived(Object.values(usernameRequirement).every(Boolean));

    // Realtime password validation state
    type PasswordRequirementKey = 'length' | 'uppercase';
    let passwordRequirement = $derived({
        length: password.length >= 8,
        uppercase: /[A-Z]/.test(password),
    });

    // [NOTE]: Maybe add password strength indicator for the future

    // Check if all requirements are met
    let allRequirementMet = $derived(Object.values(passwordRequirement).every(Boolean));

    // Check confirm password match
    let passwordMatch = $derived(password === confirmPassword);

    // List of password for easy rendering and validation
    const requirementList = [
        {key: 'length' as PasswordRequirementKey, text: 'Minimum 8 characters long'},
        {
            key: 'uppercase' as PasswordRequirementKey,
            text: 'Contains at least one uppercase letter'
        },
    ]

    // Debounce function to check username availability after user stops typing
    function handleUsernameInput() {
        clearTimeout(debounceTimer);
        usernameAvailable = null;
        usernameCheckMessage = '';
        isCheckingUsername = false;

        // Validate username length 4-20 characters
        if (!allUsernameRequirementMet) {
            return;
        }

        isCheckingUsername = true;
        usernameCheckMessage = 'Checking availability...';

        // Set a 700ms timer to check after user stops typing
        debounceTimer = setTimeout(async () => {
            const result = await checkUsernameAvailability(username);
            usernameAvailable = result.available;
            usernameCheckMessage = result.message;
            isCheckingUsername = false;
        }, 1200);
    }

    // Form submission handler
    async function handleSubmit() {
        // Reset previous messages
        error = null;
        successMessage = null;
        isLoading = true;

        // Client-side validation
        if (!username || !email || !password || !confirmPassword) {
            error = 'All fields are required.';
            isLoading = false;
            return;
        }
        if (!allRequirementMet) {
            error = 'Password does not meet all requirements.';
            isLoading = false;
            return;
        }
        if (password !== confirmPassword) {
            error = 'Passwords do not match.';
            isLoading = false;
            return;
        }

        // Call register function
        const result = await register(username, email, password);

        // Handle the response
        if (result.success) {
            successMessage = 'Registration successfull! Redirecting to login page';
            // Redirect to login page
            setTimeout(() => {
                goto('/login');
            }, 1500);
        } else {
            // Display error message from backend
            error = result.error || 'An error occurred while registering.';
        }
        isLoading = false;
    }
</script>

<Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header class="text-center">
        <Card.Title class="text-3xl">Register</Card.Title>
        <Card.Description>Register New Account</Card.Description>
    </Card.Header>
    <Card.Content>
        <!-- User form element to handle submission -->
        <form onsubmit={handleSubmit} class="flex flex-col">
            <div class="grid gap-2">
                <Label for="username-{id}">Username</Label>
                <div class="relative">
                    <!-- Bind input value to the state variable -->
                    <Input bind:value={username} oninput={handleUsernameInput}
                           id="username-{id}" type="text"
                           placeholder="username" required/>
                    <!-- Username requirement and availability check -->
                    <div class="mt-1 grid gap-2 text-sm">
                        <!-- Length requirement -->
                        {#each usernameRequirementsList as req}
                            <p
                                    class:text-green-500={usernameRequirement[req.key]}
                                    class:text-red-500={!usernameRequirement[req.key]}
                                    class="transition-colors"
                            >
                                {usernameRequirement[req.key] ? '✅' : '❌'}
                                {req.text}
                            </p>
                        {/each}
                    </div>
                    <!-- Dynamic availability message -->
                    <div class="mt-0 text-sm transition-opacity grid duration-300">
                        {#if usernameCheckMessage}
                            <p
                                    class:text-green-500={usernameAvailable}
                                    class:text-red-500={!usernameAvailable}
                            >
                                {#if isCheckingUsername}
                                    {usernameCheckMessage}
                                {:else}
                                    {usernameAvailable ? '✅' : '❌'} {usernameCheckMessage}
                                {/if}
                            </p>
                        {/if}
                    </div>
                </div>
            </div>
            <div class="grid gap-2 mb-2">
                <Label for="email-{id}">Email</Label>
                <Input bind:value={email} id="email-{id}" type="email"
                       placeholder="example@gmail.com" required/>
            </div>
            <div class="grid mb-2">
                <Label for="password-{id}" class="pb-2">Password</Label>
                <div class="relative">
                    <Input bind:value={password} id="password-{id}"
                           type={showPassword ? 'text' : 'password'}
                           required
                           class="pr-10"/>
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
                <!-- Password requirement UI -->
                <div class="mt-1 grid gap-1 text-sm">
                    {#each requirementList as req}
                        <p
                                class:text-green-500={passwordRequirement[req.key]}
                                class:text-red-500={!passwordRequirement[req.key]}
                                class="transition-colors"
                        >
                            {passwordRequirement[req.key] ? '✅' : '❌'}
                            {req.text}
                        </p>
                    {/each}
                </div>
            </div>
            <div class="grid gap-2 -mb-1">
                <Label for="confirm-password-{id}">Confirm Password</Label>
                <div class="relative">
                    <Input bind:value={confirmPassword} id="confirm-password-{id}"
                           type={showConfirmPassword ? 'text' : 'password'} required
                           class="pr-10"/>
                    <!-- Toggle button for password visibility -->
                    <button
                            type="button"
                            onclick={() => (showConfirmPassword = !showConfirmPassword)}
                            class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-300 hover:text-gray-400"
                            aria-label="Toggle password visibility"
                    >
                        {#if showConfirmPassword}
                            <!-- Eye Icon -->
                            <Eye/>
                        {:else}
                            <!-- Eye Off Icon -->
                            <EyeOff/>
                        {/if}
                    </button>
                </div>
                <div class="-mt-1 text-sm transition-opacity grid duration-300">
                    {#if confirmPassword.length > 0}
                        <p
                                class:text-green-500={passwordMatch}
                                class:text-red-500={!passwordMatch}
                                class="transition-colors"
                        >
                            {passwordMatch ? '✅' : '❌'}
                            {passwordMatch ? 'Passwords match' : 'Passwords do not match'}
                        </p>
                    {/if}
                </div>
            </div>
            <!-- Disable button while loading -->
            <Button type="submit" class="w-full mt-2" disabled={isLoading}>
                {#if isLoading}
                    <LoaderCircle class="animate-spin"/>
                    Registering...
                {:else}
                    Register
                {/if}
            </Button>

            <!-- Display error or success message -->
            {#if error}
                <p class="text-sm text-red-500">{error}</p>
            {/if}
            {#if successMessage}
                <p class="text-sm text-green-500">{successMessage}</p>
            {/if}
        </form>
        <div class="mt-4 text-center text-sm">
            Already have an account?
            <a href="/login" class="underline"> Login </a>
        </div>
    </Card.Content>
</Card.Root>
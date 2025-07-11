<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";
    import {checkUsernameAvailability, register} from "$lib/store/auth";
    import {goto} from "$app/navigation";

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
            <div class="grid gap-2 mb-1">
                <Label for="username-{id}">Username</Label>
                <!-- Bind input value to the state variable -->
                <Input bind:value={username} oninput={handleUsernameInput}
                       id="username-{id}" type="text"
                       placeholder="username" required/>
                <!-- Username requirement and availability check -->
                <div class="mt-1 grid gap-1 text-sm">
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
            <div class="grid gap-2 mb-2">
                <Label for="email-{id}">Email</Label>
                <Input bind:value={email} id="email-{id}" type="email"
                       placeholder="example@gmail.com" required/>
            </div>
            <div class="grid gap-2 mb-2">
                <Label for="password-{id}">Password</Label>
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
                <!-- Password requirement UI -->
                <div class="mt-2 grid gap-1 text-sm">
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
            <div class="grid gap-2 mb-2">
                <Label for="confirm-password-{id}">Confirm Password</Label>
                <div class="relative">
                    <Input bind:value={confirmPassword} id="confirm-password-{id}"
                           type={showPassword ? 'text' : 'password'} required
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
                <div class="mt-2 text-sm">
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
                    <svg class="mr-2 h-4 w-4 animate-spin"
                         xmlns="http://www.w3.org/2000/svg"
                         viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10"
                                stroke="currentColor"
                                stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor"
                              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
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
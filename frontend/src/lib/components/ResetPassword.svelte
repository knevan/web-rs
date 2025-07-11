<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";

    const id = $props.id();
    let showPassword = $state(false);
</script>

<Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header class="text-center">
        <Card.Title class="text-3xl">Reset Password</Card.Title>
        <Card.Description class="text-center text-sm">Set Your New Password
        </Card.Description>
    </Card.Header>
    <Card.Content>
        <div class="grid gap-4">
            <div class="grid gap-2">
                <Label for="email-{id}">New Password</Label>
                <div class="relative">
                    <Input id="email-{id}" type={showPassword ? 'text' : 'password'}
                           required/>
                    <!-- Toggle button for password visibility -->
                    <button
                            type="button"
                            onclick={() => (showPassword = !showPassword)}
                            class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-300 hover:text-gray-400"
                            aria-label="Toggle password visibility"
                    >
                        {#if showPassword}
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
                                <path
                                        d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"
                                />
                                <path
                                        d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"
                                />
                                <line x1="2" x2="22" y1="2" y2="22"/>
                            </svg>
                        {:else}
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
                        {/if}
                    </button>
                </div>
            </div>
            <div class="grid gap-2">
                <Label for="confirm-password-{id}">Confirm New Password</Label>
                <div class="relative">
                    <Input id="confirm-password-{id}"
                           type={showPassword ? 'text' : 'password'} required/>
                    <!-- Toggle button for password visibility -->
                    <button
                            type="button"
                            onclick={() => (showPassword = !showPassword)}
                            class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-300 hover:text-gray-400"
                            aria-label="Toggle password visibility"
                    >
                        {#if showPassword}
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
                                <path
                                        d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"
                                />
                                <path
                                        d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"
                                />
                                <line x1="2" x2="22" y1="2" y2="22"/>
                            </svg>
                        {:else}
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
                        {/if}
                    </button>
                </div>
            </div>
            <Button type="submit" class="w-full">Reset Password</Button>
        </div>
        <div class="mt-4 text-center text-sm">
            Back to Login
            <a href="/login" class="underline"> Login </a>
        </div>
    </Card.Content>
</Card.Root>
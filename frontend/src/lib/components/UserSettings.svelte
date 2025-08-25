<script lang="ts">
    import {apiFetch, auth} from "$lib/store/auth";
    import {page} from "$app/state";
    import {goto} from "$app/navigation";
    import {toast} from "svelte-sonner";
    import {Input} from "$lib/components/ui/input";
    import {Label} from "$lib/components/ui/label";
    import {Button} from "$lib/components/ui/button";
    import {UserRound} from "@lucide/svelte";

    let userProfile = $state({
        email: '',
        displayName: '',
        avatarUrl: '',
    });

    let avatarFile = $state<File | null>();
    let avatarPreviewUrl = $state<string | null>();

    let passwordData = $state({
        newPassword: '',
        newPasswordConfirm: ''
    });

    let isLoading = $state(true);
    let error = $state<string | null>(null);

    let isAvatarLoading = $state(false);
    let isProfileLoading = $state(false);
    let isPasswordLoading = $state(false);

    $effect(() => {
        if (!$auth.isAuthenticated) {
            const redirectTo = page.url.pathname;
            goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
            return;
        }

        async function fetchUserProfile() {
            isLoading = true;
            error = null;
            try {
                const response = await apiFetch('/api/user/profile');
                if (!response.ok) {
                    throw new Error(`Failed to fetch user profile: ${response.statusText}`);
                }
                const data = await response.json();
                userProfile.email = data.email ?? '';
                userProfile.displayName = data.displayName ?? '';
                userProfile.avatarUrl = data.avatarUrl ?? '';
            } catch (err) {
                const errorMessage = err instanceof Error ? err.message : 'Unknown user profile error';
                error = errorMessage;
                toast.error(errorMessage, {
                    position: "top-center",
                    richColors: true,
                    closeButton: false,
                    duration: 2000,
                });
            } finally {
                isLoading = false;
            }
        }

        fetchUserProfile();
    });

    $effect(() => {
        if (avatarPreviewUrl) {
            URL.revokeObjectURL(avatarPreviewUrl);
        }
        if (avatarFile) {
            avatarPreviewUrl = URL.createObjectURL(avatarFile);
        } else {
            avatarPreviewUrl = null;
        }
    });

    function handleFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            avatarFile = target.files[0];
        }
    }

    async function handleAvatarUpdate() {
        if (!avatarFile) {
            toast.warning('Please select an image first.', {
                position: "top-center",
                richColors: true,
                closeButton: false,
                duration: 2000,
            });
            return;
        }

        isAvatarLoading = true;

        const avatarPromise = async () => {
            const formData = new FormData();
            formData.append('avatar', avatarFile!);

            const response = await fetch('/api/user/avatar', {
                method: 'POST',
                body: formData
            });

            if (!response.ok) {
                throw new Error(`Failed to upload avatar profile: ${response.statusText}`);
            }

            const result = await response.json();
            userProfile.avatarUrl = result.url;
            avatarFile = null;
        };
        toast.promise(avatarPromise(), {
            position: "top-center",
            richColors: true,
            closeButton: false,
            duration: 2000,
            loading: 'Uploading avatar...',
            success: 'Avatar updated successfully.',
            error: (err) => err instanceof Error ? err.message : 'Update avatar profile failed!',
            finally: () => {
                isAvatarLoading = false;
            }
        })
    }

    async function handleProfileUpdate() {
        isProfileLoading = true;
        const profilePromise = async () => {
            const response = await fetch('/api/user/profile', {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    displayName: userProfile.displayName,
                    email: userProfile.email,
                })
            });
            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to update profile.');
            }
        };

        toast.promise(profilePromise(), {
            position: "top-center",
            richColors: true,
            closeButton: false,
            duration: 2000,
            loading: 'Updating profile...',
            success: 'Profile updated successfully!',
            error: (err) => err instanceof Error ? err.message : 'Could not update profile.',
            finally: () => {
                isProfileLoading = false;
            }
        });
    }

    async function handlePasswordUpdate() {
        if (passwordData.newPassword.length < 8) {
            toast.error('Password must be at least 8 characters long.');
            return;
        }
        if (passwordData.newPassword !== passwordData.newPasswordConfirm) {
            toast.error('Passwords do not match.');
            return;
        }

        isPasswordLoading = true;

        const passwordPromise = async () => {
            const response = await fetch('/api/user/password', {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    newPassword: passwordData.newPassword
                })
            });
            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to update password.');
            }
        };
        toast.promise(passwordPromise(), {
            position: "top-center",
            richColors: true,
            closeButton: false,
            duration: 2000,
            success: () => {
                passwordData.newPassword = '';
                passwordData.newPasswordConfirm = '';
                return 'Password updated successfully!';
            },
            error: (err) => err instanceof Error ? err.message : 'Could not change password.',
            finally: () => {
                isPasswordLoading = false;
            }
        });
    }
</script>

<div class="max-w-3xl mx-auto p-4 md:p-6 text-gray-200">
    <div class="mb-8">
        <h1 class="text-3xl font-bold text-center text-gray-800 dark:text-gray-200">Change Profile Settings</h1>
        <p class="text-gray-800 dark:text-gray-200 mt-4 text-center text-lg">
            Hello, <span class="font-semibold text-lg">{$auth.user?.identifier ?? 'User'}</span>.
        </p>
    </div>

    {#if isLoading}
        <div class="text-center">
            <p>Loading...</p>
        </div>
    {:else if error}
        <div class="bg-red-900/20 border border-red-500 rounded-lg p-6 text-center">
            <h3 class="text-xl text-red-300">Could not load your settings</h3>
            <p class="text-gray-400 mt-2">{error}</p>
        </div>
    {:else}
        <div class="space-y-10">
            <section>
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2 mb-4">
                    Change Avatar</h2>
                <p class="text-base mb-4 text-gray-800 dark:text-gray-200">
                    Use the form below to change your avatar. Using inappropriate images may result in an account ban.
                </p>
                <div class="flex flex-col gap-3">

                    {#if avatarPreviewUrl || userProfile.avatarUrl}
                        <img
                                src={avatarPreviewUrl ?? userProfile.avatarUrl}
                                alt="Avatar Preview"
                                class="w-24 h-24 rounded-full object-cover bg-gray-800 border-2 border-gray-600"
                        />
                    {:else}
                        <div class="w-24 h-24 rounded-full flex items-center justify-center bg-gray-100 dark:bg-gray-800 border-2 border-gray-300 dark:border-gray-600">
                            <UserRound class="w-20 h-20 text-gray-400 dark:text-gray-500"/>
                        </div>
                    {/if}

                    <div class="flex flex-col items-start gap-3">
                        <Input
                                type="file"
                                id="avatar-upload"
                                class="hidden"
                                accept="image/png, image/jpeg, image/gif"
                                onchange={handleFileSelect}
                                disabled={isAvatarLoading}
                        />
                        <Label
                                for="avatar-upload"
                                class="cursor-pointer bg-gray-700 text-white font-semibold py-1 px-3 rounded-md hover:bg-gray-600 transition-colors"
                        >
                            Choose File
                        </Label>
                        {#if avatarFile}
                            <div class="flex items-center gap-2">
                                <span class="ml-4 text-gray-400">{avatarFile.name}</span>
                                <Button
                                        onclick={handleAvatarUpdate}
                                        class="ml-4 bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg transition-colors"
                                        disabled={isAvatarLoading}>
                                    {isAvatarLoading ? 'Uploading...' : 'Upload'}
                                </Button>
                            </div>
                        {/if}
                    </div>
                </div>
            </section>
            <section>
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2 mb-4">
                    Account Information</h2>
                <form onsubmit={handleProfileUpdate} class="space-y-6">
                    <div>
                        <Label for="displayName"
                               class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Change Comment
                            Name</Label>
                        <Input
                                type="text"
                                id="displayName"
                                bind:value={userProfile.displayName}
                                class="w-full bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500"
                                placeholder="Enter your display name"
                                disabled={isProfileLoading}
                        />
                    </div>
                    <div>
                        <Label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Change
                            Email</Label>
                        <Input
                                type="email"
                                id="email"
                                bind:value={userProfile.email}
                                class="w-full bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500"
                                placeholder="your.email@example.com"
                                disabled={isProfileLoading}
                        />
                    </div>
                    <div>
                        <Button
                                type="submit"
                                class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg transition-colors disabled:bg-blue-800 disabled:cursor-not-allowed"
                                disabled={isProfileLoading}
                        >
                            {isProfileLoading ? 'Saving...' : 'Save'}
                        </Button>
                    </div>
                </form>
            </section>

            <section>
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2 mb-4">
                    Change Password</h2>
                <form onsubmit={handlePasswordUpdate} class="space-y-4">
                    <div>
                        <Label for="new-password"
                               class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">New
                            Password</Label>
                        <Input
                                type="password"
                                id="new-password"
                                bind:value={passwordData.newPassword}
                                class="w-full bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500"
                                placeholder="More than 8 characters"
                                disabled={isPasswordLoading}
                        />
                    </div>
                    <div>
                        <Label for="confirm-password"
                               class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Confirm
                            Password</Label>
                        <Input
                                type="password"
                                id="confirm-password"
                                bind:value={passwordData.newPasswordConfirm}
                                class="w-full bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500"
                                placeholder="Confirm your new password"
                                disabled={isPasswordLoading}
                        />
                    </div>
                    <div>
                        <Button
                                type="submit"
                                class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg transition-colors disabled:bg-blue-800 disabled:cursor-not-allowed"
                                disabled={isPasswordLoading}
                        >
                            {isPasswordLoading ? 'Saving...' : 'Save'}
                        </Button>
                    </div>
                </form>
            </section>
        </div>
    {/if}
</div>
<script lang="ts">
    import Pagination from "$lib/components/Pagination.svelte";
    import ConfirmationAlert from "$lib/components/ConfirmationAlert.svelte";
    import {Button} from "$lib/components/ui/button";
    import {toast} from "svelte-sonner";
    import {Edit, Trash2} from "@lucide/svelte";
    import {apiFetch} from "$lib/store/auth";

    type User = {
        id: number,
        username: string,
        email: string,
        role_name: string,
    };

    // Props from the parent component. This is correct.
    let {rowsPerPage = 20, searchQuery = ''} = $props();

    // Internal state for the component.
    let users = $state<User[] | null>(null);
    let editingUser = $state<User | null>(null);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);
    let totalItems = $state(0);
    let currentPage = $state(1);
    let activeUserId = $state<number | null>(null);
    let deleteUser = $state<User | null>(null);
    let previousSearchQuery = $state(searchQuery);
    
    let totalPages = $derived(Math.ceil(totalItems / rowsPerPage));

    async function loadUsers(page: number, query: string, limit: number) {
        isLoading = true;
        errorMessage = null;
        try {
            const url = new URL('/api/admin/users/list', window.location.origin);
            url.searchParams.set('page', String(page));
            // Use the 'limit' parameter which comes from rowsPerPage
            url.searchParams.set('pageSize', String(limit));
            if (query) {
                url.searchParams.set('search', query);
            }
            const response = await apiFetch(url.href);
            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to fetch user data');
            }
            const data = await response.json();
            users = data.items;
            totalItems = data.totalItems;
        } catch (error: any) {
            console.error('Failed to load users', error);
            errorMessage = error.message;
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        // This line establishes a dependency on `searchQuery`.
        // The effect will re-run whenever `searchQuery` changes.
        searchQuery;

        // By resetting the page here, we ensure that any new search starts from the beginning.
        // We don't need to check the previous value; the effect itself is the change detector.
        currentPage = 1;
    });

    // Effect 2: This effect is responsible for fetching the data.
    // It will run whenever its dependencies (currentPage, searchQuery, rowsPerPage) change.
    $effect(() => {
        // Now the logic is clean. We just call loadUsers with the current state of our dependencies.
        loadUsers(currentPage, searchQuery, rowsPerPage);
    });
</script>

<div class="border bg-card text-card-foreground rounded-lg shadow-sm overflow-x-auto">
    <table class="series-table text-sm w-full">
        <thead class="bg-muted/50 text-muted-foreground uppercase">
        <tr>
            <th scope="col" class="px-4 py-3 text-center">ID</th>
            <th scope="col" class="px-4 py-3 text-center">Username</th>
            <th scope="col" class="px-4 py-3 text-center">Email</th>
            <th scope="col" class="px-4 py-3 text-center">Role</th>
        </tr>
        </thead>
        <tbody>
        {#if isLoading}
            <tr>
                <td colspan="4" class="text-center py-8 text-muted-foreground">Loading user list...</td>
            </tr>
        {:else if errorMessage}
            <tr>
                <td colspan="4" class="text-center py-8 text-destructive">{errorMessage}</td>
            </tr>
        {:else if users && users.length > 0}
            {#each users as user (user.id)}
                <tr
                        class="border-b border-border hover:bg-muted/50 transition-colors cursor-pointer"
                >
                    {#if activeUserId === user.id}
                        <td colspan="4" class="px-4 py-2">
                            <div class="flex items-center justify-center space-x-2">
                                <Button
                                        size="iconLabel"
                                        class="hover:text-blue-800 hover:bg-blue-100 transition-colors cursor-pointer"
                                        title="Edit {user.username}"
                                >
                                    <Edit/>
                                    Edit
                                </Button>
                                <Button
                                        size="iconLabel"
                                        variant="destructive"
                                        class="hover:bg-destructive/90 transition-colors"
                                        title="Delete {user.username}"
                                >
                                    <Trash2/>
                                    Delete
                                </Button>
                            </div>
                        </td>
                    {:else}
                        <td class="px-4 py-3 text-foreground text-center">{user.id}</td>
                        <td class="px-4 py-3 font-medium text-foreground text-center">{user.username}</td>
                        <td class="px-4 py-3 text-foreground text-center">{user.email}</td>
                        <td class="px-4 py-3 text-foreground text-center">{user.role_name}</td>
                    {/if}
                </tr>
            {/each}
        {:else}
            <tr>
                <td colspan="4" class="text-center py-8 text-muted-foreground">No Users Found</td>
            </tr>
        {/if}
        </tbody>
    </table>
</div>

{#if totalPages > 1}
    <div class="flex justify-center mt-4">
        <Pagination bind:currentPage totalPages={totalPages}/>
    </div>
{/if}

<style>
    .series-table {
        min-width: 100%;
        text-align: left;
    }
</style>
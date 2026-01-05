<script lang="ts">
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import {type ColumnDef} from "@tanstack/table-core";
    import {renderSnippet} from "./ui/data-table";
    import {apiFetch} from "$lib/store/auth";
    import {toast} from "svelte-sonner";
    import ConfirmationAlert from "./ConfirmationAlert.svelte";
    import Pagination from "./Pagination.svelte";
    import Button from "./ui/button/button.svelte";
    import {Ellipsis, Trash2, UserCog} from "@lucide/svelte";
    import EditUser from "./EditUser.svelte";
    import {useDataGrid} from "./data-grid/parts";
    import DataGrid from "./data-grid/parts/data-grid.svelte";
    import {useWindowSize} from "./data-grid/hooks/use-window-size.svelte";

    type User = {
        id: number,
        username: string,
        email: string,
        role_name: string,
        role_id: number,
        is_active: boolean,
    };

    let {rowsPerPage = 25, searchQuery = ''} = $props();

    // Internal state for the component.
    let users = $state<User[]>([]);
    let editingUser = $state<User | null>(null);
    let errorMessage = $state<string | null>(null);
    let isLoading = $state(true);
    let totalItems = $state(0);
    let currentPage = $state(1);
    let deleteUser = $state<User | null>(null);
    let totalPages = $derived(Math.ceil(totalItems / rowsPerPage));

    // Layout Table
    const windowSize = useWindowSize({defaultHeight: 760});
    const gridHeight = $derived(Math.max(400, windowSize.height - 250));

    // Column Definitions
    const columns: ColumnDef<User, unknown>[] = [
        {
            accessorKey: 'id',
            header: 'ID',
            size: 80,
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'number'
                }
            }
        },
        {
            accessorKey: 'username',
            header: 'Username',
            size: 250,
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'short-text'
                }
            }
        },
        {
            accessorKey: 'email',
            header: 'Email',
            size: 250,
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'short-text'
                }
            }
        },
        {
            accessorKey: 'role_name',
            header: 'Role',
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'badge',
                    classMap: {
                        'Super Admin': 'bg-primary text-primary-foreground',
                        'Admin': 'bg-secondary text-secondary-foreground',
                        'Moderator': 'bg-blue-100 text-blue-700 border-blue-200 hover:bg-blue-200',
                        'User': 'bg-muted text-muted-foreground'
                    }
                }
            }
        },
        {
            accessorKey: 'is_active',
            header: 'Status',
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'badge',
                    options: [
                        {value: 'true', label: 'Active'},
                        {value: 'false', label: 'InActive'},
                    ],
                    variantMap: {
                        'true': 'outline',
                        'false': 'destructive'
                    },
                    classMap: {
                        'true': 'bg-green-100 text-green-700 border-green-200 hover:bg-green-100',
                        'false': 'bg-red-100 text-red-700 border-red-200 hover:bg-red-100'
                    }
                }
            }
        },
        {
            id: 'actions',
            header: 'Actions',
            size: 80,
            cell: ({row}) => renderSnippet(toolsActions, {user: row.original}),
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'actions'
                }
            }
        }
    ]

    const {table, ...dataGridProps} = useDataGrid({
        columns,
        data: () => users,
        onDataChange: (newData) => {
            users = newData;
        },
        getRowId: (row) => String(row.id),
        readOnly: true,
        enableColumnSelection: false,
        enableSearch: false,
    })

    async function loadUsers(page: number, query: string, limit: number) {
        isLoading = true;
        errorMessage = null;
        try {
            const url = new URL('/api/admin/users/paginated/list-search', window.location.origin);
            url.searchParams.set('page', String(page));
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

    async function confirmDelete() {
        if (!deleteUser) return;
        const userToDelete = deleteUser;
        deleteUser = null;

        const deleteRequest = async () => {
            const response = await apiFetch(`/api/admin/users/delete/${userToDelete.id}`, {
                method: 'DELETE',
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({
                    message: 'Failed to delete user'
                }));
                throw new Error(errorData.message);
            }
            return userToDelete.username;
        };

        toast.promise(deleteRequest(), {
            loading: `Deleting user ${userToDelete.username}...`,
            success: (username) => {
                loadUsers(currentPage, searchQuery, rowsPerPage);
                return `User ${username} deleted successfully`;
            },
            error: (err: any) => {
                const message = err instanceof Error ? err.message : "Unknown error";
                loadUsers(currentPage, searchQuery, rowsPerPage);
                return `Error deleting user. ${message}`;
            },
            finally: () => {
                activeUserId = null;
            }
        });
    }

    function handleEdit(user: User) {
        editingUser = user;
    }

    function handleDelete(user: User) {
        deleteUser = user;
    }

    function handleEditClose() {
        editingUser = null;
        activeUserId = null;
        loadUsers(currentPage, searchQuery, rowsPerPage);
    }

    function cancelDelete() {
        deleteUser = null;
    }

    $effect(() => {
        void searchQuery;

        // By resetting the page here, we ensure that any new search starts from the beginning.
        // We don't need to check the previous value; the effect itself is the change detector.
        currentPage = 1;
    });

    // It will run whenever its dependencies (currentPage, searchQuery, rowsPerPage) change.
    $effect(() => {
        loadUsers(currentPage, searchQuery, rowsPerPage);
    });
</script>

{#if editingUser}
    <EditUser user={editingUser} onClose={handleEditClose}/>
{/if}

{#if deleteUser}
    <ConfirmationAlert
            open={!!deleteUser}
            title="Are you sure you want to delete this user?"
            message={`This action will permanently delete user "${deleteUser.username}". This action cannot be undone.`}
            onConfirm={confirmDelete}
            onCancel={cancelDelete}
    />
{/if}

{#snippet toolsActions(user)}
    <div class="flex justify-center">
        <DropdownMenu.Root>
            <DropdownMenu.Trigger>
                {#snippet child({props})}
                    <Button {...props} variant="ghost" size="icon" class="h-8 w-8 p-0">
                        <span class="sr-only">Open menu</span>
                        <Ellipsis class="h-4 w-4"/>
                    </Button>
                {/snippet}
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
                <DropdownMenu.Label>Actions</DropdownMenu.Label>
                <DropdownMenu.Item onclick={() => handleEdit(user)}>
                    <UserCog class="mr-2 h-4 w-4"/>
                    Edit
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => handleDelete(user)} class="text-destructive focus:bg-destructive/10">
                    <Trash2 class="mr-2 h-4 w-4"/>
                    Delete
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>
{/snippet}

<div class="space-y-2 flex flex-col w-full">
    <DataGrid
            {...dataGridProps}
            {table}
            isLoading={isLoading}
            loadingMessage="Loading user data..."
            height={gridHeight}
            class="w-full"
    />

    {#if totalPages > 0}
        <div class="flex justify-center mt-2">
            <Pagination bind:currentPage totalPages={totalPages}/>
        </div>
    {/if}
</div>
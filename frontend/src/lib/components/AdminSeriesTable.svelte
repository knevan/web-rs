<script lang="ts">
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import EditSeries from "$lib/components/EditSeries.svelte";
    import {Button} from "$lib/components/ui/button";
    import {apiFetch} from "$lib/store/auth";
    import Pagination from "$lib/components/Pagination.svelte";
    import {
        FilePen,
        Wrench,
        Trash2,
        ExternalLink,
        Ellipsis,
        TableOfContents
    } from "@lucide/svelte";
    import RepairChapterSeries from "$lib/components/RepairChapterSeries.svelte";
    import ConfirmationAlert from "./ConfirmationAlert.svelte";
    import {toast} from "svelte-sonner";
    import ChapterList from "./ChapterList.svelte";
    import {useWindowSize} from "./data-grid/hooks/use-window-size.svelte";
    import type {ColumnDef} from "@tanstack/table-core";
    import {renderSnippet} from "./data-grid/data-logic";
    import {useDataGrid} from "./data-grid/parts";
    import DataGrid from "./data-grid/parts/data-grid.svelte";

    // Define the type for a series item
    type Series = {
        id: number;
        title: string;
        originalTitle: string | null;
        authors: string[];
        description: string;
        coverImageUrl: string;
        sourceUrl: string;
        lastUpdated: string;
        processingStatus: string;
    };

    let {rowsPerPage = 25, searchQuery = ''} = $props();

    let series = $state<Series[]>([]);
    let editingSeries = $state<Series | null>(null);
    let repairSeriesId = $state<number | null>(null);
    let viewChapterSeriesId = $state<Series | null>(null);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);
    let totalItems = $state(0);
    let currentPage = $state(1);
    let totalPages = $derived(Math.ceil(totalItems / rowsPerPage));
    let activeSeriesId = $state<number | null>(null);
    let deleteSeries = $state<Series | null>(null);
    let previousSearchQuery = $state(searchQuery);
    let prevRowsPerPage = $state(rowsPerPage);

    const windowSize = useWindowSize({defaultHeight: 760});
    const gridHeight = $derived(Math.max(400, windowSize.height - 250));

    // Column Definitions
    const columns: ColumnDef<Series, unknown>[] = [
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
            accessorKey: 'title',
            header: 'Series Name',
            size: 250,
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'long-text'
                }
            }
        },
        {
            accessorKey: 'authors',
            header: 'Authors',
            size: 100,
            cell: ({getValue}) => (getValue() as string[]).join(', '),
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
            accessorKey: 'lastUpdated',
            header: 'Last Updated',
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'date'
                }
            }
        },
        {
            accessorKey: 'processingStatus',
            header: 'Status',
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'badge',
                    classMap: {
                        // GREEN / SUCCESS (Active/Done)
                        'Available': 'bg-green-100 text-green-700 border-green-200',
                        'Completed': 'bg-green-100 text-green-700 border-green-200',
                        'Ongoing': 'bg-emerald-100 text-emerald-700 border-emerald-200',

                        // BLUE (Transient/Processing)
                        'Processing': 'bg-blue-100 text-blue-700 border-blue-200 animate-pulse',
                        'Pending': 'bg-sky-100 text-sky-700 border-sky-200',

                        // YELLOW (Inactive/Paused)
                        'Hiatus': 'bg-yellow-100 text-yellow-700 border-yellow-200',
                        'Discontinued': 'bg-orange-100 text-orange-700 border-orange-200',

                        // RED (Error/Deletion)
                        'Error': 'bg-red-100 text-red-700 border-red-200 font-bold',
                        'DeletionFailed': 'bg-red-100 text-red-700 border-red-200 font-bold',

                        // GRAY (System Operations)
                        'PendingDeletion': 'bg-gray-100 text-gray-700 border-gray-200',
                        'Deleting': 'bg-gray-200 text-gray-700 border-gray-300 animate-pulse'
                    }
                }
            }
        },
        {
            id: 'source',
            header: 'Source',
            cell: ({row}) => renderSnippet(sourceSnippet, {url: row.original.sourceUrl}),
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'url'
                }
            }
        },
        {
            id: 'actions',
            header: 'Actions',
            size: 90,
            cell: ({row}) => renderSnippet(actionsSnippet, {manga: row.original}),
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
        data: () => series,
        onDataChange: (newData) => {
            series = newData
        },
        getRowId: (row) => String(row.id),
        readOnly: true,
        enableColumnSelection: false,
        enableSearch: false,
    })

    // Load series data
    async function loadSeries(page: number, query: string) {
        isLoading = true;
        errorMessage = null;

        try {
            const url = new URL('/api/admin/series/paginated/list-search', window.location.origin);
            url.searchParams.set('page', String(page));
            url.searchParams.set('pageSize', rowsPerPage.toString());
            if (query) {
                url.searchParams.append('search', query);
            }

            const response = await apiFetch(url.href);
            // Fetch logic here
            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || "Failed to fetch series data");
            }
            const data = await response.json();
            series = data.items;
            totalItems = data.totalItems;
        } catch (error: any) {
            console.error("Failed to load series", error);
            errorMessage = error.message;
        } finally {
            isLoading = false;
        }
    }

    async function confirmDelete() {
        if (!deleteSeries) return;

        const seriesToDelete = deleteSeries;
        deleteSeries = null;

        const deleteRequest = async () => {
            const response = await apiFetch(`/api/admin/series/delete/${seriesToDelete.id}`, {
                method: "DELETE",
            });
            if (!response.ok) {
                const errorData = await response.json().catch(() => ({
                    message: "Failed to delete series",
                }));
                throw new Error(errorData.message);
            }
            return seriesToDelete.title;
        };

        toast.promise(deleteRequest(), {
            loading: `Scheduling "${seriesToDelete.title}" for deletion...`,
            success: (title) => {
                loadSeries(currentPage, searchQuery);
                return `Series "${title}" deleted successfully!`;
            },
            error: (err) => {
                const message = err instanceof Error ? err.message : "Unknown error";
                loadSeries(currentPage, searchQuery);
                return `Failed to delete series: ${message}`;
            },
            finally: () => {
                activeSeriesId = null;
            }
        });
    }

    function cancelDelete() {
        deleteSeries = null;
    }

    function handleEditClose() {
        editingSeries = null;
        activeSeriesId = null;
        // We still reload the data to simulate a refresh after editing.
        loadSeries(currentPage, searchQuery);
    }

    function handleRowClick(id: number) {
        activeSeriesId = activeSeriesId === id ? null : id;
    }

    function handleRepairClose() {
        repairSeriesId = null;
    }

    $effect(() => {
        // This effect now has a clear, single path.

        let pageToLoad = currentPage;

        // Check if a new search has been performed.
        if (searchQuery !== previousSearchQuery) {
            // If it's a new search, we must load page 1.
            pageToLoad = 1;
            // Also, update the current page state if it's not already 1.
            if (currentPage !== 1) {
                currentPage = 1;
            }
            // Update the tracker for the next run.
            previousSearchQuery = searchQuery;
        }

        // Call loadSeries ONCE with the guaranteed correct page and query.
        loadSeries(pageToLoad, searchQuery);
    });
</script>

<!-- Conditionally render the modals based on their state -->
{#if editingSeries}
    <EditSeries series={editingSeries} onclose={handleEditClose}/>
{/if}

{#if repairSeriesId}
    <RepairChapterSeries seriesId={repairSeriesId} onclose={handleRepairClose}/>
{/if}

{#if viewChapterSeriesId}
    <ChapterList
            seriesId={viewChapterSeriesId.id}
            seriesTitle={viewChapterSeriesId.title}
            onClose={() => viewChapterSeriesId = null}
    />
{/if}

{#if deleteSeries}
    <ConfirmationAlert
            open={!!deleteSeries}
            title="Are you sure to delete series?"
            message={`This action will permanently mark the series "${deleteSeries.title}" for deletion. This cannot be undone.`}
            onConfirm={confirmDelete}
            onCancel={cancelDelete}
    />
{/if}

{#snippet sourceSnippet({url}: { url: string })}
    <div class="flex justify-center">
        <a href={url} target="_blank" rel="noopener noreferrer">
            <ExternalLink class="size-4"/>
        </a>
    </div>
{/snippet}

{#snippet actionsSnippet({manga}: { manga: Series })}
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

                <DropdownMenu.Item onclick={() => editingSeries = manga}>
                    <FilePen class="mr-2 h-4 w-4"/>
                    Edit
                </DropdownMenu.Item>

                <DropdownMenu.Item onclick={() => viewChapterSeriesId = manga}>
                    <TableOfContents class="mr-2 h-4 w-4"/>
                    Chapters
                </DropdownMenu.Item>

                <DropdownMenu.Item onclick={() => repairSeriesId = manga.id}>
                    <Wrench class="mr-2 h-4 w-4"/>
                    Repair
                </DropdownMenu.Item>

                <DropdownMenu.Separator/>

                <DropdownMenu.Item onclick={() => deleteSeries = manga}
                                   class="text-destructive focus:bg-destructive/10">
                    <Trash2 class="mr-2 h-4 w-4"/>
                    Delete
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>
{/snippet}


<div class="space-y-2 flex flex-col w-full mt-4">
    <DataGrid
            {...dataGridProps}
            {table}
            isLoading={isLoading}
            loadingMessage="Loading series data..."
            emptyMessage={errorMessage ? errorMessage : "No series found"}
            height={gridHeight}
            class="w-full"
    />

    {#if totalPages > 1}
        <div class="flex justify-center mt-4">
            <Pagination
                    bind:currentPage={currentPage}
                    totalPages={totalPages}
            />
        </div>
    {/if}
</div>
<style>
    .series-table {
        min-width: 100%;
        text-align: left;

    }
</style>
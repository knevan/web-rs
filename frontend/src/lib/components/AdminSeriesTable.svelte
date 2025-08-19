<script lang="ts">
    import EditSeries from "$lib/components/EditSeries.svelte";
    import {Button} from "$lib/components/ui/button";
    // import {apiFetch} from "$lib/store/auth"; // No longer needed for mocking
    import Pagination from "$lib/components/Pagination.svelte";
    import {FilePen, Wrench, Trash2} from "@lucide/svelte";
    import RepairChapterSeries from "$lib/components/RepairChapterSeries.svelte";
    import ConfirmationAlert from "./ConfirmationAlert.svelte";
    import {toast} from "svelte-sonner";

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

    let {rowsPerPage = 20, searchQuery = ''} = $props();

    let series = $state<Series[] | null>(null);
    let editingSeries = $state<Series | null>(null);
    let repairSeriesId = $state<number | null>(null);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);
    let totalItems = $state(0);
    let currentPage = $state(1);
    let totalPages = $derived(Math.ceil(totalItems / rowsPerPage));
    let activeSeriesId = $state<number | null>(null);
    let deleteSeries = $state<Series | null>(null);
    let previousSearchQuery = $state(searchQuery);
    // let prevRowsPerPage = $state(rowsPerPage);

    // This function now loads data from our MOCK_DATA array instead of an API.
    async function loadSeries(page: number, query: string) {
        isLoading = true;
        errorMessage = null;

        try {
            const url = new URL('/api/admin/series/list', window.location.origin);
            url.searchParams.set('page', String(page));
            url.searchParams.set('pageSize', rowsPerPage.toString());
            if (query) {
                url.searchParams.append('search', query);
            }

            const response = await fetch(url.href);
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
            const response = await fetch(`/api/admin/series/delete/${seriesToDelete.id}`, {
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
            position: "top-center",
            richColors: true,
            duration: 2000,
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

{#if deleteSeries}
    <ConfirmationAlert
            open={!!deleteSeries}
            title="Are you sure to delete series?"
            message={`This action will permanently mark the series "${deleteSeries.title}" for deletion. This cannot be undone.`}
            onConfirm={confirmDelete}
            onCancel={cancelDelete}
    />
{/if}

<div class="overflow-x-auto bg-white rounded-lg shadow">
    <table class="min-w-full text-sm text-left text-gray-500">
    </table>
</div>

<div class="border bg-card text-card-foreground rounded-lg shadow-sm overflow-x-auto">
    <table class="series-table text-sm w-full">
        <thead class="bg-muted/50 text-muted-foreground uppercase">
        <tr>
            <th scope="col" class="px-4 py-3 text-center">Series Name</th>
            <th scope="col" class="px-4 py-3 text-center">Series Id</th>
            <th scope="col" class="px-4 py-3 text-center">Author</th>
            <th scope="col" class="px-4 py-3 text-center">Last Updated</th>
            <th scope="col" class="px-4 py-3 text-center">Status</th>
            <th scope="col" class="px-4 py-3 text-center">Source Urls</th>
        </tr>
        </thead>
        <tbody>
        {#if isLoading}
            <tr>
                <td colspan="5" class="text-center py-8 text-muted-foreground">
                    Loading manga list...
                </td>
            </tr>
        {:else if errorMessage}
            <tr>
                <td colspan="5"
                    class="text-center py-8 text-destructive">{errorMessage}</td>
            </tr>
        {:else if series && series.length > 0}
            {#each series as manga (manga.id)}
                <tr class="border-b border-border hover:bg-muted/50 transition-colors cursor-pointer"
                    onclick={() => handleRowClick(manga.id)}>
                    <td class="px-4 py-3 font-medium text-foreground justify-center">{manga.title}</td>
                    {#if activeSeriesId === manga.id}
                        <td colspan="4" class="px-4 py-2">
                            <div class="flex items-center justify-center space-x-2">
                                <Button onclick={(e) => { e.stopPropagation(); editingSeries = manga; }}
                                        size="iconLabel"
                                        class="hover:text-blue-800 hover:bg-blue-100 transition-colors cursor-pointer"
                                        title="Edit {manga.title}">
                                    <FilePen/>
                                    Edit
                                </Button>
                                <Button onclick={(e) => { e.stopPropagation(); repairSeriesId = manga.id }}
                                        size="iconLabel"
                                        title="Repair {manga.title}">
                                    <Wrench/>
                                    Repair
                                </Button>
                                <Button onclick={(e) => {e.stopPropagation(); deleteSeries = manga; }}
                                        size="iconLabel"
                                        variant="destructive"
                                        class="hover:bg-destructive/90 transition-colors"
                                        title="Delete {manga.title}">
                                    <Trash2/>
                                    Delete
                                </Button>
                            </div>
                        </td>
                    {:else}
                        <td class="px-4 py-3 text-foreground text-center">{manga.id}</td>
                        <td class="px-4 py-3 text-foreground">{manga.authors.join(', ')}</td>
                        <td class="px-4 py-2 text-left">{manga.lastUpdated}</td>
                        <td class="px-4 py-2 text-foreground text-center">{manga.processingStatus}</td>
                        <td class="px-4 py-3 text-foreground">
                            <a href={manga.sourceUrl} target="_blank"
                               class="text-primary hover:underline">
                                Source URLs
                            </a>
                        </td>
                    {/if}
                </tr>
            {/each}
        {:else}
            <tr>
                <td colspan="5" class="text-center py-8 text-muted-foreground">No Manga
                    Found
                </td>
            </tr>
        {/if}
        </tbody>
    </table>
</div>

{#if totalPages > 1}
    <div class="flex justify-center mt-4">
        <Pagination
                bind:currentPage={currentPage}
                totalPages={totalPages}
        />
    </div>
{/if}

<style>
    .series-table {
        min-width: 100%;
        text-align: left;

    }
</style>
<script lang="ts">
    import EditSeries from "$lib/components/EditSeries.svelte";
    import {Button} from "$lib/components/ui/button";
    // import {apiFetch} from "$lib/store/auth"; // No longer needed for mocking
    import Pagination from "$lib/components/Pagination.svelte";
    import {FilePen, Wrench, Trash2} from "@lucide/svelte";
    import RepairChapterSeries from "$lib/components/RepairChapterSeries.svelte";

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
    };

    // --- Mock Data Generation ---
    // A helper function to create a single mock series item.
    function createMockSeries(id: number): Series {
        return {
            id: id,
            title: `Manga Title ${id}`,
            originalTitle: `Original Manga Title ${id}`,
            authors: [`Author ${id % 5 + 1}`],
            description: `This is a mock description for manga series number ${id}.`,
            coverImageUrl: `https://via.placeholder.com/150/0000FF/808080?Text=Manga+${id}`,
            sourceUrl: `#`,
            lastUpdated: new Date(Date.now() - id * 1000 * 3600).toISOString(),
        };
    }

    // Generate a large array of mock data to test pagination thoroughly.
    const MOCK_DATA: Series[] = Array.from({length: 2500}, (_, i) => createMockSeries(i + 1));
    // --- End of Mock Data Generation ---


    let series = $state<Series[] | null>(null);
    let editingSeries = $state<Series | null>(null);
    let repairSeriesId = $state<number | null>(null);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);
    let totalItems = $state(0);
    let currentPage = $state(1);
    let pageSize = $state(25); // You can change this to test different page sizes
    let totalPages = $derived(Math.ceil(totalItems / pageSize));
    let activeSeriesId = $state<number | null>(null);

    // This function now loads data from our MOCK_DATA array instead of an API.
    async function loadSeries(page: number) {
        isLoading = true;
        errorMessage = null;

        // Simulate a network delay to make the loading state visible.
        setTimeout(() => {
            try {
                // Calculate the start and end index for the current page.
                const start = (page - 1) * pageSize;
                const end = start + pageSize;

                // Get the slice of data for the current page.
                const pagedItems = MOCK_DATA.slice(start, end);

                series = pagedItems;
                totalItems = MOCK_DATA.length; // Set total items to the full length of our mock data.

            } catch (error: any) {
                console.error("Failed to load mock series", error);
                errorMessage = error.message;
            } finally {
                isLoading = false;
            }
        }, 500); // 500ms delay
    }

    function handleEditClose() {
        editingSeries = null;
        activeSeriesId = null;
        // We still reload the data to simulate a refresh after editing.
        loadSeries(currentPage);
    }

    function handleRowClick(id: number) {
        activeSeriesId = activeSeriesId === id ? null : id;
    }

    function handleRepairClose() {
        repairSeriesId = null;
    }

    // This $effect hook will run whenever `currentPage` changes,
    // triggering our new mock `loadSeries` function.
    $effect(() => {
        loadSeries(currentPage);
    });

</script>

<!-- Conditionally render the modals based on their state -->
{#if editingSeries}
    <EditSeries series={editingSeries} onclose={handleEditClose}/>
{/if}

{#if repairSeriesId}
    <RepairChapterSeries seriesId={repairSeriesId} onclose={handleRepairClose}/>
{/if}

<div class="overflow-x-auto bg-white rounded-lg shadow">
    <table class="min-w-full text-sm text-left text-gray-500">
    </table>
</div>

<div class="border bg-card text-card-foreground rounded-lg shadow-sm overflow-x-auto">
    <table class="series-table text-sm w-full">
        <thead class="bg-muted/50 text-muted-foreground uppercase">
        <tr>
            <th scope="col" class="px-4 py-3">Series Name</th>
            <th scope="col" class="px-4 py-3">Series Id</th>
            <th scope="col" class="px-4 py-3">Author</th>
            <th scope="col" class="px-4 py-3">Last Updated</th>
            <th scope="col" class="px-4 py-3">Source Urls</th>
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
                                <Button onclick={(e) => {e.stopPropagation();}}
                                        size="iconLabel"
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
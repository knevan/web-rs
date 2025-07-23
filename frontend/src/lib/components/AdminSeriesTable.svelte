<script lang="ts">
    import EditSeries from "$lib/components/EditSeries.svelte";
    import {Button} from "$lib/components/ui/button";
    import {apiFetch} from "$lib/store/auth";

    type Series = {
        id: number;
        title: string;
        originalTitle: string | null;
        authors: string[];
        description: string;
        coverImageUrl: string;
        sourceUrl: string;
        // totalChapters: number;
        // lastChapter: number;
        lastUpdated: string;
    };

    type SeriesApiResponse = {
        id: number;
        title: string;
        original_title: string;
        description: string;
        cover_image_url: string;
        source_url: string;
        authors: string[];
        last_updated: string;
    }

    let series = $state<Series[] | null>(null);
    let editingSeries = $state<Series | null>(null);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);
    let totalItems = $state(0);

    // let {series = []}: { series: Series[] } = $props();
    async function loadSeries() {
        isLoading = true;
        errorMessage = null;
        try {
            const response = await apiFetch("/api/admin/series/list?page=1&page_size=25");

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to fetch series. Status: ${response.status}');
            }

            const data: {
                items: SeriesApiResponse[],
                total_items: number
            } = await response.json();

            series = data.items.map(item => ({
                id: item.id,
                title: item.title,
                originalTitle: item.original_title,
                authors: item.authors,
                description: item.description,
                coverImageUrl: item.cover_image_url,
                sourceUrl: item.source_url,
                lastUpdated: item.last_updated,
            }));

            totalItems = data.total_items;

        } catch (error: any) {
            console.error("Failed to load series", error);
            errorMessage = error.message;
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        loadSeries();
    });

    function handleClose() {
        editingSeries = null;
        // Optionally, you can add logic here to refresh the table data
        loadSeries();
    }

    const editIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>`;

</script>

{#if editingSeries}
    <EditSeries series={editingSeries} onclose={handleClose}/>
{/if}

<div class="overflow-x-auto bg-white rounded-lg shadow">
    <table class="min-w-full text-sm text-left text-gray-500">
        <thead class="bg-gray-100 text-xs text-gray-700 uppercase">
        <tr>
            <th scope="col" class="px-4 py-3">Edit</th>
            <th scope="col" class="px-4 py-3">Manga Name</th>
            <th scope="col" class="px-4 py-3">Manga Id</th>
            <th scope="col" class="px-4 py-3">Author</th>
            <!--th scope="col" class="px-4 py-3">Total Chapter</th>-->
            <!--<th scope="col" class="px-4 py-3">Last Chapters</th>-->
            <th scope="col" class="px-4 py-3">Last Updated</th>
            <th scope="col" class="px-4 py-3">Source Urls</th>
        </tr>
        </thead>
        <tbody>
        {#if isLoading}
            <tr>
                <td colspan="6" class="text-center py-8 text-gray-500">Loading manga
                    list...
                </td>
            </tr>
        {:else if errorMessage}
            <tr>
                <td colspan="6" class="text-center py-8 text-gray-500">{errorMessage}</td>
            </tr>
        {:else if series && series.length > 0}
            {#each series as manga (manga.id)}
                <tr class="border-b hover:bg-gray-50">
                    <td class="px-4 py-3">
                        <Button onclick={ () => editingSeries = manga }
                                class="p-1 text-blue-600 hover:text-blue-800 hover:bg-blue-100 rounded-full transition-colors"
                                title="Edit {manga.title}">
                            {@html editIcon}
                        </Button>
                    </td>
                    <td class="px-4 py-3 font-medium text-gray-900">{manga.title}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.id}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.authors.join(', ')}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.lastUpdated}</td>
                    <td class="px-4 py-3 text-gray-900">
                        <a href={manga.sourceUrl} target="_blank"
                           class="text-blue-600 hover:underline">
                            Source URLs
                        </a>
                    </td>
                </tr>
            {/each}
        {:else}
            <tr>
                <td colspan="8" class="text-center py-8 text-gray-500">No Manga Found</td>
            </tr>
        {/if}
        </tbody>
    </table>
</div>
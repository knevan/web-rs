<script lang="ts">
    import EditSeries from "$lib/components/EditSeries.svelte";
    import {Button} from "$lib/components/ui/button";

    type Series = {
        id: number;
        title: string;
        originalTitle: string | null;
        authors: string[];
        description: string;
        coverImageUrl: string;
        sourceUrl: string;
        totalChapters: number;
        lastChapter: number;
        lastUpdated: string;
    };

    let {series = []}: { series: Series[] } = $props();

    let editingSeries = $state<Series | null>(null);

    function handleClose() {
        editingSeries = null;
        // Optionally, you can add logic here to refresh the table data
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
            <th scope="col" class="px-4 py-3">Manga Name</th>
            <th scope="col" class="px-4 py-3">Manga Id</th>
            <th scope="col" class="px-4 py-3">Author</th>
            <th scope="col" class="px-4 py-3">Total Chapter</th>
            <th scope="col" class="px-4 py-3">Last Chapters</th>
            <th scope="col" class="px-4 py-3">Last Updated</th>
            <th scope="col" class="px-4 py-3">Source Urls</th>
        </tr>
        </thead>
        <tbody>
        {#if series.length > 0}
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
                    <td class="px-4 py-3 text-gray-900">{manga.totalChapters}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.lastChapter}</td>
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
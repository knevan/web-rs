<script lang="ts">
    import {Tabs} from "bits-ui";

    let {mangas = []} = $props();
</script>

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
        {#if mangas.length > 0}
            {#each mangas as manga (manga.id)}
                <tr class="border-b hover:bg-gray-50">
                    <td class="px-4 py-3 font-medium text-gray-900">{manga.name}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.id}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.author}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.totalChapters}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.lastChapter}</td>
                    <td class="px-4 py-3 text-gray-900">{manga.lastUpdated}</td>
                    <td class="px-4 py-3 text-gray-900">
                        <a href={manga.sourceUrls} target="_blank" class="text-blue-600 hover:underline">
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
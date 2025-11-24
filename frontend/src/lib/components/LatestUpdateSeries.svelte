<script lang="ts">
    import Pagination from "$lib/components/Pagination.svelte";
    import {goto} from "$app/navigation";
    import slugify from "slugify";

    type LatestRelease = {
        id: number;
        title: string;
        original_title: string;
        description: string;
        authors: string[];
        cover_image_url: string;
        last_chapter_found_in_storage?: number;
        updated_at: string;
        chapter_title?: string;
    }

    type PaginatedResponse = {
        items: LatestRelease[];
        total_items: number;
    }

    const ITEMS_PER_PAGE = 50;

    let seriesList = $state<LatestRelease[]>([]);
    let isLoading = $state(true);
    let currentPage = $state(1);
    let totalItems = $state(0);
    let error = $state<string | null>(null);

    const totalPages = $derived(Math.ceil(totalItems / ITEMS_PER_PAGE));

    $effect(() => {
        const fetchLatestReleases = async () => {
            isLoading = true;
            error = null;
            try {
                const response = await fetch(`/api/series/latest-updated-series?page=${currentPage}&limit=${ITEMS_PER_PAGE}`);
                if (!response.ok) {
                    throw new Error(response.statusText);
                }
                const data: PaginatedResponse = await response.json();
                seriesList = data.items;
                totalItems = data.total_items;
            } catch (err) {
                console.log(err);
                error = err instanceof Error ? err.message : 'Unknown error';
                seriesList = [];
                totalItems = 0;
            } finally {
                isLoading = false;
            }
        };
        fetchLatestReleases();
    })

    function handleSeriesClick(series: LatestRelease) {
        const seriesSlug = slugify(series.title || '', {lower: true});
        goto(`/manga/${series.id}/${seriesSlug}`)
    }

    function formatRelativeTime(datestring: string): string {
        if (!datestring) return '';
        const date = new Date(datestring);
        const now = new Date();
        let seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        if (seconds < 60) return 'Just Now';

        const interval = {
            day: 86400,
            hour: 3600,
            minute: 60
        };

        const result: string[] = [];
        for (const [unit, unitSeconds] of Object.entries(interval)) {
            if (result.length >= 2) break;
            const count = Math.floor(seconds / unitSeconds);
            if (count > 0) {
                result.push(`${count} ${unit}${count > 1 ? 's' : ''}`);
                seconds %= unitSeconds;
            }
        }
        return result.length > 0 ? `${result.join(', ')}` : 'Just Now';
    }
</script>

<section class="flex flex-col gap-4">
    <h2 class="text-xl font-bold">Latest Release</h2>

    {#if isLoading}
        <p class="text-center text-gray-500 py-10">Loading...</p>
    {:else if error}
        <p class="text-center text-red-500 py-10">Error: {error}</p>
    {:else if seriesList.length === 0}
        <p class="text-center text-gray-500 py-10">No recently updated series found in the last week.</p>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-4">
            {#each seriesList as series (series.id)}
                <div
                        class="flex items-start gap-4 p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800/50 cursor-pointer transition-colors"
                        onclick={() => handleSeriesClick(series)}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && handleSeriesClick(series)}
                >
                    <img
                            src={series.cover_image_url}
                            alt="Cover for {series.title}"
                            class="w-20 h-28 object-cover rounded-md flex-shrink-0"
                            loading="lazy"
                    />
                    <div class="flex flex-col pt-1">
                        <h3 class="font-semibold text-md leading-tight">{series.title}</h3>
                        {#if series.last_chapter_found_in_storage}
                            <a
                                    href="/manga/{series.id}/{slugify(series.title, { lower: true })}/read-chapter/{series.last_chapter_found_in_storage}"
                                    class="text-sm text-blue-500 hover:underline"
                            >
                                Chapter {series.last_chapter_found_in_storage}
                            </a>
                        {/if}
                        <p class="text-sm text-gray-600 dark:text-gray-300 line-clamp-1">
                            {series.original_title}
                        </p>
                        <p class="text-sm text-gray-600 dark:text-gray-300 line-clamp-1">
                            {series.authors.join(', ') || 'N/A'}
                        </p>
                        <p class="text-sm text-gray-600 dark:text-gray-300 line-clamp-2">
                            {series.description}
                        </p>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                            {formatRelativeTime(series.updated_at)}
                        </p>
                    </div>

                </div>
            {/each}
        </div>
        {#if totalPages > 1}
            <div class="flex justify-center mt-6">
                <Pagination bind:currentPage totalPages={totalPages}/>
            </div>
        {/if}
    {/if}
</section>
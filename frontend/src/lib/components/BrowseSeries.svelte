<script lang="ts">
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import {Check, ChevronDown, RotateCcw, X} from "@lucide/svelte";
    import {Button} from '$lib/components/ui/button';
    import slugify from "slugify";

    type BrowseSeriesItem = {
        id: number;
        title: string;
        original_title: string | null;
        authors: string[];
        description: string;
        cover_image_url: string;
        categories: string[];
        last_chapter_found_in_storage: number | null;
        updated_at: string;
    }

    type PaginatedResult = {
        items: BrowseSeriesItem[];
        total_items: number;
    };

    type CategoryTag = {
        id: number;
        name: string;
    };

    type FilterState = 'neutral' | 'include' | 'exclude';
    type SortOrder = 'updated' | 'new' | 'views' | 'ratings';

    let allTags = $state<CategoryTag[]>([]);
    let tagFilterState = $state<Record<number, FilterState>>({});
    let seriesResult = $state<PaginatedResult | null>(null);
    let isLoading = $state(false);
    let error = $state<string | null>(null);
    let page = $state(1);
    let fetchController: AbortController | null = null;
    let orderBy = $state<SortOrder>('updated');

    const PAGE_SIZE = 50;

    const includedTags = $derived(
        Object.entries(tagFilterState)
            .filter(([, state]) => state === 'include')
            .map(([id]) => +id)
    );

    const excludedTags = $derived(
        Object.entries(tagFilterState)
            .filter(([, state]) => state === 'exclude')
            .map(([id]) => +id)
    );

    const sortLabels: Record<SortOrder, string> = {
        updated: 'Last Updated',
        new: 'Newest',
        views: 'Most Views',
        ratings: 'Highest Rated',
    }

    $effect(() => {
        async function fetchTags() {
            try {
                const response = await fetch('/api/series/tags');
                if (!response.ok) throw new Error('Failed to fetch tags');
                const tags: CategoryTag[] = await response.json();
                allTags = tags.sort((a, b) => a.name.localeCompare(b.name));

                // Initialize filter state for each tag to 'neutral'
                const initialState: Record<number, FilterState> = {};
                for (const tag of allTags) {
                    initialState[tag.id] = 'neutral';
                }
                tagFilterState = initialState;
                await handleSearch();
            } catch (e) {
                error = e instanceof Error ? e.message : 'An unknown error occurred';
            }
        }

        fetchTags();
    });

    async function fetchFilteredSeries(signal: AbortSignal) {
        isLoading = true;
        error = null;

        try {
            const query = new URLSearchParams({
                page: page.toString(),
                page_size: PAGE_SIZE.toString(),
                order_by: orderBy,
            });

            if (includedTags.length > 0) {
                query.set('include', includedTags.join(','));
            }
            if (excludedTags.length > 0) {
                query.set('exclude', excludedTags.join(','));
            }

            const response = await fetch(`/api/series/browse?${query.toString()}`, {signal});

            if (!response.ok) {
                throw new Error(`Server responded with ${response.status}`);
            }

            seriesResult = await response.json();
        } catch (e) {
            if (e instanceof Error && e.name !== 'AbortError') {
                error = e.message;
            }
        } finally {
            if (!signal.aborted) {
                isLoading = false;
            }
        }
    }

    async function handleSearch() {
        if (fetchController) {
            fetchController.abort();
        }
        fetchController = new AbortController();
        await fetchFilteredSeries(fetchController.signal);
    }

    function handleTagClick(tagId: number) {
        const currentState = tagFilterState[tagId];
        let nextState: FilterState = 'neutral';

        if (currentState === 'neutral') {
            nextState = 'include';
        } else if (currentState === 'include') {
            nextState = 'exclude';
        } else if (currentState === 'exclude') {
            nextState = 'neutral';
        }

        tagFilterState[tagId] = nextState;
        page = 1;
    }

    function resetFilters() {
        const resetState: Record<number, FilterState> = {};
        for (const tag of allTags) {
            resetState[tag.id] = 'neutral';
        }
        tagFilterState = resetState;
        page = 1;
        handleSearch();
    }

    function handleSortChange(newOrder: SortOrder) {
        if (orderBy === newOrder) return;
        orderBy = newOrder;
        page = 1;
        handleSearch();
    }

    function getStateClasses(state: FilterState): string {
        switch (state) {
            case 'include':
                return 'text-green-500 dark:text-green-400';
            case 'exclude':
                return 'text-red-500 dark:text-red-400';
            default:
                return 'text-gray-800 dark:text-gray-200';
        }
    }

    function formatRelativeTime(datestring: string): string {
        if (!datestring) return 'Unknown';

        const date = new Date(datestring);
        const now = new Date();
        // Calculate the difference in seconds between now and the provided date
        let seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        // If the difference is less than a minute, return "Just now"
        if (seconds < 60) return "Just Now";

        const intervals = {
            year: 31536000,
            month: 2592000,
            week: 604800,
            day: 86400,
            hour: 3600,
            minute: 60
        };

        const result = [];

        for (const [unit, unitSeconds] of Object.entries(intervals)) {
            // Show two most significant units
            if (result.length >= 2) {
                break;
            }

            const count = Math.floor(seconds / unitSeconds);
            if (count > 0) {
                // Add unit and its count to result array
                result.push(`${count} ${unit}${count > 1 ? 's' : ''}`);
                seconds %= unitSeconds;
            }
        }
        if (result.length === 0) return "Just Now";

        return result.join(', ');
    }
</script>

<svelte:head>
    <title>Browse Series - Manga Reader</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-1 py-8 text-gray-900 dark:text-white">
    <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm mb-8">
        <div class="p-4 sm:p-6">
            <h2 class="text-xl sm:text-2xl font-semibold mb-4 text-gray-900 dark:text-white">
                Filter
            </h2>

            <hr class="my-4 border-gray-300 dark:border-gray-600"/>

            {#if allTags.length > 0}
                <div class="max-h-[300px] sm:max-h-80 overflow-y-auto">
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-x-4 gap-y-2">
                        {#each allTags as tag (tag.id)}
                            <button
                                    type="button"
                                    onclick={() => handleTagClick(tag.id)}
                                    class="flex items-center gap-2 p-1 text-sm font-medium cursor-pointer select-none rounded-md hover:bg-gray-500/10 transition-colors w-full text-left"
                                    aria-pressed={tagFilterState[tag.id] !== 'neutral'}
                                    aria-label="Toggle {tag.name} filter"
                            >
								<span
                                        class="flex-shrink-0 w-4 h-4 flex items-center justify-center font-mono font-bold {getStateClasses(tagFilterState[tag.id])}"
                                >
									{#if tagFilterState[tag.id] === 'include'}
										<Check size={16}/>
									{:else if tagFilterState[tag.id] === 'exclude'}
										<X size={16}/>
									{:else}
										?
									{/if}
								</span>

                                <span class="truncate flex-1 {getStateClasses(tagFilterState[tag.id])}">
									{tag.name}
								</span>
                            </button>
                        {/each}
                    </div>
                </div>

                <hr class="my-4 border-gray-300 dark:border-gray-700/80"/>

                <div class="flex justify-end gap-2">
                    <Button
                            onclick={resetFilters}
                            variant="secondary"
                            class="flex items-center justify-center gap-2"
                    >
                        <RotateCcw size={16}/>
                        <span>Reset</span>
                    </Button>
                    <Button onclick={handleSearch} class="flex items-center justify-center gap-2">
                        <span>Search</span>
                    </Button>
                </div>

                {#if includedTags.length > 0 || excludedTags.length > 0}
                    <div
                            class="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg"
                    >
                        <div class="text-sm text-blue-800 dark:text-blue-200">
                            <span class="font-medium">Active Filters:</span>
                            {#if includedTags.length > 0}
								<span class="ml-2">
									<span class="text-green-600 dark:text-green-400 font-medium">Include:</span>
                                    {includedTags.length} tags
								</span>
                            {/if}
                            {#if excludedTags.length > 0}
								<span class="ml-2">
									<span class="text-red-600 dark:text-red-400 font-medium">Exclude:</span>
                                    {excludedTags.length} tags
								</span>
                            {/if}
                        </div>
                    </div>
                {/if}
            {:else}
                <div class="flex items-center justify-center py-8">
                    <div class="text-center">
                        <div
                                class="w-8 h-8 border-2 border-gray-300 border-t-blue-500 rounded-full animate-spin mx-auto mb-3"
                        ></div>
                        <p class="text-gray-500 dark:text-gray-400">Loading tags...</p>
                    </div>
                </div>
            {/if}
        </div>
    </div>

    <div class="flex justify-end gap-5">
        <DropdownMenu.Root>
            <DropdownMenu.Trigger>
                <Button variant="outline" class="w-[140px] justify-between">
                    {sortLabels[orderBy]}
                    <ChevronDown class="ml-2 h-4 w-4 shrink-0 opacity-50"/>
                </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content class="w-[140px]">
                <DropdownMenu.Item onclick={() => handleSortChange('updated')}>
                    {sortLabels.updated}
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => handleSortChange('new')}>
                    {sortLabels.new}
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => handleSortChange('views')}>
                    {sortLabels.views}
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => handleSortChange('ratings')}>
                    {sortLabels.ratings}
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>

    <div
            class="bg-white dark:bg-gray-800 rounded-lg shadow-sm"
    >
        <div class="p-4 sm:p-6">
            {#if isLoading}
                <div class="text-center py-12">
                    <div
                            class="w-10 h-10 border-4 border-gray-200 border-t-blue-500 rounded-full animate-spin mx-auto mb-4"
                    ></div>
                    <p class="text-gray-600 dark:text-gray-400">Fetching series...</p>
                </div>
            {:else if error}
                <div class="text-center py-12">
                    <div
                            class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6"
                    >
                        <h3 class="text-lg font-semibold text-red-800 dark:text-red-200 mb-2">
                            An Error Occurred
                        </h3>
                        <p class="text-red-600 dark:text-red-400">{error}</p>
                    </div>
                </div>
            {:else if seriesResult && seriesResult.items.length > 0}
                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-6">
                    {#each seriesResult.items as series (series.id)}
                        <a href="/manga/{series.id}/{slugify(series.title, { lower: true, strict: true })}"
                           class="flex gap-4 group">
                            <div class="w-32 flex-shrink-0">
                                <img
                                        src={series.cover_image_url}
                                        alt={series.title}
                                        class="w-full h-auto object-cover rounded shadow-md aspect-[2/3]"
                                        loading="lazy"
                                />
                            </div>
                            <div class="flex flex-col gap-1 py-1">
                                <h3 class="text-base font-semibold leading-tight group-hover:text-blue-500 transition-colors line-clamp-2">
                                    {series.title}
                                </h3>
                                <p class="text-sm text-gray-600 dark:text-gray-300 line-clamp-1">
                                    {series.original_title}
                                </p>
                                <p class="text-sm text-gray-600 dark:text-gray-300 line-clamp-2">
                                    {series.authors.join(', ') || 'N/A'}
                                </p>
                                <div class="mt-auto pt-1 flex flex-row gap-20">
                                    {#if series.last_chapter_found_in_storage}
                                        <p class="text-sm font-semibold text-sky-500 dark:text-sky-400">
                                            Latest: {series.last_chapter_found_in_storage}
                                        </p>
                                    {/if}
                                    <p class="text-sm text-gray-700 dark:text-gray-200">
                                        {formatRelativeTime(series.updated_at)}
                                    </p>
                                </div>
                                <p class="text-sm text-gray-500 dark:text-gray-300 mt-1 line-clamp-2">
                                    {series.description}
                                </p>
                                <div class="mt-auto pt-1">
                                    <p class="text-sm text-gray-500 dark:text-gray-400 line-clamp-1">
                                        {series.categories.join(', ')}
                                    </p>
                                </div>
                            </div>
                        </a>
                    {/each}
                </div>

                <div class="mt-8 pt-4 border-t border-gray-200 dark:border-gray-700">
                    <p class="text-sm text-gray-600 dark:text-gray-400 text-center">
                        Showing {seriesResult.items.length} of {seriesResult.total_items} series
                    </p>
                </div>
            {:else}
                <div class="text-center py-12 px-4">
                    <div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-8 max-w-md mx-auto">
                        <h3 class="text-xl font-semibold text-gray-700 dark:text-gray-300 mb-2">
                            No Results Found
                        </h3>
                        <Button
                                onclick={resetFilters}
                                class="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md transition-colors"
                        >
                            <RotateCcw size={16}/>
                            Reset
                        </Button>
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>

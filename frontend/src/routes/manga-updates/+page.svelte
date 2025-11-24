<svelte:head>
    <title>Manga Updates</title>
    <meta name="description" content="Recently updated manga chapters"/>
</svelte:head>

<script lang="ts">
    import SeriesCarousel from '$lib/components/SeriesCarousel.svelte';
    import LatestUpdateSeries from "$lib/components/LatestUpdateSeries.svelte";
    import Button from '$lib/components/ui/button/button.svelte';

    type Manga = {
        id: number;
        title: string;
        cover_image_url: string;
        view_count?: number;
        authors?: string[];
    };

    let selectedPeriod = $state<'hour' | 'day' | 'week' | 'month'>('day');
    let mostViewed = $state<Manga[]>([]);
    let isLoadingMostViewed = $state(true);
    let newSeries = $state<Manga[]>([]);
    let isLoadingNewSeries = $state(true);

    const dynamicText = $derived(
        selectedPeriod === 'hour' ? 'This Hour' :
            selectedPeriod === 'day' ? 'Today' :
                selectedPeriod == 'week' ? 'This Week' :
                    'This Month'
    )

    $effect(() => {
        const fetchMostViewed = async () => {
            isLoadingMostViewed = true;
            try {
                const response = await fetch(`/api/series/most-viewed?period=${selectedPeriod}&limit=20`);
                if (!response.ok) {
                    throw new Error(`Failed to fetch most viewed manga`);
                }
                mostViewed = await response.json();
            } catch (error) {
                console.error('Error fetching data:', error);
                mostViewed = [];
            } finally {
                isLoadingMostViewed = false;
            }
        };
        fetchMostViewed();
    });

    $effect(() => {
        const fetchNewSeries = async () => {
            isLoadingNewSeries = true;
            try {
                const response = await fetch(`/api/series/new-series`);
                if (!response.ok) {
                    throw new Error(`Failed to fetch new manga series`);
                }
                newSeries = await response.json();
            } catch (error) {
                console.error('Error fetching data:', error);
                newSeries = [];
            } finally {
                isLoadingNewSeries = false;
            }
        }
        fetchNewSeries();
    });
</script>

<div class="w-full space-y-12">
    <header>
        <h1 class="text-2xl sm:text-3xl md:text-4xl font-bold mb-2">Recently Updated Series</h1>
        <p class="text-base sm:text-lg text-gray-500">
            New chapters are immediately updated on our website as soon as they are translated.
        </p>
    </header>

    <section class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
            <h2 class="flex flex-col font-bold leading-tight text-lg sm:flex-row sm:gap-1.5 sm:text-xl sm:leading-normal">
                <span>Most Viewed</span>
                <span>{dynamicText}</span>
            </h2>
            <div class="flex items-center gap-0.5 rounded-lg bg-gray-200 dark:bg-gray-700 p-0.5 sm:p-1">
                <Button
                        onclick={() => (selectedPeriod = 'hour')}
                        variant="ghost"
                        size="xs"
                        class="text-xs font-semibold rounded-md transition-colors sm:px-3 sm:text-sm {selectedPeriod === 'hour'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Hour
                </Button>
                <Button
                        onclick={() => (selectedPeriod = 'day')}
                        variant="ghost"
                        size="xs"
                        class="text-xs font-semibold rounded-md transition-colors sm:text-sm {selectedPeriod === 'day'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Today
                </Button>
                <Button
                        onclick={() => (selectedPeriod = 'week')}
                        variant="ghost"
                        size="xs"
                        class="text-xs font-semibold rounded-md transition-colors sm:text-sm {selectedPeriod === 'week'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Week
                </Button>
                <Button
                        onclick={() => (selectedPeriod = 'month')}
                        variant="ghost"
                        size="xs"
                        class="text-xs font-semibold rounded-md transition-colors sm:text-sm {selectedPeriod === 'month'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Month
                </Button>
            </div>
        </div>
        {#if isLoadingMostViewed}
            <p>Loading most viewed manga...</p> {:else}
            <SeriesCarousel manga={mostViewed}/>
        {/if}
    </section>

    <section class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
            <h2 class="text-lg font-bold sm:text-xl">New Series</h2>
            <a href="/browse?sort=new"
               class="px-3 py-2 text-sm font-semibold text-white bg-blue-600 rounded-sm shadow transition-colors hover:bg-blue-700">
                View More
            </a>
        </div>
        {#if isLoadingNewSeries}
            <p>Loading new series...</p> {:else}
            <SeriesCarousel manga={newSeries}/>
        {/if}
    </section>
    <LatestUpdateSeries/>
</div>
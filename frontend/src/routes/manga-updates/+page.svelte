<svelte:head>
    <title>Manga Updates</title>
    <meta name="description" content="Recently updated manga chapters"/>
</svelte:head>

<script lang="ts">
    import SeriesCarousel from '$lib/components/SeriesCarousel.svelte';

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

    const mostViewedTitleSeries = $derived(
        selectedPeriod === 'hour' ? 'Most Viewed This Hour' :
            selectedPeriod === 'day' ? 'Most Viewed Today' :
                selectedPeriod == 'week' ? 'Most Viewed This Week' :
                    'Most Viewed This Month'
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
        <h1 class="text-2xl sm:text-4xl font-bold mb-2">Recenly Updated Manga Chapters</h1>
        <p class="text-lg text-gray-500">
            New chapters are immediately updated on our website as soon as they are translated.
        </p>
    </header>

    <section class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
            <h2 class="text-xl font-bold">{mostViewedTitleSeries}</h2>
            <div class="flex items-center gap-2 rounded-lg bg-gray-200 dark:bg-gray-700 p-1">
                <button
                        onclick={() => (selectedPeriod = 'hour')}
                        class="px-3 py-1 text-sm font-semibold rounded-md transition-colors {selectedPeriod === 'hour'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Hour
                </button>
                <button
                        onclick={() => (selectedPeriod = 'day')}
                        class="px-3 py-1 text-sm font-semibold rounded-md transition-colors {selectedPeriod === 'day'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Today
                </button>
                <button
                        onclick={() => (selectedPeriod = 'week')}
                        class="px-3 py-1 text-sm font-semibold rounded-md transition-colors {selectedPeriod === 'week'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Week
                </button>
                <button
                        onclick={() => (selectedPeriod = 'month')}
                        class="px-3 py-1 text-sm font-semibold rounded-md transition-colors {selectedPeriod === 'month'
						? 'bg-blue-600 text-white shadow'
						: 'text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'}"
                >
                    Month
                </button>
            </div>
        </div>
        {#if isLoadingMostViewed}
            <p>Loading most viewed manga...</p> {:else}
            <SeriesCarousel manga={mostViewed}/>
        {/if}
    </section>

    <section class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
            <h2 class="text-xl font-bold">New</h2>
            <a href="/manga/new"
               class="px-4 py-2 text-sm font-semibold text-white bg-blue-600 rounded-lg shadow transition-colors hover:bg-blue-700">
                View More
            </a>
        </div>
        {#if isLoadingNewSeries}
            <p>Loading new series...</p> {:else}
            <SeriesCarousel manga={newSeries}/>
        {/if}
    </section>
</div>
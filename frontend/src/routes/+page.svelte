<svelte:head>
    <title>Test</title>
    <meta name="description" content="Svelte demo app"/>
</svelte:head>

<script lang="ts">
    import SeriesCarousel from '$lib/components/SeriesCarousel.svelte';

    const placeholderManga = Array(20).fill(null).map((_, index) => ({
        id: index + 1,
        title: `Manga Title ${index + 1}`,
    }));

    const mostViewedToday = placeholderManga.slice(0, 12);

    type Manga = {
        id: number;
        title: string;
        cover_image_url: string;
    }

    let newSeries = $state<Manga[]>([]);
    let isLoadingNewSeries = $state(true);

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
    <div class="new-manga-carousel-outer">
        <SeriesCarousel manga={newSeries}/>
    </div>
</div>
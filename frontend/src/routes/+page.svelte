<svelte:head>
    <title>Test</title>
    <meta name="description" content="Svelte demo app"/>
</svelte:head>

<script lang="ts">
    // Import komponen yang diperlukan
    import MangaCarousel from '$lib/components/MangaCarousel.svelte';

    const placeholderManga = Array(16).fill(null).map((_, index) => ({
        id: index + 1,
        title: `Manga Title ${index + 1}`,
    }));

    const mostViewedToday = placeholderManga.slice(0, 12);
    const newManga = placeholderManga.slice(6);
</script>

<div class="new-manga-carousel">
    <div class="new-manga-carousel-outer">
        <MangaCarousel manga="{mostViewedToday}" itemsPerPage={1}/>
    </div>
</div>

<div class="most-viewed-carousel">
    <div>
        <MangaCarousel manga="{newManga}" itemsPerPage={1}/>
    </div>
</div>

<style>
    div.new-manga-carousel {
        margin-bottom: 70px;
    }

    .new-manga-carousel {
        width: 100%;
        height: 200px;
    }

    .new-manga-carousel-outer {
        width: 100%;
        height: 200px;
    }

    .most-viewed-carousel {
        width: 100%;
        height: 200px;
    }

    .new-manga-carousel-outer {
        width: 100%;
        height: 200px;
        display: flex;
    }

</style>
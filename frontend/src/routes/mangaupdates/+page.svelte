<svelte:head>
    <title>Manga Updates</title>
    <meta name="description" content="Recently updated manga chapters"/>
</svelte:head>

<script>
    // Import komponen yang diperlukan
    import MangaList from '../MangaList.svelte';

    const placeholderManga = Array(16).fill(null).map((_, index) => ({
        id: index + 1,
        title: `Manga Title ${index + 1}`,
    }));

    const mostViewedToday = placeholderManga.slice(0, 12);
    const newManga = placeholderManga.slice(6);
</script>

<div class="manga-updates">
    <h1>Recenly Updated Manga Chapters</h1>

    <p>New chapters are immediately updated on our website as soon as they are translated.</p>

    <section>
        <h2>Most Viewed Today</h2>
        <MangaList manga="{mostViewedToday}" itemsPerPage={1} />
    </section>

    <section>
        <h2>New</h2>
        <MangaList manga="{newManga}" itemsPerPage={1} />
    </section>
</div>

<style>
    .manga-updates {
        max-width: 1000px;
        margin: 0 auto;
        padding: 10px;
        left: auto;
    }

    h1 {
        margin-bottom: 10px;
    }

    p {
        margin-bottom: 30px;
    }

    section {
        margin-bottom: 20px;
    }

    h2 {
        margin-bottom: 15px;
        font-size: 1.5rem;
    }
</style>
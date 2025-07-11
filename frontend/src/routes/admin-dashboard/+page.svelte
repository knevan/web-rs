<svelte:head>
    <title>Admin Dashboard</title>
</svelte:head>

<script>
    import AdminSidebar from "$lib/components/AdminSidebar.svelte";
    import AdminMangaTable from "$lib/components/AdminMangaTable.svelte";

    // Example data that would typically come from an API
    let mangas = $state([
        {
            id: 1,
            name: 'Epic Adventure Manga',
            author: 'Walaweh',
            totalChapters: 150,
            lastChapter: 'Ch. 150',
            lastUpdated: '2025-07-04',
            sourceUrls: 'https://example.com/manga/1'
        },
        {
            id: 2,
            name: 'Naga bima sakti',
            author: 'munaroh',
            totalChapters: 110,
            lastChapter: 'Ch. 110',
            lastUpdated: '2025-05-04',
            sourceUrls: 'https://example.com/manga/2'
        }
    ])
</script>

<div class="p-4 sm:p-3 md:p-8">
    <h1 class="text-2xl md:text-3xl font-bold text-gray-800 mb-6">Admin Dashboard</h1>

    <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <!-- Main Content : Manga List -->
        <div class="lg:col-span-3">
            <h2 class="text-xl font-semibold text-gray-700 mb-4"> List Manga</h2>
            <AdminMangaTable {mangas}/>
        </div>

        <!-- Sidebar : Admin Sidebar -->
        <div class="lg:col-span-1">
            <AdminSidebar/>
        </div>
    </div>
</div>
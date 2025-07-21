<script lang="ts">
    import image_test from "$lib/images/image_image.webp"
    import {page} from "$app/state";
    import {goto} from "$app/navigation";

    interface MangaSeries {
        id: number;
        title: string;
        description: string;
        cover_image_url: string;
        views_count: number;
        bookmarks_count: number;
        processing_status: string;
        updated_at: string;
    }

    interface MangaChapter {
        chapter_number: number;
        title: string;
        created_at: string;
    }

    interface MangaData {
        series: MangaSeries;
        chapters: MangaChapter[];
        authors: string[];
        categories: string[];
    }

    // MOCK DATA
    const userMockData = true;

    const mockMangaData = {
        series: {
            id: 99,
            title: "Test Manga",
            description: "This is a detailed placeholder description for testing the component's layout. It can be long to see how text wrapping works and to ensure the overall design remains consistent even with a large amount of text content.",
            cover_image_url: "image_test",
            views_count: 1234567,
            bookmarks_count: 89000,
            processing_status: "monitoring",
            updated_at: "2025-07-01T10:00:00Z",
        },
        chapters: [
            {chapter_number: 5.0, title: "The Rune of Reactivity", created_at: "2025-07-01T10:00:00Z"},
            {chapter_number: 4.0, title: "A New State", created_at: "2025-06-25T10:00:00Z"},
            {chapter_number: 3.0, title: "The Effect Awakens", created_at: "2025-06-18T10:00:00Z"},
            {chapter_number: 2.0, title: "Deriving a Conclusion", created_at: "2025-06-11T10:00:00Z"},
            {chapter_number: 1.0, title: "The Journey Begins", created_at: "2025-06-04T10:00:00Z"},
        ],
        authors: ["Joe Blow", "Blow Jobs"],
        categories: ["Action", "Fantasy", "Adventure", "Isekai", "Magic"]
    };

    // Props manga ID accepted from routing or parent component
    let {mangaId = null}: { mangaId: string | null } = $props();

    // State Management
    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let mangaData = $state<MangaData | null>(null);

    // Base URL for Manga API
    const API_BASE_URL = 'http://localhost:8000';

    const currentMangaId = $derived(mangaId || page?.params['manga_name']);
    const status = $derived(mangaData?.series?.processing_status === 'monitoring' ? 'Ongoing' : 'Completed');
    const statusClass = $derived(status === 'Ongoing' ? 'text-green-400' : 'text-gray-400');
    const chaptersCount = $derived(mangaData?.chapters?.length || 0);
    const sortedChapters = $derived(mangaData?.chapters ? [...mangaData.chapters].sort((a, b) => b.chapter_number - a.chapter_number) : []);

    // use $effect hook to fetch manga data
    // $effect running first time when component mounts
    $effect(() => {
        async function loadData() {
            isLoading = true;
            error = null;
            mangaData = null;

            await new Promise(resolve => setTimeout(resolve, 1000));

            if (userMockData) {
                // Mode Mock
                console.log('Using Mock Data');
                mangaData = mockMangaData;
                isLoading = false;
                return;
            } else {
                if (!currentMangaId) {
                    error = "No manga ID provided";
                    isLoading = false;
                    return;
                }

                try {
                    console.log(`Fetching REAL data for ID: ${currentMangaId}`);
                    const response = await fetch(`${API_BASE_URL}/api/manga/${currentMangaId}`);

                    if (!response.ok) {
                        error = `Failed to fetch manga: ${response.status} ${response.statusText}`;
                        return;
                    }
                    mangaData = await response.json();

                } catch (err) {
                    console.error('Error fetching manga data:', err);
                    error = `Gagal memuat data manga: ${err instanceof Error ? err.message : 'Unknown Error'}`;
                } finally {
                    isLoading = false;
                }
            }
        }

        loadData();
    });

    // --- Utility dan Navigation Handlers (Tidak perlu diubah) ---
    // Fungsi-fungsi ini adalah JavaScript biasa dan tidak terpengaruh oleh Runes.
    function formatCount(num: number): string {
        if (!num) return '0';
        if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
        if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
        return num.toString();
    }

    function timeAgo(dateString: string): string {
        if (!dateString) return 'Unknown';
        const date = new Date(dateString);
        const now = new Date();
        const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        let interval = seconds / 31536000;
        if (interval > 1) return Math.floor(interval) + " years ago";
        interval = seconds / 2592000;
        if (interval > 1) return Math.floor(interval) + " months ago";
        interval = seconds / 86400;
        if (interval > 1) return Math.floor(interval) + " days ago";
        interval = seconds / 3600;
        if (interval > 1) return Math.floor(interval) + " hours ago";
        interval = seconds / 60;
        if (interval > 1) return Math.floor(interval) + " minutes ago";
        return Math.floor(seconds) + " seconds ago";
    }

    function handleChapterClick(chapterNumber: number) {
        if (currentMangaId && chapterNumber) {
            goto(`/manga/${currentMangaId}/chapter/${chapterNumber}`);
        }
    }

    function handleCategoryClick(category: string) {
        goto(`/browse?category=${encodeURIComponent(category)}`);
    }

    function handleBackToList() {
        goto('/manga');
    }
</script>

<svelte:head>
    <title>{mangaData?.series?.title || 'Loading...'} - Manga Reader</title>
    <meta name="description" content={mangaData?.series?.description || 'Manga details'}/>
</svelte:head>

<div class="min-h-screen bg-dark-primary text-gray-100">
    <!-- Header dengan back button -->
    <div class="sticky top-0 z-10 bg-dark-primary/90 backdrop-blur-sm border-b border-gray-800">
        <div class="max-w-6xl mx-auto px-4 py-3">
            <button onclick={handleBackToList}
                    class="flex items-center gap-2 text-gray-400 hover:text-white transition-colors">
                <i class="fas fa-arrow-left"></i>
                Back to Manga List
            </button>
        </div>
    </div>

    <div class="max-w-6xl mx-auto p-4 md:p-8">
        <!-- Loading State -->
        {#if isLoading}
            <div class="flex flex-col justify-center items-center h-96">
                <div class="loader mb-4"></div>
                <p class="text-gray-400">Loading manga data...</p>
            </div>
        {:else if error}
            <!-- Error State -->
            <div class="text-center py-12">
                <div class="text-red-400 mb-4">
                    <i class="fas fa-exclamation-triangle text-4xl mb-2"></i>
                    <h2 class="text-2xl font-bold">Error Loading Manga</h2>
                </div>
                <p class="text-gray-400 mb-4">{error}</p>
                <button onclick={() => {
                    isLoading = true;
                    error = null;
                }}
                        class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-lg transition-colors">
                    Try Again
                </button>
            </div>
        {:else if mangaData}
            <!-- Main Content -->
            <div class="space-y-8">
                <!-- Manga Info Section -->
                <div class="main-container p-6 md:p-8">
                    <div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 gap-8">
                        <!-- Cover Image -->
                        <div class="md:col-span-1">
                            <div class="sticky top-24">
                                <img src={mangaData.series.cover_image_url} alt={mangaData.series.title}
                                     class="w-full h-auto object-cover rounded-lg shadow-lg" loading="lazy"/>
                            </div>
                        </div>

                        <!-- Manga Details -->
                        <div class="md:col-span-2 lg:col-span-3 space-y-6">
                            <!-- Title dan Author -->
                            <div class="space-y-2">
                                <h1 class="text-3xl md:text-4xl font-bold text-white leading-tight">
                                    {mangaData.series.title}
                                </h1>
                                <p class="text-gray-400">
                                    Author:
                                    <span class="text-blue-300">
                                        {mangaData.authors?.join(', ') || 'Unknown'}
                                    </span>
                                </p>
                            </div>

                            <!-- Stats Grid -->
                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 p-4 bg-dark-secondary rounded-lg">
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-white flex items-center justify-center gap-2">
                                        <i class="fas fa-book-open text-blue-400"></i>
                                        {chaptersCount}
                                    </div>
                                    <p class="text-sm text-gray-400">Chapters</p>
                                </div>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-white flex items-center justify-center gap-2">
                                        <i class="fas fa-eye text-green-400"></i>
                                        {formatCount(mangaData.series.views_count)}
                                    </div>
                                    <p class="text-sm text-gray-400">Views</p>
                                </div>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-white flex items-center justify-center gap-2">
                                        <i class="fas fa-bookmark text-yellow-400"></i>
                                        {formatCount(mangaData.series.bookmarks_count)}
                                    </div>
                                    <p class="text-sm text-gray-400">Bookmarked</p>
                                </div>
                                <div class="text-center">
                                    <div class="text-2xl font-bold {statusClass}">
                                        {status}
                                    </div>
                                    <p class="text-sm text-gray-400">Status</p>
                                </div>
                            </div>

                            <!-- Categories -->
                            {#if mangaData.categories?.length > 0}
                                <div class="space-y-3">
                                    <h3 class="text-lg font-semibold text-white">Categories</h3>
                                    <div class="flex flex-wrap gap-2">
                                        {#each mangaData.categories as category}
                                            <button onclick={() => handleCategoryClick(category)}
                                                    class="category-badge px-3 py-1 text-sm font-medium rounded-full">
                                                {category}
                                            </button>
                                        {/each}
                                    </div>
                                </div>
                            {/if}

                            <!-- Last Update -->
                            <div class="text-gray-400 text-sm">
                                Last Update:
                                <span class="text-gray-300">
                                    {timeAgo(mangaData.series.updated_at)}
                                </span>
                            </div>

                            <!-- Description -->
                            {#if mangaData.series.description}
                                <div class="space-y-3">
                                    <h3 class="text-lg font-semibold text-white">Description</h3>
                                    <p class="text-gray-300 leading-relaxed">
                                        {mangaData.series.description}
                                    </p>
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>

                <!-- Chapter List Section -->
                {#if sortedChapters.length > 0}
                    <div class="main-container p-6">
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-2xl font-bold text-white">
                                Chapters ({chaptersCount})
                            </h2>
                            <div class="text-sm text-gray-400">
                                Latest: Chapter {sortedChapters[0]?.chapter_number}
                            </div>
                        </div>

                        <div class="grid gap-3">
                            {#each sortedChapters as chapter}
                                <button onclick={() => handleChapterClick(chapter.chapter_number)}
                                        class="chapter-item p-4 rounded-lg text-left">
                                    <span class="flex justify-between items-center">
                                        <span class="flex-1">
                                            <span class="font-semibold text-white mb-1">
                                                Chapter {chapter.chapter_number}
                                                {#if chapter.title}
                                                    <span class="text-gray-300">- {chapter.title}</span>
                                                {/if}
                                            </span>
                                            <span class="text-sm text-gray-400">
                                                {new Date(chapter.created_at).toLocaleDateString('id-ID', {
                                                    year: 'numeric',
                                                    month: 'long',
                                                    day: 'numeric'
                                                })}
                                            </span>
                                        </span>
                                        <span class="text-blue-400">
                                            <i class="fas fa-chevron-right"></i>
                                        </span>
                                    </span>
                                </button>
                            {/each}
                        </div>
                    </div>
                {:else}
                    <div class="text-center py-12">
                        <i class="fas fa-book-open text-4xl text-gray-600 mb-4"></i>
                        <p class="text-gray-400">No chapters available</p>
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    :global(body) {
        --dark-primary: #1a1a2e;
        --dark-secondary: #16213e;
        background-color: var(--dark-primary);
        color: #e0e0e0;
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }

    .bg-dark-primary {
        background-color: var(--dark-primary);
    }

    .bg-dark-secondary {
        background-color: var(--dark-secondary);
    }

    .main-container {
        background-color: var(--dark-secondary);
        border-radius: 1rem;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
    }

    .category-badge {
        background-color: #0f3460;
        color: #c0c0ff;
        transition: all 0.3s ease;
    }

    .category-badge:hover {
        background-color: #5372f0;
        color: white;
        transform: translateY(-2px);
    }

    .chapter-item {
        background-color: #1e293b;
        border-left: 4px solid #4a5568;
        transition: all 0.3s ease;
    }

    .chapter-item:hover {
        background-color: #334155;
        border-left-color: #5372f0;
        transform: translateX(8px);
    }

    .loader {
        border: 4px solid #f3f3f3;
        border-top: 4px solid #5372f0;
        border-radius: 50%;
        width: 50px;
        height: 50px;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }

    @media (max-width: 768px) {
        .main-container {
            margin: 0 -1rem;
            border-radius: 0;
        }
    }
</style>
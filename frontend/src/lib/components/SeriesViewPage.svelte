<script lang="ts">
    import {goto} from "$app/navigation";
    import slugify from "slugify";
    import {auth} from "$lib/store/auth";
    import {page} from "$app/state";
    import {Button} from "$lib/components/ui/button";
    import {toast} from "svelte-sonner";

    interface MangaSeries {
        id: number;
        title: string;
        original_title: string;
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
        authors: string[];
        categoryTags: string[];
        chapters: MangaChapter[];
    }

    // Props manga ID accepted from routing or parent Layout
    let {mangaId = null}: { mangaId: string | null } = $props();

    // State Management
    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let mangaData = $state<MangaData | null>(null);
    let isBookmarked = $state(false);

    const authState = $derived($auth);

    // Base URL for Manga API
    //const API_BASE_URL = 'http://localhost:8000';

    const currentMangaId = $derived(mangaId);
    const seriesSlug = $derived(slugify(mangaData?.series?.title || '', {lower: true}));
    const chaptersCount = $derived(mangaData?.chapters?.length || 0);
    const sortedChapters = $derived(mangaData?.chapters ? [...mangaData.chapters].sort((a, b) => b.chapter_number - a.chapter_number) : []);
    const firstChapter = $derived(mangaData?.chapters && mangaData.chapters.length > 0 ? [...mangaData.chapters].sort(
        (a, b) => a.chapter_number - b.chapter_number)[0] : null);

    const displayStatus = $derived(() => {
        const rawStatus = mangaData?.series?.processing_status?.toLowerCase() || '';

        if (new Set(['pending', 'processing', 'available', 'ongoing']).has(rawStatus)) {
            return {label: 'Ongoing', className: 'text-green-400'};
        }

        if (rawStatus === 'completed') {
            return {label: 'Completed', className: 'text-gray-400'};
        }

        if (rawStatus === 'hiatus') {
            return {label: 'Hiatus', className: 'text-yellow-400'};
        }

        if (rawStatus === 'discontinued') {
            return {label: 'Discontinued', className: 'text-red-400'};
        }

        return {label: 'Ongoing', className: 'text-green-400'};
    });

    // use $effect hook to fetch manga data
    // $effect running first time when Layout mounts
    $effect(() => {
        async function loadData() {
            isLoading = true;
            error = null;
            mangaData = null;

            //await new Promise(resolve => setTimeout(resolve, 100));

            if (!currentMangaId) {
                error = "No manga ID provided";
                isLoading = false;
                return;
            }

            try {
                console.log(`Fetching REAL data for ID: ${currentMangaId}`);
                const response = await fetch(`/api/series/details/${currentMangaId}`);

                if (!response.ok) {
                    error = `Failed to fetch manga: ${response.status} ${response.statusText}`;
                    return;
                }
                mangaData = await response.json();

                if (authState.isAuthenticated) {
                    try {
                        const bookmarkStatusResponse = await fetch(`/api/series/${currentMangaId}/bookmark/status`);
                        if (bookmarkStatusResponse.ok) {
                            const data = await bookmarkStatusResponse.json();
                            isBookmarked = data.isBookmarked;
                        } else {
                            console.warn("Could not fetch bookmark status");
                        }
                    } catch (bookmarkError) {
                        console.error("Error fetch bookmark status", bookmarkError);
                    }
                }
            } catch (err) {
                console.error('Error fetching manga data:', err);
                error = `Failed to load: ${err instanceof Error ? err.message : 'Unknown Error'}`;
            } finally {
                isLoading = false;
            }
        }

        loadData();
    });

    // Fungsi-fungsi ini adalah JavaScript biasa dan tidak terpengaruh oleh Runes.
    function formatCount(num: number): string {
        if (!num) return '0';
        if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
        if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
        return num.toString();
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

    function handleChapterClick(chapterNumber: number) {
        if (currentMangaId && chapterNumber) {
            goto(`/manga/${currentMangaId}/${seriesSlug}/read-chapter/${chapterNumber}`);
        }
    }

    function handleCategoryClick(category: string) {
        goto(`/browse?category=${encodeURIComponent(category)}`);
    }

    async function handleBookmarkSaveClick() {
        if (!mangaData) return;

        const newBookmarkStatus = !isBookmarked;
        const originalBookmarkStatus = isBookmarked;
        const originalCount = mangaData.series.bookmarks_count;

        isBookmarked = newBookmarkStatus;
        mangaData.series.bookmarks_count += newBookmarkStatus ? 1 : -1;

        const updateBookmarkStatus = async () => {
            const method = newBookmarkStatus ? "POST" : "DELETE";
            const response = await fetch(`/api/series/${currentMangaId}/bookmark`, {
                method: method,
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({message: 'Server error'}));
                throw new Error(errorData.message || `Failed to update bookmark.`);
            }
            return newBookmarkStatus;
        };

        isBookmarked = originalBookmarkStatus;
        mangaData.series.bookmarks_count = originalCount;

        toast.promise(updateBookmarkStatus(), {
            position: "top-center",
            richColors: true,
            closeButton: false,
            duration: 1500,
            loading: `Add series to Bookmark Library...`,
            success: (isNowBookmarked) => {
                isBookmarked = isNowBookmarked;
                if (mangaData) {
                    mangaData.series.bookmarks_count += isNowBookmarked ? 1 : -1;
                }
                return isNowBookmarked ? 'Series added to your library' : 'Series removed from your library';
            },
            error: (err) => {
                return err instanceof Error ? err.message : `Failed to update bookmark.`;
            }
        });
    }

    function handleLoginClick() {
        const redirectTo = page.url.pathname;
        goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
    }

    /*function handleBackToList() {
        goto('/manga');
    }*/
</script>

<svelte:head>
    <title>{mangaData?.series?.title || 'Loading...'} - Manga Reader</title>
    <meta name="description" content={mangaData?.series?.description || 'Manga details'}/>
</svelte:head>

<div class="min-h-screen bg-dark-primary text-gray-100">
    <!-- Header
    <div class="sticky top-0 z-10 bg-dark-primary/90 backdrop-blur-sm border-b border-gray-800">
        <div class="max-w-6xl mx-auto px-4 py-3">
            <button onclick={handleBackToList}
                    class="flex items-center gap-2 text-gray-400 hover:text-white transition-colors">
                Back to Manga List
            </button>
        </div>
    </div> -->

    <div class="max-w-6xl mx-auto p-4 md:p-5">
        <!-- Loading State -->
        {#if isLoading}
            <div class="flex flex-col justify-center items-center h-96">
                <div class="w-12 h-12 border-4 border-gray-300 border-t-blue-600 rounded-full animate-spin"></div>
                <p class="text-gray-400">Loading manga data...</p>
            </div>
        {:else if error}
            <!-- Error State -->
            <div class="text-center py-12">
                <div class="text-red-400 mb-4">

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
            <div class="space-y-1">
                <div class="bg-[#16213e] shadow-2xl rounded-none md:rounded-2xl p-6 md:p-8 flex flex-col gap-8">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-7">

                        <div class="md:col-span-1 flex justify-center">
                            <img src={mangaData.series.cover_image_url} alt={mangaData.series.title}
                                 class="w-full h-auto object-cover rounded-lg shadow-lg max-w-[200px] md:max-w-full"
                                 style="max-height: 300px;"
                                 loading="lazy"/>
                        </div>

                        <div class="md:col-span-2 flex flex-col space-y-4">
                            <div class="space-y-2">
                                <h1 class="text-xl md:text-xl font-medium text-white leading-tight">
                                    {mangaData.series.title}
                                </h1>
                                <p class="text-sm md:text-sm text-gray-400 -mt-1">
                                    {mangaData.series.original_title}
                                </p>
                                <p class="text-gray-400 text-sm">
                                    Author:
                                    <span class="text-blue-300 text-sm">
                                        {mangaData.authors?.join(', ') || 'Unknown'}
                                    </span>
                                </p>
                            </div>

                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-x-1 p-2 bg-dark-secondary rounded-lg">
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-400">Chapters</p>
                                    <div class="text-xl font-bold text-white flex items-center justify-center gap-2">

                                        {chaptersCount}
                                    </div>
                                </div>
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-400">Views</p>
                                    <div class="text-xl font-bold text-white flex items-center justify-center gap-2">

                                        {formatCount(mangaData.series.views_count)}
                                    </div>
                                </div>
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-400">Bookmarked</p>
                                    <div class="text-xl font-bold text-white flex items-center justify-center gap-2">

                                        {formatCount(mangaData.series.bookmarks_count)}
                                    </div>
                                </div>
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-400">Status</p>
                                    <div class="text-sm font-bold {displayStatus().className}">
                                        {displayStatus().label}
                                    </div>
                                </div>
                            </div>

                            {#if mangaData.categoryTags?.length > 0}
                                <div class="space-y-3">
                                    <div class="flex flex-wrap gap-2">
                                        {#each mangaData.categoryTags as category}
                                            <button onclick={() => handleCategoryClick(category)}
                                                    class="px-3 py-1 text-sm font-medium rounded-full bg-[#0f3460] text-[#c0c0ff] transition-all duration-300 ease-in-out hover:bg-blue-600 hover:text-white hover:-translate-y-0.5">
                                                {category}
                                            </button>
                                        {/each}
                                    </div>
                                </div>
                            {/if}

                            <div class="text-gray-400 text-sm">
                                Last Update:
                                <span class="text-gray-300">
                                    {formatRelativeTime(mangaData.series.updated_at)}
                                </span>
                            </div>

                            <div class="flex items-center gap-4 pt-2">
                                {#if firstChapter}
                                    <Button onclick={() => handleChapterClick(firstChapter.chapter_number)} size="lg"
                                            class="flex-1 text-center cursor-pointer text-lg bg-blue-600 hover:bg-blue-700 text-white font-semibold py-6 px-5 rounded-lg transition-colors duration-300 shadow-md"
                                    >
                                        Read Chapter {firstChapter.chapter_number}
                                    </Button>
                                {/if}

                                {#if authState.isAuthenticated}
                                    <Button onclick={handleBookmarkSaveClick} size="lg"
                                            class={[
                                    'flex-1 text-center cursor-pointer border border-gray-400 bg-white/10 backdrop-blur-sm text-gray-200 hover:bg-white/20 font-semibold py-6 px-5 rounded-lg transition-colors duration-300 flex items-center justify-center gap-2',
                                    isBookmarked && '!bg-yellow-500 hover:!bg-yellow-600 !border-yellow-500'
                                ]}
                                    >
                                        <span>{isBookmarked ? 'Bookmarked' : 'Bookmark'}</span>
                                    </Button>
                                {:else}
                                    <Button onclick={handleLoginClick} size="lg"
                                            class="flex-1 text-center cursor-pointer border border-gray-400 bg-white/10 backdrop-blur-sm text-gray-200 hover:bg-white/20 font-semibold py-6 px-5 rounded-lg transition-colors duration-300 flex items-center justify-center gap-2">
                                        <span class="text-lg">LOGIN</span>
                                    </Button>
                                {/if}
                            </div>
                        </div>
                    </div>


                    {#if mangaData.series.description}
                        <div class="space-y-3 pt-5 border-t border-gray-700/50">
                            <h3 class="text-md font-semibold text-white">Description</h3>
                            <p class="text-gray-300 leading-relaxed">
                                {mangaData.series.description}
                            </p>
                        </div>
                    {/if}
                </div>

                <!-- Chapter List Section -->
                {#if sortedChapters.length > 0}
                    <div class="bg-[#16213e] shadow-2xl rounded-none md:rounded-2xl p-6 md:p-8">
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-2xl font-bold text-white">
                                Chapters ({chaptersCount})
                            </h2>
                            <div class="text-sm text-gray-400">
                                Latest: Chapter {sortedChapters[0]?.chapter_number}
                            </div>
                        </div>

                        <div class="chapter-list-container max-h-[42rem] overflow-y-auto pr-2">
                            <div class="grid gap-3 grid-cols-1 md:grid-cols-2">
                                {#each sortedChapters as chapter}
                                    <button onclick={() => handleChapterClick(chapter.chapter_number)}
                                            class="p-4 rounded-lg text-left cursor-pointer bg-[#1e293b] border-l-4 border-gray-600 transition-all duration-300 ease-in-out hover:bg-gray-700 hover:border-blue-500 hover:translate-x-2">
                                    <span class="flex justify-between items-center">
                                        <span class="flex-1">
                                            <span class="font-semibold text-white mb-1">
                                                <!--{chapter.chapter_number}-->
                                                {#if chapter.title}
                                                    <span class="text-gray-300">{chapter.title}</span>
                                                {/if}
                                            </span>
                                            <span class="text-sm text-gray-400">
                                                {formatRelativeTime(chapter.created_at)}
                                            </span>
                                        </span>
                                        <span class="text-blue-400">

                                        </span>
                                    </span>
                                    </button>
                                {/each}
                            </div>
                        </div>
                    </div>
                {:else}
                    <div class="text-center py-12">
                        <p class="text-gray-400">No chapters available</p>
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    .chapter-list-container::-webkit-scrollbar {
        width: 8px;
    }

    .chapter-list-container::-webkit-scrollbar-track {
        background-color: transparent;
    }

    .chapter-list-container::-webkit-scrollbar-thumb {
        background-color: #4a5568;
        border-radius: 10px;
        border: 2px solid #16213e;
    }

    .chapter-list-container::-webkit-scrollbar-thumb:hover {
        background-color: #718096;
    }
</style>
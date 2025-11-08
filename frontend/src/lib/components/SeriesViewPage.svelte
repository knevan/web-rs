<script lang="ts">
    import {goto} from "$app/navigation";
    import slugify from "slugify";
    import {apiFetch, auth} from "$lib/store/auth";
    import {page} from "$app/state";
    import {Button} from "$lib/components/ui/button";
    import {toast} from "svelte-sonner";
    import {Star} from "@lucide/svelte";
    import {Badge} from "$lib/components/ui/badge";
    import CommentSystem from "$lib/components/comments/CommentSystem.svelte";

    interface MangaSeries {
        id: number;
        title: string;
        original_title: string;
        description: string;
        cover_image_url: string;
        views_count: number;
        bookmarks_count: number;
        total_rating_score: number;
        total_ratings_count: number;
        processing_status: string;
        updated_at: string;
    }

    interface MangaChapter {
        chapter_number: number;
        title: string;
        created_at: string;
    }

    interface CategoryTag {
        id: number;
        name: string;
    }

    interface MangaData {
        series: MangaSeries;
        authors: string[];
        categoryTags: CategoryTag[];
        chapters: MangaChapter[];
    }

    // Props manga ID accepted from routing or parent Layout
    let {mangaId = null}: { mangaId: string | null } = $props();

    // State Management
    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let mangaData = $state<MangaData | null>(null);
    let isBookmarked = $state(false);
    let isDescriptionExpanded = $state(false);
    let descriptionElement = $state<HTMLParagraphElement | null>(null);
    let isDescriptionOverflowing = $state(false);
    let hoveredRating = $state(0);
    let userRating = $state(0);

    const authState = $derived($auth);
    const MAX_DESCRIPTION_LINES = 5;
    const currentMangaId = $derived(mangaId);
    const seriesSlug = $derived(slugify(mangaData?.series?.title || '', {lower: true}));
    const chaptersCount = $derived(mangaData?.chapters?.length || 0);
    const sortedChapters = $derived(mangaData?.chapters ? [...mangaData.chapters].sort((a, b) => b.chapter_number - a.chapter_number) : []);
    const firstChapter = $derived(mangaData?.chapters && mangaData.chapters.length > 0 ? [...mangaData.chapters].sort(
        (a, b) => a.chapter_number - b.chapter_number)[0] : null);

    const displayStatus = $derived(() => {
        const rawStatus = mangaData?.series?.processing_status?.toLowerCase() || '';

        if (new Set(['pending', 'processing', 'available', 'ongoing']).has(rawStatus)) {
            return {label: 'Ongoing', className: 'text-green-500'};
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

    const averageRating = $derived(
        mangaData && mangaData.series.total_ratings_count > 0
            ? (mangaData.series.total_rating_score / mangaData.series.total_ratings_count) : 0
    );

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
                        const bookmarkStatusResponse = await apiFetch(`/api/series/${currentMangaId}/bookmark/status`);
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

    $effect(() => {
        const element = descriptionElement;
        if (!element) return;

        const style = window.getComputedStyle(element);
        const lineHeight = parseFloat(style.lineHeight);
        const maxHeight = lineHeight * MAX_DESCRIPTION_LINES;

        if (element.scrollHeight > maxHeight) {
            isDescriptionOverflowing = true;
        } else {
            isDescriptionOverflowing = false;
        }
    })

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

    function handleCategoryClick(tag: CategoryTag) {
        goto(`/browse?category=${encodeURIComponent(tag.name)}`);
    }

    async function handleRatingClick(rating: number) {
        if (!authState.isAuthenticated) {
            toast.warning('Please log in to rate this series', {
                position: "top-center",
                closeButton: false,
                duration: 3000,
                action: {
                    label: 'Login',
                    onClick: () => handleLoginClick(),
                },
            });
            return;
        }

        if (!currentMangaId) return;
        userRating = rating;
        try {
            const response = await fetch(`/api/series/${currentMangaId}/rate`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({rating: rating}),
            });
            const result = await response.json();
            if (!response.ok) {
                throw new Error(result.error || 'Failed to submit rating');
            }

            if (mangaData) {
                mangaData.series.total_rating_score = result.new_total_score;
                mangaData.series.total_ratings_count = result.new_total_count;
            }
            toast.success(`You rated this series ${rating} stars!`, {
                position: "top-center",
                closeButton: false,
                duration: 1500,
            });
        } catch (err) {
            userRating = 0;
            toast.error(err instanceof Error ? err.message : 'An unknown error occurred', {
                position: "top-center",
                closeButton: false,
                duration: 3000,
            });
        }
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
            const response = await apiFetch(`/api/series/${currentMangaId}/bookmark`, {
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
</script>

<svelte:head>
    <title>{mangaData?.series?.title || 'Loading...'} - Manga Reader</title>
    <meta name="description" content={mangaData?.series?.description || 'Manga details'}/>
</svelte:head>

<div class="min-h-screen bg-dark-primary text-gray-100">
    <div class="max-w-6xl mx-auto md:p-5">
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
            </div>
        {:else if mangaData}
            <!-- Main Content -->
            <div>
                <div class="bg-gray-200/0 shadow-sm rounded-none md:rounded-sm p-4 md:p-8 flex flex-col gap-8">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-7">
                        <div class="md:col-span-1 flex justify-center">
                            <img src={mangaData.series.cover_image_url} alt={mangaData.series.title}
                                 class="w-full h-auto object-cover rounded-lg shadow-lg max-w-[200px] md:max-w-full aspect-[2/3]"
                                 style="max-height: 400px;" loading="lazy"/>
                        </div>

                        <div class="md:col-span-2 flex flex-col space-y-4">
                            <div class="space-y-2">
                                <h1 class="text-xl md:text-xl font-medium text-gray-800 dark:text-gray-200 leading-tight">
                                    {mangaData.series.title}
                                </h1>
                                <p class="text-sm md:text-sm text-gray-800 dark:text-gray-200 -mt-1">
                                    {mangaData.series.original_title}
                                </p>
                                <p class="text-gray-800 dark:text-gray-200 text-sm">
                                    Author:
                                    <span class="text-blue-700 text-sm">
                                        {mangaData.authors?.join(', ') || 'Unknown'}
                                    </span>
                                </p>
                            </div>
                            <div class="flex items-center gap-2 md:gap-3">
                                <div class="flex items-center">
                                    {#each {length: 5} as _, i}
                                        {@const starValue = i + 1}
                                        <Star class="
                                            {averageRating >= starValue ? 'text-yellow-400' : 'text-gray-500'}
                                            {averageRating >= (starValue - 0.5) && averageRating < starValue ? 'text-yellow-400 half-star' : ''}
                                            h-4 w-4 md:h-6 md:w-6 stroke-1 fill-current
                                        "/>
                                    {/each}
                                </div>
                                <div class="font-bold text-base md:text-lg text-gray-800 dark:text-gray-200">
                                    {averageRating.toFixed(1)}
                                </div>
                                <div class="text-gray-800 dark:text-gray-200 text-sm md:text-lg">
                                    ({formatCount(mangaData.series.total_ratings_count)})
                                </div>
                            </div>

                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-x-1 p-2 bg-dark-secondary rounded-lg">
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-800 dark:text-gray-200">Chapters</p>
                                    <div class="text-xl font-bold text-gray-800 dark:text-gray-200 flex items-center justify-center gap-2">
                                        {chaptersCount}
                                    </div>
                                </div>
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-800 dark:text-gray-200">Views</p>
                                    <div class="text-xl font-bold text-gray-800 dark:text-gray-200 flex items-center justify-center gap-2">
                                        {formatCount(mangaData.series.views_count)}
                                    </div>
                                </div>
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-800 dark:text-gray-200">Bookmarked</p>
                                    <div class="text-xl font-bold text-gray-800 dark:text-gray-200 flex items-center justify-center gap-2">
                                        {formatCount(mangaData.series.bookmarks_count)}
                                    </div>
                                </div>
                                <div class="text-center flex flex-col items-center gap-x-1">
                                    <p class="text-md text-gray-800 dark:text-gray-200">Status</p>
                                    <div class="text-sm font-bold text-gray-800 dark:text-gray-200 {displayStatus().className}">
                                        {displayStatus().label}
                                    </div>
                                </div>
                            </div>

                            <div class="text-gray-800 dark:text-gray-200 text-sm">
                                Last Update:
                                <span class="text-gray-800 dark:text-gray-200">
                                    {formatRelativeTime(mangaData.series.updated_at)}
                                </span>
                            </div>

                            <div class="flex items-center gap-2 pt-2">
                                {#if firstChapter}
                                    <Button onclick={() => handleChapterClick(firstChapter.chapter_number)} size="lg"
                                            class="w-1/2 flex text-center cursor-pointer border border-transparent bg-blue-600 hover:bg-blue-700 text-white font-semibold py-6 px-5 rounded-lg transition-colors duration-300 shadow-md"
                                    >
                                        <span class="text-base md:text-lg ">
											Read Chapter {firstChapter.chapter_number}
										</span>
                                    </Button>
                                {/if}

                                {#if authState.isAuthenticated}
                                    <Button onclick={handleBookmarkSaveClick} size="lg"
                                            class={[
                                                'w-1/2 text-center cursor-pointer border border-gray-400 bg-white/10 backdrop-blur-sm text-gray-800 dark:text-gray-200 hover:bg-white/20 font-semibold py-6 px-5 rounded-lg transition-colors duration-300 flex items-center justify-center gap-2',
                                                isBookmarked && '!bg-yellow-500 hover:!bg-yellow-600 !border-yellow-500'
                                            ]}
                                    >
                                        <span class="text-base md:text-lg">
											{isBookmarked ? 'Bookmarked' : 'Bookmark'}
										</span>
                                    </Button>
                                {:else}
                                    <Button onclick={handleLoginClick} size="lg"
                                            variant="outline"
                                            class="w-1/2 text-center cursor-pointer border border-gray-400 bg-gray-700/20 dark:bg-white/10 backdrop-blur-sm text-gray-800 dark:text-gray-200 hover:bg-white/20 font-semibold py-6 px-5 rounded-lg transition-colors duration-300 flex items-center justify-center gap-2">
                                        <span class="text-base md:text-lg">LOGIN</span>
                                    </Button>
                                {/if}
                            </div>
                            {#if mangaData.categoryTags?.length > 0}
                                <div class="space-y-3">
                                    <h3 class="text-gray-800 dark:text-gray-200 font-semibold text-xl mt-2 text-center md:text-left">
                                        Categories
                                    </h3>
                                    <div class="flex flex-wrap gap-2 mt-4">
                                        {#each mangaData.categoryTags as tag (tag.id)}
                                            <Badge onclick={() => handleCategoryClick(tag)}
                                                   role="button"
                                                   class="px-3 py-1 text-sm font-medium rounded-full bg-[#0f3460] text-white text-center transition-all duration-300 ease-in-out hover:bg-blue-600 hover:text-white hover:-translate-y-0.5">
                                                {tag.name}
                                            </Badge>
                                        {/each}
                                    </div>
                                </div>
                            {/if}
                        </div>
                    </div>
                    {#if mangaData.series.description}
                        <div class="space-y-2 pt-4 border-t border-gray-700/40">
                            <h3 class="text-md font-bold text-gray-800 dark:text-gray-200">Description</h3>
                            <p bind:this={descriptionElement}
                               class={[
                                'text-gray-800 dark:text-gray-200 leading-relaxed transition-all duration-300',
                                (isDescriptionOverflowing && !isDescriptionExpanded) && 'line-clamp-4'
                               ].filter(Boolean).join(' ')}
                            >
                                {mangaData.series.description}
                            </p>
                            {#if isDescriptionOverflowing}
                                <div class="flex justify-end">
                                    <Button onclick={() => isDescriptionExpanded = !isDescriptionExpanded}
                                            class="text-blue-400 hover:text-blue-300 font-semibold text-sm">
                                        {isDescriptionExpanded ? 'Show Less' : 'Read More'}
                                    </Button>
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>

                <!-- Chapter List Section -->
                {#if sortedChapters.length > 0}
                    <div class="bg-gray-200/0 shadow-sm rounded-none md:rounded-sm md:mt-1 p-4 md:p-8">
                        <div class="mb-6 p-4 bg-dark-secondary rounded-lg">
                            <h3 class="text-base md:text-lg font-semibold text-gray-800 dark:text-gray-200 mb-2 text-center">
                                {authState.isAuthenticated ? 'Rate this Series' : 'Rate this Series'}
                            </h3>
                            <div class="flex justify-center items-center gap-1 md:gap-2"
                                 role="group"
                                 aria-label="Star Rating"
                                 onmouseleave={() => hoveredRating = 0}
                                 onfocusout={() => hoveredRating = 0}
                            >
                                {#each {length: 5} as _, i}
                                    {@const starValue = i + 1}
                                    <Button
                                            type="button"
                                            size="iconLabel"
                                            variant="ghost"
                                            onmouseenter={() => hoveredRating = starValue}
                                            onclick={() => handleRatingClick(starValue)}
                                            class="transition-transform duration-200 active:scale-125 hover:scale-125"
                                    >
                                        <Star class="
                                             {(hoveredRating || userRating) >= starValue ? 'text-yellow-400' : 'text-gray-500'}
                                             {!authState.isAuthenticated ? 'opacity-50' : ''}
                                             !w-5 !h-5 md:w-6 md:h-6 fill-current stroke-1"
                                        />
                                    </Button>
                                {/each}
                            </div>
                        </div>
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-200">
                                Chapters
                            </h2>
                            <div class="text-sm text-gray-800 dark:text-gray-2000">
                                Latest: Chapter {sortedChapters[0]?.chapter_number}
                            </div>
                        </div>

                        <div class="chapter-list-container max-h-[42rem] overflow-y-auto pr-2">
                            <div class="grid gap-3 grid-cols-1 md:grid-cols-2">
                                {#each sortedChapters as chapter}
                                    <button onclick={() => handleChapterClick(chapter.chapter_number)}
                                            class="p-4 rounded-sm text-left cursor-pointer bg-gray-200/10 dark:bg-gray-200/5 border border-gray-300/90 ">
                                        <span class="flex justify-between items-center">
                                            <span class="flex flex-1 flex-col items-start">
                                                <span class="font-semibold text-gray-800 dark:text-gray-200">
                                                    {#if chapter.title}
                                                        <span class="text-gray-800 dark:text-gray-200">{chapter.title}</span>
                                                    {/if}
                                                </span>
                                                <span class="text-sm text-gray-800 dark:text-gray-200">
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
                        <p class="text-gray-800 dark:text-gray-200">No chapters available</p>
                    </div>
                {/if}

                {#if mangaId}
                    <div class="bg-gray-200/0 shadow-2xl rounded-none md:rounded-sm md:mt-1 p-4 md:p-8">
                        <h2 class="text-2xl font-bold mb-1 ml-1 text-gray-800 dark:text-gray-200">
                            Comments
                        </h2>
                        <CommentSystem
                                entityType="series"
                                entityId={+mangaId}
                                currentUser={$auth.user}
                        />
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
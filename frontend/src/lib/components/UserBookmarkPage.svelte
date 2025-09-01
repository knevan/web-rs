<script lang="ts">
    import slugify from "slugify";
    import {page} from "$app/state";
    import {goto} from "$app/navigation";
    import {apiFetch} from "$lib/store/auth";
    import {Timer} from "@lucide/svelte";

    interface LatestChapterInfo {
        chapterNumber: number;
        title: string;
    }

    interface BookmarkedSeries {
        id: number;
        title: string;
        coverImageUrl: string;
        updatedAt: string;
        latestChapter: LatestChapterInfo | null;
    }

    let bookmarks = $state<BookmarkedSeries[]>([]);
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    $effect(() => {
        async function fetchUserBookmarkLibrary() {
            isLoading = true;
            error = null;
            try {
                const response = await apiFetch(`/api/user/bookmark`);

                if (!response.ok) {
                    if (response.status === 401) {
                        const redirectTo = page.url.pathname;
                        goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
                        return;
                    }
                    throw new Error(`Failed to fetch bookmark library: ${response.statusText}`);
                }
                bookmarks = await response.json();
            } catch (err) {
                console.error('Error fetching user bookmarks:', err);
                error = err instanceof Error ? err.message : 'An unknown error occurred';
            } finally {
                isLoading = false;
            }
        }

        fetchUserBookmarkLibrary();
    });

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

        return result.join(", ");
    }

    function createSlugTitle(title: string): string {
        return slugify(title, {lower: true});
    }
</script>

<div class="max-w-4xl mx-auto">
    <div class="mb-8 border-b p-6">
        <h1 class="text-xl md:text-3xl font-bold text-center text-gray-800 dark:text-gray-200">
            Your Bookmarked Manga Library
        </h1>
        <p class="text-lg md:text-3xl text-center text-gray-800 dark:text-gray-200">
            The list of Series you bookmarked.
        </p>
    </div>
    {#if isLoading}
        <div class="grid grid-cols-3 md:grid-cols-4 gap-4 md:gap-5">
            {#each Array(10) as _}
                <div class="animate-pulse">
                    <div class="aspect-[2/3] bg-gray-700 rounded-md"></div>
                    <div class="h-4 bg-gray-700 rounded mt-2 w-3/4"></div>
                    <div class="h-3 bg-gray-700 rounded mt-1 w-1/2"></div>
                </div>
            {/each}
        </div>
    {:else if error}
        <div class="text-center py-10 bg-red-900/20 border border-red-500 rounded-lg">
            <p class="text-red-300">Could not load your bookmarks.</p>
            <p class="text-sm text-gray-400">{error}</p>
        </div>
    {:else if bookmarks.length === 0}
        <div class="text-center py-16 border-2 border-dashed border-gray-700 rounded-lg">
            <h3 class="text-xl font-semibold text-white">Your Bookmark Library is Empty</h3>
            <p class="text-gray-400 mt-2">
                Start adding series to your library to see them here.
            </p>
            <button onclick={() => goto('/')}
                    class="mt-6 bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg transition-colors">
                Browse Series
            </button>
        </div>
    {:else}
        <ul class="grid grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-5">
            {#each bookmarks as bookmark (bookmark.id)}
                <li>
                    <a
                            href="/manga/{bookmark.id}/{createSlugTitle(bookmark.title)}"
                            class="group block transition-transform duration-300 ease-in-out hover:-translate-y-1"
                    >
                        <figure class="aspect-[2/3] bg-gray-800 rounded-md shadow-lg overflow-hidden">
                            <img
                                    src={bookmark.coverImageUrl}
                                    alt="Cover for {bookmark.title}"
                                    class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                    loading="lazy"
                            />
                        </figure>
                        <h4 class="font-semibold text-lg truncate text-gray-800 dark:text-gray-100"
                            title="{bookmark.title}">
                            {bookmark.title}
                        </h4>
                    </a>
                    <div class="text-gray-600 dark:text-gray-400 space-y-1 mt-2">
                        <span class="text-lg ml-1 block mb-1 text-wrap">
                            <Timer class="inline-block mr-2 mt-4"/>
                            {formatRelativeTime(bookmark.updatedAt)}
                        </span>
                        <div>
                            <span class="block text-lg border-b pb-0.5 mb-1">Latest Chapter</span>
                            <span class="truncate text-lg">
                                {#if bookmark.latestChapter}
                                    <a href="/manga/{bookmark.id}/{createSlugTitle(bookmark.title)}/read-chapter/{bookmark.latestChapter.chapterNumber}"
                                       class="truncate text-lg block"
                                    >
                                        Chapter {bookmark.latestChapter.title ?? `Chapter ${bookmark.latestChapter.chapterNumber}`}
                                    </a>
                                {:else }
                                    <span class="truncate text-lg">No Chapter</span>
                                {/if}
                            </span>
                        </div>
                    </div>

                </li>
            {/each}
        </ul>
    {/if}
</div>
<script lang="ts">
    import slugify from "slugify";
    import {page} from "$app/state";
    import {goto} from "$app/navigation";

    interface LatestChapterInfo {
        chapter_number: number;
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
                const response = await fetch(`/api/user/bookmark`);

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

        return result.join(', ');
    }

    function createSlugTitle(title: string): string {
        return slugify(title, {lower: true});
    }
</script>

<div class="w-full">
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
        <div class="grid grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-5">
            {#each bookmarks as bookmark (bookmark.id)}
                <a
                        href="/manga/{bookmark.id}/{createSlugTitle(bookmark.title)}"
                        class="group relative overflow-hidden rounded-md shadow-lg block transition-transform duration-300 ease-in-out hover:-translate-y-1"
                >
                    <div class="aspect-[2/3] bg-blue-600 rounded-md">
                        <img
                                src={bookmark.coverImageUrl}
                                alt="Cover"
                                class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                loading="lazy"
                        />
                        <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent"></div>
                    </div>

                    <div class="absolute bottom-0 left-0 right-0 p-3 text-white">
                        <h3 class="font-semibold truncate text-base" title="{bookmark.title}">
                            {bookmark.title}
                        </h3>
                        <div class="flex items-center justify-between text-sm text-gray-400 mt-1">
                            <span class="truncate">
                                {#if bookmark.latestChapter}
                                    {bookmark.latestChapter.title ?? `Chapter ${bookmark.latestChapter.chapter_number}`}
                                    {:else }
                                    No Chapter
                                    {/if}
                            </span>
                            <span class="">{formatRelativeTime(bookmark.updatedAt)}</span>
                        </div>
                    </div>
                </a>
            {/each}
        </div>
    {/if}
</div>
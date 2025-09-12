<script lang="ts">
    import {goto} from "$app/navigation";
    import {ChevronRight, ChevronLeft} from "@lucide/svelte";
    import {Button} from "$lib/components/ui/button";
    import * as Select from '$lib/components/ui/select/index.js';
    import CommentSystem from "$lib/components/comments/CommentSystem.svelte";
    import {auth} from "$lib/store/auth";

    interface ChapterViewData {
        seriesTitle: string;
        chapterTitle: string;
        chapterNumber: number;
        pages: string[];
        allChapters: { chapter_number: number, title: string }[];
        prevChapterNumber: number | null;
        nextChapterNumber: number | null;
    }

    let {mangaId, chapterNumber}: { mangaId: string, chapterNumber: string } = $props();

    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let chapterView = $state<ChapterViewData | null>(null);
    let selectedChapterString = $state(chapterNumber);

    $effect(() => {
        async function loadChapterData() {
            isLoading = true;
            error = null;
            //chapterViewData = null;

            try {
                const response = await fetch(`/api/series/${mangaId}/chapter/${chapterNumber}`);
                if (!response.ok) {
                    const errorData = await response.json();
                    throw new Error(errorData.message || `Failed to fetch: ${response.status}`);
                }

                chapterView = await response.json();

                try {
                    // Record the views using a fire-and-forget approach
                    void fetch(`/api/series/${mangaId}/views-count`, {method: 'POST'});
                    console.log(`View recorded for series ID: ${mangaId}`);
                } catch (viewError) {
                    console.error(viewError);
                }

                if (chapterView) {
                    selectedChapterString = chapterView.chapterNumber.toString();
                }

                console.log('API Response for chapterView:', $state.snapshot(chapterView));
            } catch (e: any) {
                error = e.message;
            } finally {
                isLoading = false;
            }
        }

        loadChapterData();

        document.documentElement.scrollTo(0, 0);
    });

    function navigateToChapter(num: number | string | null | undefined) {
        if (num) {
            goto(`/manga/${mangaId}/read-chapter/${num}`);
        }
    }

    function goBackToSeries() {
        goto(`/manga/${mangaId}`);
    }

    function handleKeydown(_event: KeyboardEvent) {
        if (!isLoading && chapterView && selectedChapterString !== chapterNumber) {
            navigateToChapter(selectedChapterString);
        }
    }

    // This effect runs whenever the user selects a new chapter from the dropdown.
    // It navigates to the new chapter.
    $effect(() => {
        if (!isLoading && chapterView && selectedChapterString !== chapterNumber) {
            navigateToChapter(selectedChapterString);
        }
    })

</script>

<svelte:head>
    <title>
        {chapterView ? `${chapterView.seriesTitle} - Ch. ${chapterView.chapterNumber}` : 'Loading Chapter...'}
    </title>
</svelte:head>

<svelte:window onkeydown={handleKeydown}/>

<header class="top-0 z-20 backdrop-blur-md border border-gray-700 shadow-lg mt-2">
    <div class="max-w-7xl mx-auto md:px-2 pt-1">
        <div class="flex flex-col gap-1 pb-1">
            <div class="flex items-center gap-4">
                <div class="pl-4">
                    <h1 class="text-lg font-bold truncate">{chapterView?.seriesTitle || 'Loading...'}</h1>
                    <p class="text-sm text-gray-800 dark:text-gray-200 -mt-1 truncate">
                        Chapter {chapterView?.chapterTitle || '...'}</p>
                </div>
            </div>

            {#if chapterView}
                <div class="flex justify-between items-center gap-2 pt-3 px-4 md:px-[100px]">
                    <Button size="lg"
                            onclick={() => navigateToChapter(chapterView?.prevChapterNumber)}
                            disabled={!chapterView.prevChapterNumber}
                            aria-label="Previous chapter"
                            class="px-3 py-3 rounded-md hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                        <ChevronLeft class="h-5 w-5"/>
                        PREV
                    </Button>

                    <Select.Root type="single" bind:value={selectedChapterString}>
                        <Select.Trigger class="w-[140px] text-md rounded-md border border-border bg-background">
                            Chapter {selectedChapterString}
                        </Select.Trigger>
                        <Select.Content>
                            {#each chapterView.allChapters as chap}
                                <Select.Item value={chap.chapter_number.toString()}
                                             class="text-md">
                                    Chapter {chap.chapter_number}
                                </Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>

                    <Button size="lg"
                            onclick={() => navigateToChapter(chapterView?.nextChapterNumber)}
                            disabled={!chapterView.nextChapterNumber}
                            aria-label="Previous chapter"
                            class="px-3 py-3 rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                        NEXT
                        <ChevronRight class="h-5 w-5"/>
                    </Button>
                </div>
            {/if}
            <div class="w-24 hidden sm:block">
            </div>
        </div>
    </div>
</header>

<main class="max-w-7xl mx-auto py-2 md:px-0">
    {#if isLoading}
        <div class="flex flex-col justify-center items-center h-[calc(100vh-10rem)]">
            <div class="w-12 h-12 border-4 border-gray-600 border-t-blue-500 rounded-full animate-spin"></div>
            <p class="mt-4 text-gray-400">Loading Chapter...</p>
        </div>
    {:else if error}
        <div class="text-center py-12 px-4 h-[calc(100vh-10rem)] flex flex-col justify-center items-center">
            <div class="text-red-400 mb-4">
            </div>
            <h2 class="text-2xl font-bold text-white mb-2">Failed to Load Chapter</h2>
            <p class="text-gray-400 mb-6">{error}</p>
            <button onclick={goBackToSeries}
                    class="bg-blue-600 hover:bg-blue-700 text-white font-bold px-6 py-2 rounded-lg transition-colors">
                Back to Series
            </button>
        </div>
    {:else if chapterView}
        <div class="flex flex-col items-center w-full max-w-[820px] mx-auto">
            {#each chapterView.pages as pageUrl, i (pageUrl)}
                <img src={pageUrl}
                     alt="Manga Page {i + 1} for Chapter {chapterView.chapterNumber}"
                     class="max-w-full h-auto"
                     loading="lazy"/>
            {/each}
        </div>

        <div class="flex justify-between items-center my-4 px-4 md:px-[100px]">
            <Button size="lg"
                    onclick={() => navigateToChapter(chapterView?.prevChapterNumber)}
                    disabled={!chapterView.prevChapterNumber}
                    class="flex items-center gap-2 px-4 py-2 rounded-lg  hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                <ChevronLeft/>
                PREV
            </Button>

            <Select.Root type="single" bind:value={selectedChapterString}>
                <Select.Trigger class="w-[140px] text-md rounded-md border border-border bg-background">
                    Chapter {selectedChapterString}
                </Select.Trigger>
                <Select.Content>
                    {#each chapterView.allChapters as chap}
                        <Select.Item value={chap.chapter_number.toString()}
                                     class="text-md">
                            Chapter {chap.chapter_number}
                        </Select.Item>
                    {/each}
                </Select.Content>
            </Select.Root>

            <Button size="lg"
                    onclick={() => navigateToChapter(chapterView?.nextChapterNumber)}
                    disabled={!chapterView.nextChapterNumber}
                    class="flex items-center gap-2 px-4 py-2 rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                NEXT
                <ChevronRight/>
            </Button>
        </div>
        <div class="max-w-[820px] mx-auto mt-4 px-4 md:px-0">
            <h2 class="text-2xl font-bold mb-1 ml-1">
                Comments
            </h2>
            <CommentSystem
                    seriesId={+mangaId}
                    currentUser={$auth.user}
            />
        </div>
    {/if}
</main>
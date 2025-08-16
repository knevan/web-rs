<script lang="ts">
    import {goto} from "$app/navigation";
    import {ChevronRight, ChevronLeft} from "@lucide/svelte";
    import {Button} from "$lib/components/ui/button";
    import * as Select from '$lib/components/ui/select/index.js';


    interface ChapterPageData {
        pages: string[];
    }

    interface SeriesDetailsData {
        series: { title: string; };
        chapters: { chapter_number: number; title: string | null }[];
    }

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

    const parsedChapterNumber = $derived(parseInt(chapterNumber, 10));

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

    function handleKeydown(event: KeyboardEvent) {
        if (!isLoading && chapterView && selectedChapterString !== chapterNumber) {
            navigateToChapter(selectedChapterString);
        }
    }

    let currentChapterString = $derived(
        chapterView ? chapterView.chapterNumber.toString() : ''
    );

    $effect(() => {
        if (!isLoading && chapterView && selectedChapterString !== chapterNumber) {
            navigateToChapter(selectedChapterString);
        }
    })

    // This effect runs whenever the user selects a new chapter from the dropdown.
    // It navigates to the new chapter.
    $effect(() => {
        // We check if chapterView exists to avoid running on initial load
        // before data is ready. We also parse the string back to a number.
        if (chapterView && currentChapterString) {
            const newChapterNum = parseInt(currentChapterString, 10);
            // Navigate only if the selected chapter is different from the current one.
            if (newChapterNum !== chapterView.chapterNumber) {
                navigateToChapter(newChapterNum);
            }
        }
    });
</script>

<svelte:head>
    <title>
        {chapterView ? `${chapterView.seriesTitle} - Ch. ${chapterView.chapterNumber}` : 'Loading Chapter...'}
    </title>
</svelte:head>

<svelte:window on:keydown={handleKeydown}/>

<header class="top-0 z-20 backdrop-blur-md border border-gray-700 shadow-lg mt-2">
    <div class="max-w-7xl mx-auto px-4 pt-2">
        <div class="flex flex-col gap-1 pb-1">
            <div class="flex items-center gap-4">
                <div class="pl-4">
                    <h1 class="text-lg font-bold truncate">{chapterView?.seriesTitle || 'Loading...'}</h1>
                    <p class="text-sm text-gray-400 -mt-1 truncate">Chapter {chapterView?.chapterTitle || '...'}</p>
                </div>
            </div>

            {#if chapterView}
                <div class="flex justify-between items-center gap-2 pt-3 px-4">
                    <Button size="lg"
                            onclick={() => navigateToChapter(chapterView?.prevChapterNumber)}
                            disabled={!chapterView.prevChapterNumber}
                            aria-label="Previous chapter"
                            class="px-3 py-3 rounded-md hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                        <ChevronLeft class="h-5 w-5"/>
                        PREV
                    </Button>

                    <!--<select onchange={(e) => navigateToChapter(e.currentTarget.value)}
                            class=" border border-gray-600 rounded-md px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                            bind:value={chapterView.chapterNumber}>
                        {#each chapterView.allChapters as chap}
                            <option value={chap.chapter_number}>
                                Chapter {chap.chapter_number}
                            </option>
                        {/each}
                    </select>-->

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
                <i class="fas fa-exclamation-triangle text-5xl"></i>
            </div>
            <h2 class="text-2xl font-bold text-white mb-2">Failed to Load Chapter</h2>
            <p class="text-gray-400 mb-6">{error}</p>
            <button onclick={goBackToSeries}
                    class="bg-blue-600 hover:bg-blue-700 text-white font-bold px-6 py-2 rounded-lg transition-colors">
                Back to Series
            </button>
        </div>
    {:else if chapterView}
        <div class="flex flex-col items-center max-w-[820px]">
            {#each chapterView.pages as pageUrl, i (pageUrl)}
                <img src={pageUrl}
                     alt="Manga Page {i + 1} for Chapter {chapterView.chapterNumber}"
                     class="max-w-full h-auto"
                     loading="lazy"/>
            {/each}
        </div>

        <div class="flex justify-between items-center my-4">
            <Button size="lg"
                    onclick={() => navigateToChapter(chapterView?.prevChapterNumber)}
                    disabled={!chapterView.prevChapterNumber}
                    class="flex items-center gap-2 px-4 py-2 rounded-lg  hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                <ChevronLeft/>
                PREV
            </Button>
            <select onchange={(e) => navigateToChapter(e.currentTarget.value)}
                    class=" border border-gray-600 rounded-md px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                    bind:value={chapterView.chapterNumber}>
                {#each chapterView.allChapters as chap}
                    <option value={chap.chapter_number}>
                        Chapter {chap.chapter_number}
                    </option>
                {/each}
            </select>
            <Button size="lg"
                    onclick={() => navigateToChapter(chapterView?.nextChapterNumber)}
                    disabled={!chapterView.nextChapterNumber}
                    class="flex items-center gap-2 px-4 py-2 rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                NEXT
                <ChevronRight/>
            </Button>
        </div>
    {/if}
</main>
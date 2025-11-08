<script lang="ts">
    import {Search, X} from "@lucide/svelte";
    import { search } from '$lib/store/searchStore.svelte.js';
	import Input from "./ui/input/input.svelte";
	import Button from "./ui/button/button.svelte";
    import slugify from "slugify";
	import { error } from "@sveltejs/kit";

    interface UserSearchSeriesResult {
        id: number;
        title: string;
        original_title: string | null;
        cover_image_url: string;
        last_chapter_found_in_storage: number | null;
        updated_at: string;
    }

    // This variable will hold the reference to the input element.
    let searchInput = $state<HTMLInputElement | null>(null);
    let searchValue = $state('');
    let searchResults = $state<UserSearchSeriesResult[]>([]);
    let isFocused = $state(false);
    let totalResults = $state(0);
    let debounceTimer: number | null = $state(null);
    let abortController: AbortController | null = $state(null);
    let isLoading = $state(false);

    const mockSeries = [
        { id: 1, title: 'Kimetsu no Yaiba', author: 'Gotouge Koyoharu' },
        { id: 2, title: 'Mo Dao Zu Shi', author: 'Mo Xiang Tong Xiu' },
        { id: 3, title: 'Mairimashita! Iruma-kun', author: 'Nishi Osamu' }
    ];

    let filteredSeries = $derived(() => {
        console.log(`%c$derived is running. searchValue: "${searchValue}"`);
        if (!searchValue.trim()) {
            return [];
        }
        const lowercasedQuery = searchValue.toLowerCase();
        const result = mockSeries.filter(
            (series) =>
                series.title.toLowerCase().includes(lowercasedQuery) ||
                series.author.toLowerCase().includes(lowercasedQuery)
        );
        console.log(`%cFiltering result:`, 'color: green;', result);
        return result
    });

    $effect(() => {
        const query = searchValue.trim();

        if (debounceTimer) {
            clearTimeout(debounceTimer);
        }

        if (abortController) {
            abortController.abort();
        }

        if (query === '') {
            isLoading = false;
            searchResults = [];
            totalResults = 0;
            return;
        }

        isLoading = true;

        debounceTimer = setTimeout(async () => {
            abortController = new AbortController();
            const { signal } = abortController;

            try {
                const response = await fetch(`/api/series/search?q=${encodeURIComponent(query)}`, {
                    signal
                });

                if (!response.ok) {
                    throw new Error('Network response was not ok');
                }

                const data: UserSearchSeriesResult[] = await response.json();

                searchResults = data;
                totalResults = data.length;
            } catch (err) {
                if ((err as Error).name !== 'AbortError') {
                    console.error('Search fetch error:', err);
                    searchResults = [];
                    totalResults = 0;
                }
            } finally {
                isLoading = false;
            }
        }, 300);

        return () => {
            if (debounceTimer) {
                clearTimeout(debounceTimer);
            }
            if (abortController) {
                abortController.abort();
            }
        }
    });

    // Use an effect to automatically focus the input when it becomes visible.
    $effect(() => {
        console.log(
            'DEBUG: `SearchSeries.svelte` $effect for focus is running. `search.isOpen` is:',
            search.isOpen
        );
        if (search.isOpen) {
            isFocused = true;
            searchInput?.focus();
        } else {
            isFocused = false;
        }
    });

    // Use an effect to add a global keydown listener to close the search on 'Escape' press.
    $effect(() => {
        console.log('DEBUG: `SearchSeries.svelte` $effect for keydown listener is setting up.');
        const handleKeydown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                search.close();
            }
        };
        window.addEventListener('keydown', handleKeydown);
        // Cleanup function: remove the event listener when the component is destroyed.
        return () => {
            console.log(
                'DEBUG: `SearchSeries.svelte` is being destroyed (unmounted). Removing keydown listener.'
            );
            window.removeEventListener('keydown', handleKeydown);
        };
    });

    function formatRelativeTime(datestring: string): string {
        if (!datestring) return '';
        const date = new Date(datestring);
        const now = new Date();
        let seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        if (seconds < 60) return 'Just Now';

        const interval = {
            day: 86400,
            hour: 3600,
            minute: 60
        };

        const result: string[] = [];
        for (const [unit, unitSeconds] of Object.entries(interval)) {
            if (result.length >= 2) break;
            const count = Math.floor(seconds / unitSeconds);
            if (count > 0) {
                result.push(`${count} ${unit}${count > 1 ? 's' : ''}`);
                seconds %= unitSeconds;
            }
        }
        return result.length > 0 ? `${result.join(', ')}` : 'Just Now';
    }

    function handleCloseClick() {
        console.log('DEBUG: Close button clicked. Calling `search.close()`.');
        search.close();
    }

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement;
        searchValue = target.value;
        console.log(`%c[on:input] searchValue updated to: "${searchValue}"`, 'color: violet;');
    }

    function handleFocusOut(event: FocusEvent) {
        const container = event.currentTarget as HTMLElement;
        const relatedTarget = event.relatedTarget as Node | null;

        // Hide the dropdown ONLY if the new focused element is outside the search container.
        // This prevents it from closing when clicking inside the dropdown itself later.
        if (!relatedTarget || !container.contains(relatedTarget)) {
            isFocused = false;
        }
    }

    function highlightMatchText(text: string, query: string) {
        if (!query) return text;
        const regex = new RegExp(`(${query.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&')})`, 'gi');
        return text.replace(regex, '<mark class="bg-blue-300 dark:bg-blue-700 bg-opacity-50 px-0 py-0 rounded-sm">$1</mark>');
    }
</script>


<div class="mx-auto flex w-full max-w-5xl items-center gap-0.5 px-2 py-2 mt-4 lg:px-0">
    <div class="relative w-full"
         onfocusin={() => (isFocused = true)}
         onfocusout={handleFocusOut}>
        <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
                value={searchValue}
                bind:ref={searchInput}
                oninput={handleInput}
                type="text"
                placeholder="Search..."
                class="pl-10 border-2 !border-blue-400 bg-transparent rounded-none text-[--color-text] placeholder:text-gray-500 focus-visible:ring-0 focus-visible:ring-offset-0"
        />

        {#if isFocused}
        <div
                class="absolute top-full left-0 bg-gray-300 right-0 z-50 border bg-base-200 p-1 shadow-lg"
        >
            {#if searchValue.length === 0}
                <div class="flex h-5 items-center justify-center text-center text-gray-700 dark:text-white">
                    <h1 class="text-base">series title or original title</h1>
                </div>
            {:else}
                {#if filteredSeries().length > 0}
                    <div class="space-y-3">
                        {#each filteredSeries() as series (series.id)}
                            <div class="flex items-center justify-between">
                                <div class="bg-white w-full custom-scrollbar">
                                    <div class="font-bold text-gray-800 dark:text-white">{series.title}</div>
                                    <div class="text-sm text-gray-600 dark:text-gray-400">{series.author}</div>
                                </div>
                            </div>
                        {/each}
                    </div>
                {:else}
                        <div class="justify-center h-5">
                            <div>No Result</div>
                        </div>
                    {/if}
                {/if}
            </div>
        {/if}
    </div>

    <Button
            onclick={handleCloseClick}
            variant="ghost"
            size="iconLabel"
            aria-label="Close Search"
            class="cursor-pointer text-[--color-text] transition-colors hover:text-[--color-theme-1]"
    >
        <X />
    </Button>
</div>

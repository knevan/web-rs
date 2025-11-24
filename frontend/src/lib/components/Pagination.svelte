<script lang="ts">
    import {Button} from "$lib/components/ui/button";
    import {Input} from "$lib/components/ui/input";

    let {
        currentPage = $bindable(),
        totalPages,
        pageCount = 5,
    } = $props<{
        currentPage: number;
        totalPages: number;
        pageCount?: number;
    }>();

    type EllipsisSide = 'left' | 'right';
    let editingEllipsis = $state<EllipsisSide | null>(null);
    let jumpInputValue = $state('');
    let debounceTimer: ReturnType<typeof setTimeout>;

    type PageItem = number | { type: 'ellipsis'; id: EllipsisSide };

    const pageNumbers = $derived(() => {
        // If pageCount is 5 and totalPages is 7 or less, it will show [1, 2, 3, 4, 5, 6, 7].
        if (totalPages <= pageCount + 5) {
            const pages: PageItem[] = [];
            for (let i = 1; i <= totalPages; i++) pages.push(i);
            return pages;
        }

        const pages: PageItem[] = [];
        const half = Math.floor(pageCount / 2);

        // If currentPage is 1, 2, or 3, it shows [1, 2, 3, 4, 5, '...', 100].
        if (currentPage <= half + 2) {
            for (let i = 1; i <= pageCount; i++) pages.push(i);
            pages.push({type: 'ellipsis', id: 'right'});
            pages.push(totalPages);
        }
        // If currentPage is 98, 99, or 100, it shows [1, '...', 96, 97, 98, 99, 100].
        else if (currentPage >= totalPages - half - 1) {
            pages.push(1);
            pages.push({type: 'ellipsis', id: 'left'});
            for (let i = totalPages - pageCount + 1; i <= totalPages; i++) pages.push(i);
        }
        // Shows [1, '...', 43, 44, 45, 46, 47, '...', 100].
        else {
            pages.push(1);
            pages.push({type: 'ellipsis', id: 'left'});
            const startRange = currentPage - half;
            const endRange = currentPage + half;
            for (let i = startRange; i <= endRange; i++) pages.push(i);
            pages.push({type: 'ellipsis', id: 'right'});
            pages.push(totalPages);
        }
        return pages;
    });

    function performJumpToPage() {
        if (!jumpInputValue) {
            cancelEdit();
            return;
        }

        const page = parseInt(jumpInputValue, 10);
        if (!isNaN(page)) {
            currentPage = Math.max(1, Math.min(page, totalPages));
        }

        cancelEdit();
    }

    // activate input field
    function startEdit(side: EllipsisSide) {
        editingEllipsis = side;
        jumpInputValue = '';
    }

    // reset
    function cancelEdit() {
        editingEllipsis = null;
        jumpInputValue = '';
        clearTimeout(debounceTimer)
    }

    function handleInputDebounce() {
        clearTimeout(debounceTimer)
        debounceTimer = setTimeout(() => {
            performJumpToPage();
        }, 1500);
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Enter') {
            clearTimeout(debounceTimer);
            performJumpToPage();
        } else if (event.key === 'Escape') {
            cancelEdit();
        }
    }
</script>

<nav aria-label="page navigation">
    <ul class="flex flex-wrap justify-center md:justify-start gap-1 md:gap-0 md:flex-nowrap md:inline-flex items-center md:-space-x-px text-sm">
        <li>
            <Button onclick={() => (currentPage -= 1)}
                    variant="outline"
                    disabled={currentPage === 1}
                    class="rounded-none md:rounded-s-md md:rounded-e-none flex items-center justify-center px-3 h-8 leading-tight text-gray-500 hover:bg-gray-100 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
                &laquo;
            </Button>
        </li>
        {#each pageNumbers() as page, i(i)}
            <li>
                {#if typeof page === 'number'}
                    <Button
                            onclick={() => (currentPage = page)}
                            class="rounded-none"
                            variant={currentPage === page ? 'default' : 'outline'}
                            size="sm"
                    >
                        {page}
                    </Button>
                {:else if page.type === 'ellipsis'}
                    {#if editingEllipsis === page.id}
                        <Input
                                type="number"
                                bind:value={jumpInputValue}
                                autofocus
                                onkeydown={handleKeydown}
                                oninput={handleInputDebounce}
                                onblur={cancelEdit}
                                class="w-12 h-8 text-center rounded-none border-gray-300 border bg-white focus:outline-none focus:ring-2 focus:ring-teal-500"
                                min="1"
                                max={totalPages}
                        />
                    {:else}
                        <Button
                                onclick={() => startEdit(page.id)}
                                variant="outline"
                                size="sm"
                                class="flex items-center justify-center rounded-none px-3 h-8 leading-tight text-gray-500 hover:bg-gray-100 hover:text-gray-700"
                                title="Jump to page..."
                        >
                            ...
                        </Button>
                    {/if}
                {/if}
            </li>
        {/each}
        <li>
            <Button
                    onclick={() => (currentPage += 1)}
                    variant="outline"
                    size="sm"
                    disabled={currentPage === totalPages}
                    class="flex items-center justify-center px-3 h-8 leading-tight text-gray-500 rounded-none md:rounded-s-none md:rounded-e-md hover:bg-gray-100 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
                &raquo;
            </Button>
        </li>
    </ul>
</nav>
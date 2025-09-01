<script lang="ts">
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

    function autoFocusInput(node: HTMLInputElement) {
        node.focus();
        node.select();
    }
</script>

<nav aria-label="page navigation">
    <ul class="inline-flex items-center -space-x-px text-sm">
        <li>
            <button onclick={() => (currentPage -= 1)}
                    disabled={currentPage === 1}
                    class="flex items-center justify-center px-3 h-8 ms-0 leading-tight
                           text-gray-500 bg-white border border-e-0 border-gray-300 rounded-s-lg hover:bg-gray-100 hover:text-gray-700 disabled:opacity-50"
            >
                &laquo;
            </button>
        </li>
        {#each pageNumbers() as page, i(i)}
            <li>
                {#if typeof page === 'number'}
                    <button
                            onclick={() => (currentPage = page)}
                            class="flex items-center justify-center px-3 h-8 leading-tight text-gray-500 bg-white border border-gray-300 hover:bg-gray-100 hover:text-gray-700"
                            class:bg-teal-600={currentPage === page}
                            class:text-white={currentPage === page}
                            class:text-gray-500={currentPage !== page}
                            class:bg-white={currentPage !== page}
                            class:hover:bg-gray-100={currentPage !== page}
                            class:hover:text-gray-700={currentPage !== page}
                    >
                        {page}
                    </button>
                {:else if page.type === 'ellipsis'}
                    {#if editingEllipsis === page.id}
                        <input
                                type="number"
                                bind:value={jumpInputValue}
                                use:autoFocusInput
                                onkeydown={handleKeydown}
                                oninput={handleInputDebounce}
                                onblur={cancelEdit}
                                class="w-12 h-8 text-center border-gray-300 border bg-white focus:outline-none focus:ring-2 focus:ring-teal-500"
                                min="1"
                                max={totalPages}
                        />
                    {:else}
                        <button
                                onclick={() => startEdit(page.id)}
                                class="flex items-center justify-center px-3 h-8 leading-tight text-gray-500 bg-white border border-gray-300 hover:bg-gray-100 hover:text-gray-700"
                                title="Jump to page..."
                        >
                            ...
                        </button>
                    {/if}
                {/if}
            </li>
        {/each}
        <li>
            <button
                    onclick={() => (currentPage += 1)}
                    disabled={currentPage === totalPages}
                    class="flex items-center justify-center px-3 h-8 leading-tight text-gray-500 bg-white border border-gray-300 rounded-e-lg hover:bg-gray-100 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
                &raquo;
            </button>
        </li>
    </ul>
</nav>
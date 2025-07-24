<script lang="ts">
    let {
        currentPage = $bindable(),
        totalPages,
        siblingCount = 4,
        boundaryCount = 1,
    } = $props<{
        currentPage: number;
        totalPages: number;
        siblingCount?: number;
        boundaryCount?: number;
    }>();

    const pageNumbers = $derived(() => {
        const totalPagesToShow = boundaryCount * 2 + siblingCount * 2 + 3;

        if (totalPages <= totalPagesToShow) {
            // if not enough pages, show all pages
            return Array.from({length: totalPages}, (_, i) => i + 1);
        }

        const result: (number | string)[] = [];
        const startPage = Array.from({length: boundaryCount}, (_, i) => i + 1);
        const endPage = Array.from({length: boundaryCount}, (_, i) => totalPages - boundaryCount + 1 + i);

        const leftSiblingStart = Math.max(currentPage - siblingCount, boundaryCount + 1);
        const rightSiblingEnd = Math.min(currentPage + siblingCount, totalPages - boundaryCount);

        // add start pages
        result.push(...startPage);

        // add left ellipsis if needed
        if (leftSiblingStart > boundaryCount + 1) {
            result.push('...');
        }

        // add middle pages (sibling pages)
        for (let i = leftSiblingStart; i <= rightSiblingEnd; i++) {
            if (!result.includes(i)) {
                result.push(i);
            }
        }

        // add right ellipsis if needed
        if (rightSiblingEnd < totalPages - boundaryCount) {
            result.push('...');
        }

        // add end pages
        for (const page of endPage) {
            if (!result.includes(page)) {
                result.push(page);
            }
        }

        return result;
    });

    function handleJumpToPage() {
        const input = prompt(`Enter page number (1 - ${totalPages}:`);
        if (input === null || input.trim() === '') {
            return;
        }

        let page = parseInt(input, 10);

        if (isNaN(page)) {
            alert('invalid number');
            return;
        }

        // clam page
        if (page < 1) {
            page = 1;
        }
        if (page > totalPages) {
            page = totalPages;
        }
        currentPage = page;
    }
</script>

<nav aria-label="page navigation">
    <ul class="inline-flex items-center -space-x-px text-sm">
        <li>
            <button onclick={() => (currentPage = 1)}
                    disabled={currentPage === 1}
                    class="flex items-center justify-center px-3 h-8 ms-0 leading-tight
                           text-gray-500 bg-white border border-e-0 border-gray-300 rounded-s-lg hover:bg-gray-100 hover:text-gray-700 disabled:opacity-50"
            >
                &laquo;
            </button>
        </li>

        {#each pageNumbers() as page, i(i)}
            <li>
                {#if typeof page === 'string'}
                    <button
                            onclick={handleJumpToPage}
                            class="flex items-center justify-center px-3 h-8 leading-tight text-gray-500 bg-white border border-gray-300 hover:bg-gray-100 hover:text-gray-700"
                            title="Jump to page..."
                    >
                        ...
                    </button>
                {:else}
                    <button
                            onclick={() => (currentPage = page)}
                            class="flex items-center justify-center px-3 h-8 leading-tight border border-gray-300"
                            class:bg-teal-600={currentPage === page}
                            class:text-white={currentPage === page}
                            class:text-gray-500={currentPage !== page}
                            class:bg-white={currentPage !== page}
                            class:hover:bg-gray-100={currentPage !== page}
                            class:hover:text-gray-700={currentPage !== page}
                    >
                        {page}
                    </button>
                {/if}
            </li>
        {/each}
        <li>
            <button
                    onclick={() => (currentPage = totalPages)}
                    disabled={currentPage === totalPages}
                    class="flex items-center justify-center px-3 h-8 leading-tight text-gray-500 bg-white border border-gray-300 rounded-e-lg hover:bg-gray-100 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
                &raquo;
            </button>
        </li>
    </ul>
</nav>
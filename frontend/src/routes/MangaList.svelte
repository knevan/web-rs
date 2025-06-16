<script lang="ts">
    import image_test from '$lib/images/image_image.webp'
    import {onMount} from 'svelte'

    interface MangaItem {
        title: string;
        // Add other properties that your manga items have
        // For example: cover?: string; author?: string; etc.
    }

    export let manga: MangaItem[] = [];
    export let itemsPerPage = 6;

    const placeholderImage = image_test;

    let scrollContainer: HTMLDivElement;
    let showLeftArrow = false;
    let showRightArrow = true;
    let itemWidth = 0;
    let containerWidth = 0;

    function calculateScrollDistance() {
        if (!scrollContainer) return 0;
        
        // Get the width of a single item including its margin/gap
        const firstItem = scrollContainer.querySelector('.manga-item');
        if (!firstItem) return 0;
        
        // Calculate item width including gap (20px from CSS)
        const itemRect = firstItem.getBoundingClientRect();
        itemWidth = itemRect.width + 20; // width + gap
        
        // Calculate container width
        containerWidth = scrollContainer.clientWidth;
        
        // Return the width of 6 items or the container width, whichever is smaller
        return Math.min(itemWidth * itemsPerPage, containerWidth);
    }

    function scrollLeft() {
        if (!scrollContainer) return;
        const scrollDistance = calculateScrollDistance();
        scrollContainer.scrollBy({left: -scrollDistance, behavior: 'smooth'});
    }

    function scrollRight() {
        if (!scrollContainer) return;
        const scrollDistance = calculateScrollDistance();
        scrollContainer.scrollBy({left: scrollDistance, behavior: 'smooth'});
    }

    function checkScrollPosition() {
        if (!scrollContainer) return;

        showLeftArrow = scrollContainer.scrollLeft > 0;
        showRightArrow = scrollContainer.scrollLeft < (scrollContainer.scrollWidth - scrollContainer.clientWidth - 10);
    }

    onMount(() => {
        checkScrollPosition();
        scrollContainer.addEventListener('scroll', checkScrollPosition);

        return () => {
            if (scrollContainer) {
                scrollContainer.removeEventListener('scroll', checkScrollPosition);
            }
        };
    });
</script>

<div class="manga-list-container">
    {#if showLeftArrow}
        <button class="scroll-button left" on:click={scrollLeft} aria-label="Scroll left">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                <path fill="none" d="M0 0h24v24H0z"/>
                <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" fill="currentColor"/>
            </svg>
        </button>
    {/if}

    <div class="manga-list" bind:this={scrollContainer}>
        {#each manga as item}
            <div class="manga-item">
                <img src={placeholderImage} alt={item.title}/>
                <h3>{item.title}</h3>
            </div>
        {/each}
    </div>

    {#if showRightArrow}
        <button class="scroll-button right" on:click={scrollRight} aria-label="Scroll right">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                <path fill="none" d="M0 0h24v24H0z"/>
                <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z" fill="currentColor"/>
            </svg>
        </button>
    {/if}
</div>


<style>
    .manga-list-container {
        position: relative;
        width: 100%;
        max-width: 1100px;
        margin: 0 auto;
        padding: 0 0px;
    }

    .manga-list {
        display: flex;
        overflow-x: auto;
        scroll-behavior: smooth;
        scrollbar-width: none; /* Firefox */
        -ms-overflow-style: none; /* IE and Edge */
        gap: 20px;
        padding: 10px 0;
        justify-content: flex-start;
        scroll-snap-type: x mandatory;
    }

    /* Hide scrollbar for Chrome, Safari and Opera */
    .manga-list::-webkit-scrollbar {
        display: none;
    }

    .manga-item {
        flex: 0 0 auto;
        width: 145px;
        text-align: center;
        scroll-snap-align: start;
    }

    .manga-item img {
        width: 100%;
        height: 220px;
        object-fit: cover;
        border-radius: 4px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .manga-item h3 {
        margin-top: 8px;
        font-size: 14px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .scroll-button {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: white;
        border: none;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
        cursor: pointer;
        z-index: 10;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .scroll-button.left {
        left: -50px;
    }

    .scroll-button.right {
        right: -50px;
    }

    .scroll-button:hover {
        background-color: #f5f5f5;
    }
</style>
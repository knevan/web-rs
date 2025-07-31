<script lang="ts">
    // Import necessary functions and types from Svelte and Embla Carousel
    import emblaCarouselSvelte from "embla-carousel-svelte";
    import type {EmblaCarouselType, EmblaOptionsType} from "embla-carousel";
    import Autoplay from 'embla-carousel-autoplay';
    import image_test from '$lib/images/image_image.webp';
    import {ChevronLeft, ChevronRight} from "@lucide/svelte";

    // Define the props for the component
    interface Props {
        manga?: MangaItem[];
    }

    interface MangaItem {
        title: string;
    }

    // Define props with default values using destructuring
    let {manga = []}: Props = $props();

    // State declaration state
    // The emblaApi instance is a reactive state variable.
    let emblaApi = $state<EmblaCarouselType | undefined>(undefined);

    // Placeholder image for development
    const placeholderImage = image_test;

    // Carousel options
    const options: EmblaOptionsType = {
        loop: true,
        align: 'start',
        dragFree: true,
    }

    // Initialize the Autoplay plugin
    // We set stopOnInteraction to false, so we can control the play/stop state manually.
    const autoPlay = Autoplay({
        delay: 1500,
        stopOnInteraction: false,
        stopOnMouseEnter: true,
    })

    // Variable to hold the Embla API instance
    // let emblaApi: EmblaCarouselType;

    // Carousel Control
    function scrollPrev() {
        emblaApi?.scrollPrev();
    }

    function scrollNext() {
        emblaApi?.scrollNext();
    }

    // Manual autoplay using embla events
    // This reactive block runs whenever emblaApi is assigned.
    $effect(() => {
        if (emblaApi) {
            const autoplayPlugin = emblaApi.plugins().autoplay;
            if (!autoplayPlugin) return;

            // When user start draging
            // Manually start the autoplay
            const onPointDown = () => {
                autoplayPlugin.stop();
            };

            // When the carousel settles after ant interaction
            // Manually start the autoplay
            const onSettle = () => {
                autoplayPlugin.play();
            };

            emblaApi.on('pointerDown', onPointDown);
            emblaApi.on('settle', onSettle);

            // Clean up listeners on component destruction or
            // if emblaApi changes
            return () => {
                if (emblaApi) {
                    emblaApi.off('pointerDown', onPointDown);
                    emblaApi.off('settle', onSettle);
                }
            };
        }
    });
</script>

<div class="manga-list-container">
    <!-- Left click button -->
    <button class="click-button left" onclick={scrollPrev} aria-label="Scroll left">
        <ChevronLeft/>
    </button>

    <!-- Embla Carousel root element -->
    <div class="manga-carousel" use:emblaCarouselSvelte={{ options, plugins: [autoPlay] }}
         onemblaInit={(e: CustomEvent<EmblaCarouselType>) => emblaApi = e.detail}>
        <!-- Embla container for the slides -->
        <div class="manga-container">
            <!-- Loop through the manga items to create slides -->
            {#each manga as item (item.title)}
                <div class="manga-slide">
                    <div class="manga-item">
                        <img src={placeholderImage} alt={item.title}/>
                        <h3>{item.title}</h3>
                    </div>
                </div>
            {/each}
        </div>
    </div>

    <!-- Right click button -->
    <button class="click-button right" onclick={scrollNext} aria-label="Scroll right">
        <ChevronRight/>
    </button>
</div>

<style>
    .manga-list-container {
        position: relative;
        width: 100%;
        max-width: 1100px;
        margin: 0 auto;
    }

    /* Manga Caraousel. It needs overflow: hidden to act as a mask. */
    .manga-carousel {
        overflow: hidden;
    }

    /* Scrollable container that holds the slides. */
    .manga-container {
        display: flex;
        gap: 20px; /* This creates space between slides */
        margin-left: -20px; /* Negative margin to compensate for the gap on the first item */
    }

    /* Each slide in the carousel. */
    .manga-slide {
        flex: 0 0 145px; /* Defines the width of each slide */
        min-width: 0;
        padding-left: 20px; /* Part of the gap implementation */
        position: relative;
    }

    .manga-item {
        text-align: center;
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

    /* Styling for navigation buttons */
    .click-button {
        position: absolute;
        top: 50%;
        /* Adjust transform to align with the image, not the whole container */
        transform: translateY(-70%);
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: rgba(255, 255, 255, 0.9);
        border: none;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
        cursor: pointer;
        z-index: 10;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color 0.2s;
    }

    .click-button.left {
        left: -50px;
    }

    .click-button.right {
        right: -50px;
    }

    .click-button:hover {
        background-color: #ffffff;
    }
</style>
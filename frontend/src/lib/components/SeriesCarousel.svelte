<script lang="ts">
    import emblaCarouselSvelte from "embla-carousel-svelte";
    import type {EmblaCarouselType, EmblaOptionsType} from "embla-carousel";
    import Autoplay from 'embla-carousel-autoplay';
    import image_test from '$lib/images/image_image.webp';
    import {ChevronLeft, ChevronRight} from "@lucide/svelte";

    // Placeholder image for development
    const placeholderImage = image_test;

    interface MangaItem {
        title: string;
    }

    // Define props with default values using destructuring
    let {manga = []}: { manga?: MangaItem[] } = $props();

    // The emblaApi instance is a reactive state variable.
    let emblaApi = $state<EmblaCarouselType | undefined>(undefined);

    // Carousel options
    const options: EmblaOptionsType = {
        loop: true,
        align: 'start',
        dragFree: true,
    }

    // Initialize the Autoplay plugin
    const autoplayPlugin = Autoplay({
        delay: 2000,
        stopOnInteraction: false,
        stopOnMouseEnter: true,
        stopOnLastSnap: false,
    })

    // Carousel Control
    function scrollPrev() {
        emblaApi?.scrollPrev();
    }

    function scrollNext() {
        emblaApi?.scrollNext();
    }

</script>

<section class="relative mx-auto w-full max-w-[1200px]">
    <!-- Left click button -->
    <button onclick={scrollPrev}
            aria-label="Scroll left"
            class="absolute left-[-50px] top-1/2 z-10 flex h-10 w-10 -translate-y-[50px] cursor-pointer items-center justify-center rounded-full border-none bg-white/90 shadow-lg transition-colors hover:bg-white">
        <ChevronLeft class="text-black"/>
    </button>

    <!-- Embla Carousel root element -->
    <div class="overflow-hidden"
         use:emblaCarouselSvelte={{ options, plugins: [autoplayPlugin] }}
         onemblaInit={(e: CustomEvent<EmblaCarouselType>) => emblaApi = e.detail}
    >
        <!-- Embla container for the slides -->
        <div class="flex gap-5">
            <!-- Loop through the manga items to create slides -->
            {#each manga as item (item.title)}
                <div class="relative min-w-0 flex-none basis-[145px]">
                    <div class="text-center">
                        <img src={placeholderImage}
                             alt={item.title}
                             class="h-[220px] w-full rounded-sm object-cover shadow-sm"
                        />
                        <h3 class="mt-2 truncate text-sm font-medium">
                            {item.title}
                        </h3>
                    </div>
                </div>
            {/each}
        </div>
    </div>

    <!-- Right click button -->
    <button onclick={scrollNext}
            aria-label="Scroll right"
            class="absolute right-[-50px] top-1/2 z-10 flex h-10 w-10 -translate-y-[50%] cursor-pointer items-center justify-center rounded-full border-none bg-white/90 shadow-lg transition-colors hover:bg-white"
    >
        <ChevronRight class="text-black"/>
    </button>
</section>
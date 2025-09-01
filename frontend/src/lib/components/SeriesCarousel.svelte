<script lang="ts">
    import emblaCarouselSvelte from "embla-carousel-svelte";
    import type {EmblaCarouselType, EmblaOptionsType} from "embla-carousel";
    import Autoplay from 'embla-carousel-autoplay';
    // import {ChevronLeft, ChevronRight} from "@lucide/svelte";
    import slugify from "slugify";

    // import {Button} from "$lib/components/ui/button";

    interface MangaItem {
        id: number;
        title: string;
        cover_image_url: string;
    }

    // Define props with default values using destructuring
    let {manga = []}: { manga?: MangaItem[] } = $props();

    // The emblaApi instance is a reactive state variable.
    let emblaApi = $state<EmblaCarouselType | undefined>(undefined);

    // Carousel options
    const options: EmblaOptionsType = {
        loop: manga.length > 5,
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

    /* Carousel Control
    function scrollPrev() {
        emblaApi?.scrollPrev();
    }

    function scrollNext() {
        emblaApi?.scrollNext();
    }*/
</script>

<section class="relative mx-auto w-full max-w-[1200px] md:mx-14">
    <!-- Left click button
    <Button onclick={scrollPrev}
            aria-label="Scroll left"
            class="absolute lg:flex left-[-50px] top-1/2 z-10 hidden md:flex h-10 w-10 -translate-y-[50px] cursor-pointer items-center justify-center rounded-full border-none bg-white/90 shadow-lg transition-colors hover:bg-white">
        <ChevronLeft class="text-black"/>
    </Button> -->

    <!-- Embla Carousel root element -->
    <div class="overflow-hidden relative flex mx-auto"
         use:emblaCarouselSvelte={{ options, plugins: [autoplayPlugin] }}
         onemblaInit={(e: CustomEvent<EmblaCarouselType>) => emblaApi = e.detail}
    >
        <!-- Embla container for the slides -->
        <div class="flex gap-5 sm:gap-5">
            <!-- Loop through the manga items to create slides -->
            {#each manga as item (item.id)}
                <div class="relative min-w-0 flex-none basis-[110px] sm:basis-[145px]">
                    <a href={`/manga/${item.id}/${slugify(item.title, { lower: true, strict: false})}`}
                       class="group block text-center"
                       aria-label={item.title}>
                        <div class="overflow-hidden rounded-sm">
                            <img src={item.cover_image_url}
                                 alt={item.title}
                                 class="h-[160px] sm:h-[220px] w-full rounded-sm object-cover shadow-sm transition-transform group-hover:scale-105"
                            />
                        </div>
                        <h3 class="mt-2 truncate text-sm font-semibold text-gray-800 dark:text-gray-200 group-hover:text-blue-600">
                            {item.title}
                        </h3>
                    </a>
                </div>
            {/each}
        </div>
    </div>

    <!-- Right click button
    <Button onclick={scrollNext}
            aria-label="Scroll right"
            class="absolute right-[-50px] hidden lg:flex top-1/2 z-10 h-10 w-10 -translate-y-[50%] cursor-pointer items-center justify-center rounded-full border-none bg-white/90 shadow-lg transition-colors hover:bg-white"
    >
        <ChevronRight class="text-black"/>
    </Button> -->
</section>
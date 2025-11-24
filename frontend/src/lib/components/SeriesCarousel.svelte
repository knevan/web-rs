<script lang="ts">
    import emblaCarouselSvelte from 'embla-carousel-svelte';
    import type {EmblaCarouselType, EmblaOptionsType} from 'embla-carousel';
    import Autoplay from 'embla-carousel-autoplay';
    import slugify from 'slugify';

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
    const options = $derived<EmblaOptionsType>({
        loop: manga.length > 5,
        align: 'start',
        dragFree: true
    });

    // Initialize the Autoplay plugin
    const autoplayPlugin = Autoplay({
        delay: 2000,
        stopOnInteraction: false,
        stopOnMouseEnter: true,
        stopOnLastSnap: false
    });

    // Force embla to reInit
    $effect(() => {
        if (emblaApi && manga.length > 0) {
            emblaApi.reInit();
        }
    })
</script>

<section class="relative mx-auto w-full">
    <!-- Embla Carousel root element -->
    <div
            class="relative mx-auto flex overflow-hidden"
            use:emblaCarouselSvelte={{ options, plugins: [autoplayPlugin] }}
            onemblaInit={(e: CustomEvent<EmblaCarouselType>) => (emblaApi = e.detail)}
    >
        <!-- Embla container for the slides -->
        <div class="flex gap-2 md:gap-5">
            <!-- Loop through the manga items to create slides -->
            {#each manga as item (item.id)}
                <div class="relative min-w-0 flex-none">
                    <a
                            href={`/manga/${item.id}/${slugify(item.title, { lower: true, strict: false })}`}
                            class="group block text-center"
                            aria-label={item.title}
                    >
                        <div class="overflow-hidden rounded-sm">
                            <img
                                    src={item.cover_image_url}
                                    alt={item.title}
                                    class="h-[160px] w-full rounded-[2px] object-cover shadow-sm transition-transform group-hover:scale-105 sm:h-[220px]"
                            />
                        </div>
                        <h3
                                class="mt-2 truncate text-sm font-semibold text-gray-800 group-hover:text-blue-600 dark:text-gray-200"
                        >
                            {item.title}
                        </h3>
                    </a>
                </div>
            {/each}
        </div>
    </div>
</section>

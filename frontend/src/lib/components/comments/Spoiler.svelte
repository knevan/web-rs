<script lang="ts">
    import {parseAndSanitize} from "$lib/utils/markdown";

    let {content} = $props<{ content: string }>();

    let revealed = $state(false);
    let spoilerElement = $state<HTMLElement | null>(null);

    const revealedContent = $derived(parseAndSanitize(content));

    // This effect handles the "click outside to close" logic.
    $effect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            // If the spoiler element exists and the click was outside of it, hide the content.
            if (revealed && spoilerElement && !spoilerElement.contains(event.target as Node)) {
                revealed = false;
            }
        };

        window.addEventListener('click', handleClickOutside, true);

        return () => {
            window.removeEventListener('click', handleClickOutside, true);
        };
    });

    function toggle(e: Event) {
        e.stopPropagation();
        revealed = !revealed;
    }
</script>

<span
        bind:this={spoilerElement}
        onclick={toggle}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggle(e); }}
        class="spoiler-container"
        class:revealed={revealed}
        role="button"
        tabindex="0"
>
	{@html revealedContent}
</span>

<style>
    .spoiler-container {
        cursor: pointer;
    }

    .spoiler-container {
        display: inline-block;
        background-color: #4a5568;
        color: transparent;
        padding: 2px 5px;
        border-radius: 4px;
        user-select: none;
        cursor: pointer;
        border: 1px dashed transparent;
        transition: background-color 0.2s ease-in-out, color 0.2s ease-in-out;
    }

    .spoiler-container.revealed {
        background-color: transparent;
        border: 1px dashed #f17106;
        color: inherit;
        user-select: auto;
        cursor: default;
    }
</style>
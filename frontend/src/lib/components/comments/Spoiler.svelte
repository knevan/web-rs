<script lang="ts">
    let {content} = $props<{ content: string }>();

    let revealed = $state(false);
    let spoilerElement = $state<HTMLElement | null>(null);

    // This effect handles the "click outside to close" logic.
    $effect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            // If the spoiler element exists and the click was outside of it, hide the content.
            if (spoilerElement && !spoilerElement.contains(event.target as Node)) {
                revealed = false;
            }
        };

        window.addEventListener('click', handleClickOutside, true);

        return () => {
            window.removeEventListener('click', handleClickOutside, true);
        };
    });
</script>

<span
        bind:this={spoilerElement}
        onclick={(e) => {
		e.stopPropagation();
		revealed = true;
	}}
        class="spoiler-container"
        role="button"
        tabindex="0"
        onkeydown={(e) => { if (e.key === 'Enter') revealed = true; }}
>
	{#if revealed}
		<span>{content}</span>
	{:else}
		<span class="spoiler-placeholder">{content}</span>
	{/if}
</span>

<style>
    .spoiler-container {
        cursor: pointer;
    }

    .spoiler-placeholder {
        background-color: #4a5568;
        color: transparent;
        padding: 1px 4px;
        border-radius: 4px;
        user-select: none;
        transition: background-color 0.2s ease-in-out;
    }

    .spoiler-container:hover .spoiler-placeholder {
        background-color: #5a6578;
    }
</style>
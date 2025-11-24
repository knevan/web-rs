<script lang="ts">
    import ChapterViewPage from "$lib/components/ChapterViewPage.svelte";
    import {page} from "$app/state";

    let {id: mangaId, chapterNumber} = $derived(page.params);
</script>

{#if mangaId && chapterNumber}
    <ChapterViewPage {mangaId} {chapterNumber}/>
{:else}
    <p>Loading chapter or invalid URL...</p>
{/if}
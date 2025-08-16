<script lang="ts">
    import ChapterViewPage from "$lib/components/ChapterViewPage.svelte";
    import {page} from "$app/state";

    const mangaId = page.params.id;
    const chapterNumber = page.params.chapterNumber;
</script>

<ChapterViewPage {mangaId} {chapterNumber}/>
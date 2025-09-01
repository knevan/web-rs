<script lang="ts">
    import {page} from "$app/state";
    import SeriesViewPage from "$lib/components/SeriesViewPage.svelte";

    const mangaId = page.params.id;
</script>

<SeriesViewPage {mangaId}/>
<script lang="ts">
    import {page} from "$app/state";
    import SeriesViewPage from "$lib/components/SeriesViewPage.svelte";

    const mangaId = $derived(page.params.id ?? null);
</script>

<SeriesViewPage {mangaId}/>
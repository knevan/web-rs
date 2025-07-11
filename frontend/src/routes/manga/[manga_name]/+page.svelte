<script lang="ts">
    import {page} from "$app/state";
    import MangaView from "$lib/components/MangaView.svelte";

    const mangaId = page.params['manga_name'];
</script>

<MangaView {mangaId}/>
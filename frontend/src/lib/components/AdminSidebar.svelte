<script lang="ts">
    // These placeholder function
    function handleAddManga() {
        console.log('Opening')
    }

    function handleDeleteManga() {
        console.log('Deleting')
    }

    function handleEditManga() {
        console.log('Editing')
    }

    function handleRepairChapter() {
        console.log('Repairing chapter')
    }

    function handleAddTags() {
        console.log('Adding tags')
    }


</script>

<aside class="p-4 rounded-lg shadow">
    <div class="flex flex-col space-y-3">
        <button onclick={handleAddManga} class="add-manga" aria-label="Add-Manga">Add Manga</button>
        <button onclick={handleDeleteManga} aria-label="Delete-Manga">Delete Manga</button>
        <button onclick={handleEditManga} aria-label="Edit-Manga">Edit Manga</button>
        <button onclick={handleRepairChapter} aria-label="Repair-Chapter">Repair Chapter</button>
        <button onclick={handleAddTags} aria-label="Add-Tags">Add Tags</button>
        <!-- Additional buttons can be added here -->
    </div>
</aside>

<style lang="postcss">
    .add-manga {
        @apply bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md;
    }
</style>
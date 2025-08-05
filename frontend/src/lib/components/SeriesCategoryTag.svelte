<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog';
    import {Button} from "$lib/components/ui/button";
    import {Badge} from "$lib/components/ui/badge/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {X} from "@lucide/svelte"

    type CategoryTag = {
        id: number;
        name: string;
    }

    let open = $state(false);
    let tags = $state<CategoryTag[]>([]);
    let newTagName = $state('');
    let isLoading = $state(false);
    let errorMessage = $state<string | null>(null);

    // Automatic sort tags alphabetically
    const sortedTags = $derived(tags.toSorted((a, b) => a.name.localeCompare(b.name)));

    // Fetch all tags
    async function loadTags() {
        isLoading = true;
        errorMessage = null;

        try {
            const response = await fetch('/api/admin/category/tag/list');

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to add tag');
            }

            const data = await response.json();
            tags = data.categories;
        } catch (e: any) {
            errorMessage = e.message;
            console.error('Error adding tags:', e);
        } finally {
            isLoading = false;
        }
    }

    async function handleAddTag() {
        const trimmedName = newTagName.trim();
        if (!trimmedName) return;

        const formattedName = trimmedName.charAt(0).toUpperCase() + trimmedName.slice(1).toLowerCase();

        isLoading = true;
        errorMessage = null;
        try {
            const response = await fetch('/api/admin/category/tag/add', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({name: formattedName})
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to add tag');
            }
            newTagName = '';
            await loadTags();
        } catch (e: any) {
            errorMessage = e.message;
            console.error('Error adding tag:', e);
        } finally {
            isLoading = false;
        }
    }

    async function handleDeleteTag(tagId: number) {
        isLoading = true;
        errorMessage = null;
        try {
            const response = await fetch(`/api/admin/category/tag/delete/${tagId}`, {
                method: 'DELETE',
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to delete tag');
            }
            await loadTags();
        } catch (e: any) {
            errorMessage = e.message;
            console.error('Error deleting tag:', e);
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        if (open) {
            loadTags();
        }
    });
</script>

<Dialog.Root bind:open>
    <Dialog.Trigger>
        <Button class="cursor-pointer">Manage Tags</Button>
    </Dialog.Trigger>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header class="-mb-3">
            <Dialog.Title>
                Manage Category Tags
            </Dialog.Title>
        </Dialog.Header>

        <Dialog.Close class="cursor-pointer">
            <span class="sr-only">Close</span>
        </Dialog.Close>

        <form
                onsubmit={(e) => {e.preventDefault(); handleAddTag();}}
                class="flex w-full items-center space-x-2"
        >
            <Input bind:value={newTagName}
                   placeholder="Enter new tag name"
                   class="flex-grow"
                   disabled={isLoading}
            />
            <Button type="submit" disabled={isLoading ||!newTagName.trim()}
                    class="cursor-pointer">
                {isLoading ? "Adding..." : "Add"}
            </Button>
        </form>

        {#if errorMessage}
            <p class="-mt-4 text-destructive">{errorMessage}</p>
        {/if}

        <div class="-mt-2 flex flex-wrap gap-2 border-t pt-2">
            {#if isLoading && tags.length === 0}
                <p class="text-gray-500 text-sm">Loading tags...</p>
            {:else if sortedTags.length > 0}
                {#each sortedTags as tag (tag.id)}
                    <Badge variant="secondary"
                           class="flex items-center gap-1.5 pl-2.5 pr-1 text-base">
                        <span>{tag.name}</span>
                        <button onclick={() => handleDeleteTag(tag.id)}
                                class="rounded-full p-0.5 text-muted-foreground transition-colors hover:bg-background/50 hover:text-foreground"
                                aria-label="Delete tag {tag.name}"
                                disabled="{isLoading}">
                            <X class="h-3 w-3"/>
                        </button>
                    </Badge>
                {/each}
            {:else if !isLoading}
                <p class="text-sm text-muted-foreground">No tags found. Add one.</p>
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>
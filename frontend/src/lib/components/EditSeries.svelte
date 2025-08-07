<script lang="ts">
    import ModalDialog from "$lib/components/ModalDialog.svelte";
    import {Label} from "$lib/components/ui/label";
    import {Input} from "$lib/components/ui/input";
    import {apiFetch} from "$lib/store/auth";
    import {toast} from "svelte-sonner";

    type Series = {
        id: number;
        title: string;
        originalTitle: string | null;
        authors: string[];
        description: string;
        coverImageUrl: string;
        sourceUrl: string;
    };

    let {series, onclose}: { series: Series, onclose: () => void } = $props();

    let formData = $state({
        title: series.title,
        originalTitle: series.originalTitle ?? '',
        authors: series.authors.join(', '),
        description: series.description,
        coverImageUrl: series.coverImageUrl,
        sourceUrl: series.sourceUrl,
    });

    let isSubmitting = $state(false);
    let errorMessage = $state<string | null>(null);
    let open = $state(true);

    $effect(() => {
        if (!open) {
            onclose();
        }
    });

    async function updateSeriesRequest() {
        // Prepare the payload for the backend API, matching the `UpdateSeriesRequest` struct
        const payload = {
            ...formData,
            authors: formData.authors.split(', ').map(author => author.trim()).filter(Boolean),
        };
        const response = await apiFetch(`/api/admin/series/update/${series.id}`, {
            method: 'PUT',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify(payload),
        });
        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.message || 'Failed to update series');
        }
        return series.title;
    }

    async function handleSubmit() {
        if (isSubmitting) return;
        isSubmitting = true;

        toast.promise(updateSeriesRequest(), {
            position: "top-center",
            richColors: true,
            closeButton: false,
            duration: 3000,
            class: "[--width:500px]",
            loading: 'Updating the series...',
            success: (title) => {
                onclose();
                return `Series "${title} has been successfully updated."`
            },
            error: (err) => {
                console.error("Failed to update series: ", err);
                const errorMessage = err instanceof Error ? err.message : 'Unknown error';
                return `Failed to update series: ${series.title}: ${errorMessage}`;
            },
            finally: () => {
                isSubmitting = false;
            },
        });
    }
</script>

<ModalDialog
        bind:open={open}
        title="Edit: {series.title}"
        onConfirm={handleSubmit}
        confirmText="Save Changes"
        disableConfirm={isSubmitting}
>
    {#snippet children()}
        <form class="space-y-4 pt-4 text-gray-800 dark:text-gray-100  max-h-[75vh] overflow-y-auto pr-3">
            <div>
                <Label for="title"
                       class="block text-sm font-medium text-gray-700 dark:text-gray-100">Title</Label>
                <Input id="title" type="text" bind:value={formData.title} required/>
            </div>
            <div>
                <Label for="originalTitle"
                       class="block text-sm font-medium text-gray-700 dark:text-gray-100">Original
                    Title
                    (optional)</Label>
                <Input id="originalTitle" type="text"
                       bind:value={formData.originalTitle}/>
            </div>
            <div>
                <Label for="authors" class="block text-sm font-medium text-gray-700">Authors
                    (comma-separated)</Label>
                <Input id="authors" type="text" bind:value={formData.authors}/>
            </div>
            <div>
                <Label for="description" class="block text-sm font-medium text-gray-700">Description</Label>
                <textarea id="description" bind:value={formData.description}
                          class="flex w-full rounded-md text-sm border border-input "
                          rows="4"> </textarea>
            </div>
            <div>
                <Label for="coverImageUrl"
                       class="block text-sm font-medium text-gray-700">Cover Image
                    URL</Label>
                <Input id="coverImageUrl" type="text"
                       bind:value={formData.coverImageUrl}/>
            </div>
            <div>
                <Label for="sourceUrl" class="block text-sm font-medium text-gray-700">Source
                    URL (optional)</Label>
                <Input id="sourceUrl" type="text" bind:value={formData.sourceUrl}/>
            </div>

            {#if errorMessage}
                <div class="text-sm text-red-500 bg-red-50 p-3 rounded-md">
                    {errorMessage}
                </div>
            {/if}
        </form>
    {/snippet}
</ModalDialog>
<script lang="ts">
    import ModalDialog from "$lib/components/ModalDialog.svelte";
    import {Label} from "$lib/components/ui/label";
    import {Input} from "$lib/components/ui/input";
    import {Button} from "$lib/components/ui/button";
    import {apiFetch} from "$lib/store/auth";
    import {toast} from "svelte-sonner";
    import {Badge} from "$lib/components/ui/badge/index.js";

    type Series = {
        id: number;
        title: string;
        originalTitle: string | null;
        authors: string[];
        description: string;
        coverImageUrl: string;
        sourceUrl: string;
    };

    type CategoryTags = {
        id: number;
        name: string;
    };

    let {series, onclose}: { series: Series, onclose: () => void } = $props();
    let isSubmitting = $state(false);
    let errorMessage = $state<string | null>(null);
    let open = $state(true);
    let fileInput: HTMLInputElement;
    let availableTags = $state<CategoryTags[]>([]);
    let selectedTagsIds = $state<Set<number>>(new Set());

    let formData = $state({
        title: series.title,
        originalTitle: series.originalTitle ?? '',
        authors: series.authors.join(', '),
        description: series.description,
        coverImageUrl: series.coverImageUrl,
        sourceUrl: series.sourceUrl,
    });

    async function fetchTags() {
        try {
            const allTagsResponse = await apiFetch('/api/admin/category/tag /list');
            if (!allTagsResponse.ok) throw new Error('Failed to fetch category tags');
            const allTagsData = await allTagsResponse.json();
            availableTags = allTagsData.categories || [];

            const seriesTagsResponse = await apiFetch(`/api/admin/series/tags/${series.id}`);
            if (!seriesTagsResponse.ok) throw new Error('Failed to fetch series tags');
            const seriesTagsData = await seriesTagsResponse.json();
            const currentTagIds = (seriesTagsData.tags || []).map((tag: CategoryTags) => tag.id);
            selectedTagsIds = new Set(currentTagIds);
        } catch (error: any) {
            toast.error('Could not load tag information.', {description: error.message});
            console.error(error);
        }
    }

    function toggleTagSelection(tagId: number) {
        const newSelectedTagIds = new Set(selectedTagsIds);
        if (newSelectedTagIds.has(tagId)) {
            newSelectedTagIds.delete(tagId);
        } else {
            newSelectedTagIds.add(tagId);
        }
        selectedTagsIds = newSelectedTagIds;
    }

    async function uploadCoverImage(file: File) {
        const uploadRequest = async () => {
            const data = new FormData();
            data.append("file", file);

            const response = await apiFetch('/api/admin/series/cover/upload/image', {
                method: "POST",
                body: data,
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to upload cover image.');
            }

            const result = await response.json();
            return result.url;
        };

        toast.promise(uploadRequest(), {
            position: "top-center",
            richColors: true,
            closeButton: false,
            duration: 3000,
            loading: `Uploading ${file.name}`,
            success: (newUrl) => {
                formData.coverImageUrl = newUrl;
                return "Cover image uploaded successfully.";
            },
            error: (err) => {
                const message = err instanceof Error ? err.message : 'Unknown upload error.';
                return `Upload image failed: ${message}`;
            },
        });
    }

    function handleFileSelected(event: Event) {
        const input = event.target as HTMLInputElement;
        if (input.files && input.files[0]) {
            const file = input.files[0];
            uploadCoverImage(file);
        }
    }

    async function updateSeriesRequest() {
        // Prepare the payload for the backend API, matching the `UpdateSeriesRequest` struct
        const payload = {
            title: formData.title,
            originalTitle: formData.originalTitle || null,
            authors: formData.authors.split(', ').map(author => author.trim()).filter(Boolean),
            description: formData.description,
            coverImageUrl: formData.coverImageUrl,
            sourceUrl: formData.sourceUrl,
            categoryIds: Array.from(selectedTagsIds)
        };
        const response = await apiFetch(`/api/admin/series/update/${series.id}`, {
            method: 'PATCH',
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

    // Fetch data when the modal opens
    $effect(() => {
        if (open) {
            fetchTags();
        }
    });

    $effect(() => {
        if (!open) {
            onclose();
        }
    });
</script>

<ModalDialog
        bind:open={open}
        title="Edit: {series.title}"
        onConfirm={handleSubmit}
        confirmText="Save Changes"
        disableConfirm={isSubmitting}
        dialogClass="sm:max-w-2xl lg:max-w-4xl"
>
    {#snippet children()}
        <form>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-8 py-4 max-h-[75vh] overflow-y-auto pr-3 custom-scrollbar">
                <div class="md:col-span-2 flex flex-col gap-4">
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
                        <Label for="authors" class="block text-sm font-medium text-gray-700 dark:text-gray-100">Authors
                            (comma-separated)</Label>
                        <Input id="authors" type="text" bind:value={formData.authors}/>
                    </div>
                    <div>
                        <Label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-100">Description</Label>
                        <textarea id="description" bind:value={formData.description}
                                  class="flex w-full rounded-md text-sm border border-input "
                                  rows="4"> </textarea>
                    </div>
                    <div>
                        <Label for="sourceUrl" class="block text-sm font-medium text-gray-700 dark:text-gray-100">Source
                            URL</Label>
                        <Input id="sourceUrl" type="text" bind:value={formData.sourceUrl}/>
                    </div>
                    <div class="grid w-full items-center gap-1.5">
                        <Label>Tags</Label>
                        {#if availableTags.length > 0}
                            <div class="flex flex-wrap gap-2 pt-2">
                                {#each availableTags as tag (tag.id)}
                                    <Badge role="button"
                                           variant={selectedTagsIds.has(tag.id) ? 'default' : 'secondary'}
                                           class="cursor-pointer"
                                           onclick={() => toggleTagSelection(tag.id)}
                                           onkeydown={(e) => {
                                               if (e.key === 'Enter' || e.key === ' ') {
                                                   e.preventDefault();
                                                   toggleTagSelection(tag.id);
                                               }
                                           }}
                                    >
                                        {tag.name}
                                    </Badge>
                                {/each}
                            </div>
                        {:else}
                            <p class="text-sm text-muted-foreground pt-2">
                                No Category tags available. Add them in 'Manage Tags'
                            </p>
                        {/if}
                    </div>
                </div>

                <div class="md:col-span-1 flex flex-col gap-1.5">
                    <Label for="coverImageUrl"
                           class="block text-sm font-medium text-gray-700 dark:text-gray-100">Cover Image</Label>
                    <div class="aspect-[3/4] w-full">
                        <img src={formData.coverImageUrl} alt="Cover Preview"
                             class="w-[200px] h-[280px] object-cover rounded-md border"/>
                        <input id="coverImageUrl" type="file"
                               bind:this={fileInput}
                               class="hidden"
                               onchange={handleFileSelected}
                               accept="image/png, image/jpeg, image/webp, image/avif"
                        />
                        <Button class="mt-3 px-1" size="sm"
                                type="button" onclick={() => fileInput.click()}>
                            Choose new cover image
                        </Button>
                    </div>
                </div>
            </div>


            {#if errorMessage}
                <div class="text-sm text-red-500 bg-red-50 p-3 rounded-md">
                    {errorMessage}
                </div>
            {/if}
        </form>
    {/snippet}
</ModalDialog>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 5px;
    }

    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }

    .custom-scrollbar::-webkit-scrollbar-thumb {
        background-color: rgba(107, 114, 128, 0.5);
        border-radius: 10px;
        border: 3px solid transparent;
    }
</style>
<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import {Label} from "$lib/components/ui/label/index.js"
    import {Input} from "$lib/components/ui/input/index.js";
    import {Button} from "$lib/components/ui/button/index.js";
    import ModalDialog from "$lib/components/ModalDialog.svelte";
    import {X, Plus, Minus} from "@lucide/svelte";
    import {apiFetch} from "$lib/store/auth";
    import {toast} from "svelte-sonner";
    import {Badge} from "$lib/components/ui/badge";

    type CategoryTag = {
        id: number;
        name: string;
    };

    let title = $state('');
    let originalTitle = $state('');
    let description = $state('');
    let authors = $state([{id: Date.now(), name: ''}]);
    let sourceUrl = $state('');
    let coverImageFile = $state<File | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);
    let isSubmitting = $state(false);
    let isDragging = $state(false);
    let open = $state(false);
    let notification = $state<{
        message: string;
        type: 'success' | 'error'
    } | null>(null);
    let availableTags = $state<CategoryTag[]>([]);
    let selectedTagIds = $state<Set<number>>(new Set());

    let {children} = $props();

    let coverPreviewUrl = $derived(coverImageFile ? URL.createObjectURL(coverImageFile) : null);

    function resetForm() {
        title = '';
        originalTitle = '';
        description = '';
        authors = [{id: Date.now(), name: ''}];
        sourceUrl = '';
        coverImageFile = null;
        selectedTagIds = new Set();
        if (fileInput) {
            fileInput.value = '';
        }
    }

    function handleFileChange(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            coverImageFile = target.files[0];
        }
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        isDragging = false;
        if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
            coverImageFile = event.dataTransfer.files[0];
            if (fileInput) {
                fileInput.files = event.dataTransfer.files;
            }
        }
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault();
        isDragging = true;
    }

    function handleDragLeave() {
        isDragging = false;
    }

    function removeImage() {
        coverImageFile = null;
        if (fileInput) {
            fileInput.value = '';
        }
    }

    function addAuthor() {
        authors = [...authors, {id: Date.now(), name: ''}];
        // authors.push({id: Date.now(), name: ''});
    }

    function removeAuthor(id: number) {
        authors = authors.filter(author => author.id !== id);
    }

    // Fetch available category tags from API
    async function fetchTags() {
        try {
            const response = await apiFetch('/api/admin/category/tag/list');
            if (!response.ok) {
                throw new Error('Failed to fetch category tags');
            }
            const data = await response.json();
            availableTags = data.categories || [];
        } catch (error: any) {
            toast.error("Could not load category tags. Please try again.", {description: error.message});
            console.error(error);
        }
    }

    $effect(() => {
        if (open) {
            if (availableTags.length === 0) {
                fetchTags();
            }
        }
    });

    function toggleTagSelection(tagId: number) {
        const newSelectedTagIds = new Set(selectedTagIds);
        if (newSelectedTagIds.has(tagId)) {
            newSelectedTagIds.delete(tagId);
        } else {
            newSelectedTagIds.add(tagId);
        }
        selectedTagIds = newSelectedTagIds
    }

    $effect(() => {
        return () => {
            if (coverPreviewUrl) {
                URL.revokeObjectURL(coverPreviewUrl);
            }
        }
    })

    // Api Interaction
    async function uploadCoverImage(file: File): Promise<string> {
        const formData = new FormData();
        formData.append('file', file);

        const response = await apiFetch('/api/admin/series/cover/upload/image', {
            method: 'POST',
            body: formData,
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() =>
                ({message: 'Failed to upload cover image'}));
            throw new Error(errorData.message);
        }

        const result = await response.json();
        return result.url;
    }

    async function createSeriesRequest() {
        if (!title || !coverImageFile || !description || !sourceUrl) {
            throw new Error("Please fill in all required fields.");
        }

        const uploadedCoverUrl = await uploadCoverImage(coverImageFile);
        const payload = {
            title,
            original_title: originalTitle || null,
            authors: authors.map(a => a.name).filter(name => name.trim() !== ''),
            description,
            cover_image_url: uploadedCoverUrl,
            source_url: sourceUrl,
            category_ids: Array.from(selectedTagIds),
        };
        const response = await apiFetch('/api/admin/series/add', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(payload),
        });
        if (!response.ok) {
            const errorData = await response.json().catch(() =>
                ({message: 'Failed to add series. Unknown error occurred.'}));
            throw new Error(errorData.message);
        }

        const result = await response.json();
        return {id: result.id, title: title};
    }

    async function handleAddSeries() {
        isSubmitting = true;

        toast.promise(createSeriesRequest(), {
            position: "top-center",
            richColors: true,
            duration: 3000,
            loading: 'Adding new series...',
            success: (data) => {
                open = false;
                resetForm();
                return `Series [${data.id}] "${data.title}"  has been created successfully!`;
            },
            error: (err) => {
                console.error("Failed to add series: ", err);
                const errorMessage = err instanceof Error ? err.message : 'Unknown error';
                return `Failed to add series: ${errorMessage} || Please try again later.`;
            },
            finally: () => {
                isSubmitting = false;
            },
        });
    }
</script>


<form class="flex flex-col space-y-3">
    <ModalDialog
            bind:open={open}
            title="Add New Series"
            confirmText={isSubmitting ? "Creating..." : "Create Series"}
            onConfirm={handleAddSeries}
            dialogClass="sm:max-w-2xl lg:max-w-4xl"
            disableConfirm={isSubmitting}
    >
        {#snippet trigger()}
            {#if children}
                {@render children()}
            {:else}
                <Button class="cursor-pointer">
                    Add Series
                </Button>
            {/if}
        {/snippet}

        {#snippet children()}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-8 py-4 max-h-[80vh] overflow-y-auto pr-3 custom-scrollbar">
                <!-- Left Column Data -->
                <div class="md:col-span-2 flex flex-col gap-4">
                    <div class="grid w-full items-center gap-1.5">
                        <Label for="seriesName">Series Title <span
                                class="text-red-500">*</span> </Label>
                        <Input id="seriesName" bind:value={title}/>
                    </div>
                    <div class="grid w-full items-center gap-1.5">
                        <Label for="originalName">Original Title</Label>
                        <Input id="originalName" bind:value={originalTitle}
                               placeholder="Optional"/>
                    </div>
                    <div class="grid w-full items-center gap-1.5">
                        <Label for="author" class="text-right">Author</Label>
                        <div class="col-span-3 flex flex-col gap-2">
                            {#each authors as author, index (author.id)}
                                <div class="flex items-center gap-2">
                                    <Input bind:value={author.name}/>
                                    {#if index === 0}
                                        <Button onclick={addAuthor} size="icon"
                                                type="button" aria-label="Add Author">
                                            <Plus class="h-4 w-4"/>
                                        </Button>
                                    {:else}
                                        <Button onclick={() => removeAuthor(author.id)}
                                                size="icon" type="button"
                                                aria-label="Remove Author">
                                            <Minus class="h-4 w-4"/>
                                        </Button>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    </div>
                    <div class="grid w-full items-center gap-1.5">
                        <Label for="description"
                               class="text-right">Description <span
                                class="text-red-500">*</span> </Label>
                        <textarea id="description" bind:value={description}
                                  class="flex min-h-[150px] w-full rounded-md border-input px-3 py-3 text-sm"
                                  placeholder="Series Description"></textarea>
                    </div>
                    <div class="grid w-full items-center gap-1.5">
                        <Label for="sourceUrl">Source Url <span
                                class="text-red-500">*</span> </Label>
                        <Input id="sourceUrl" type="url" bind:value={sourceUrl}/>
                    </div>
                    <div class="grid w-full items-center gap-1.5">
                        <Label>Tags</Label>
                        {#if availableTags.length > 0}
                            <div class="flex flex-wrap gap-2 pt-2">
                                {#each availableTags as tag (tag.id)}
                                    <Badge variant={selectedTagIds.has(tag.id) ? 'default' : 'secondary'}
                                           class="cursor-pointer"
                                           role="button"
                                           onclick={() => toggleTagSelection(tag.id)}
                                           onkeydown={(e) => {
                                               if (e.key === 'Enter' || e.key === '') {
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
                            <p class="text-sm text-muted-foreground pt-2">No category tags available. Add them in
                                'Manage Tags'.</p>
                        {/if}
                    </div>
                </div>

                <div class="md:col-span-1 flex flex-col gap-1.5">
                    <Label>Cover Image <span class="text-red-500">*</span> </Label>
                    <div class="aspect-[3/4] w-full" ondragover={handleDragOver}
                         ondrop={handleDrop} ondragleave={handleDragLeave}
                         role="button" tabindex="0">
                        {#if coverPreviewUrl}
                            <div class="relative h-full w-full">
                                <img
                                        src={coverPreviewUrl}
                                        alt="Cover preview"
                                        class="h-full w-full rounded-md object-cover"
                                />
                                <Button
                                        onclick={removeImage}
                                        variant="destructive"
                                        size="icon"
                                        class="absolute right-2 top-2 h-4 w-4 rounded-full cursor-pointer"
                                        aria-label="Remove image"
                                >
                                    <X class="h-4 w-4"/>
                                </Button>
                            </div>
                        {:else}
                            <Label
                                    for="coverImageInput"
                                    class="flex h-full w-full cursor-pointer
                                        flex-col items-center justify-center rounded-md
                                        border-2 border-dashed border-input bg-background
                                        hover:bg-accent {isDragging ? 'border-blue-500' : ''}"
                            >
                                    <span class="text-sm text-muted-foreground"
                                          class:text-blue-500={isDragging}>
                                        {#if isDragging}
                                            Drop file here
                                        {:else}
                                            Choose file
                                        {/if}
                                    </span>
                            </Label>
                            <input
                                    bind:this={fileInput}
                                    onchange={handleFileChange}
                                    id="coverImageInput"
                                    type="file"
                                    class="hidden"
                                    accept="image/png, image/jpeg, image/webp, image/avif"
                                    required
                            />
                        {/if}
                    </div>
                </div>
            </div>
            {#if notification}
                <div class="mt-2 text-sm p-3 rounded-md"
                     class:bg-red-50={notification.type === 'error'}
                     class:text-red-700={notification.type === 'error'}
                     class:bg-green-50={notification.type ==='success'}
                     class:text-green-700={notification.type ==='success'}
                >
                    {notification.message}
                </div>
            {/if}
        {/snippet}
    </ModalDialog>
</form>

<style>
    /*
    .add-manga {
        background-color: #3b82f6;
        color: #ffffff;
        font-weight: 700;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
    }

    .add-manga:hover {
        background-color: #2563eb;
    }*/

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
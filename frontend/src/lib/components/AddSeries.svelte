<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import {Label} from "$lib/components/ui/label/index.js"
    import {Input} from "$lib/components/ui/input/index.js";
    import {Button} from "$lib/components/ui/button/index.js";
    import ModalDialog from "$lib/components/ModalDialog.svelte";
    import {X, Plus, Minus} from "@lucide/svelte";
    import {apiFetch} from "$lib/store/auth";

    let title = $state('');
    let originalTitle = $state('');
    let description = $state('');
    let authors = $state([{id: Date.now(), name: ''}]);
    let sourceUrl = $state('');
    // let host = $state('');
    let coverImageFile = $state<File | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);
    let isSubmitting = $state(false);
    let isDragging = $state(false);
    let open = $state(false);
    let notification = $state<{
        message: string;
        type: 'success' | 'error'
    } | null>(null);

    let {children} = $props();

    let coverPreviewUrl = $derived(coverImageFile ? URL.createObjectURL(coverImageFile) : null);

    function resetForm() {
        title = '';
        originalTitle = '';
        description = '';
        authors = [{id: Date.now(), name: ''}];
        sourceUrl = '';
        coverImageFile = null;
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

        const response = await apiFetch('/api/admin/upload/image', {
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

    async function handleAddSeries() {
        isSubmitting = true;
        notification = null;

        if (!title || !coverImageFile || !description || !sourceUrl) {
            alert("Please fill in all required fields.");
            isSubmitting = false;
            return;
        }

        try {
            const uploadedCoverUrl = await uploadCoverImage(coverImageFile);

            const payload = {
                title,
                original_title: originalTitle || null,
                authors: authors.map(a => a.name).filter(name => name.trim() !== ''),
                description,
                cover_image_url: uploadedCoverUrl,
                source_url: sourceUrl,
            };

            const response = await apiFetch('/api/admin/series/add', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload),
            });

            if (response.ok) {
                const result = await response.json();
                notification = {
                    message: `Series added with ID: ${result.id}, Title: ${title}`,
                    type: 'success'
                };

                setTimeout(() => {
                    open = false;
                }, 3000);

                resetForm();
                // TODO: Close the modal and refresh the series list automatically
            } else {
                const errorData = await response.json().catch(() =>
                    ({message: 'Failed to add series. Unknown error occurred.'}));
                notification = {message: errorData.message, type: 'error'};
            }
        } catch (error) {
            alert((error as Error).message);
            console.error("Failed to add series: ", error);
        } finally {
            isSubmitting = false;
        }
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
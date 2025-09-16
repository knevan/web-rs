<script lang="ts">
    import {Textarea} from "$lib/components/ui/textarea";
    import {Button} from "$lib/components/ui/button";
    import {mountSpoilers, parseAndSanitize} from "$lib/utils/markdown";
    import {tick} from "svelte";
    import type {User} from "$lib/store/auth";
    import {page} from "$app/state";
    import {goto} from "$app/navigation";
    import {LogIn, Bold, Italic, Link, EyeClosed, ImagePlus} from "@lucide/svelte";
    import {toast} from "svelte-sonner";

    let {
        submitComment,
        placeholder = 'Send your comment...',
        submitLabel = 'Send',
        initialContent = '',
        currentUser = null,
        onCancel,
        cancelLabel = 'Cancel',
    } = $props<{
        submitComment: (formData: FormData) => void;
        placeholder?: string;
        submitLabel?: string;
        initialContent?: string;
        currentUser?: User | null;
        onCancel?: () => void;
        cancelLabel?: string;
    }>();

    // State for the textarea content
    let contentText = $state(initialContent);
    let activeTab = $state('write');
    let textareaElement = $state<HTMLTextAreaElement | null>(null);
    let previewContainer = $state<HTMLElement | null>(null);
    let fileInputElement = $state<HTMLInputElement | null>(null);
    let selectedFile = $state<File | null>(null);
    let previewUrl = $state<string | null>(null);

    const isLoggedIn = $derived(!!currentUser);
    const previewComment = $derived(
        parseAndSanitize(contentText),
    )

    // Effect for spoiler
    $effect(() => {
        if (previewContainer && previewComment) {
            // We need to wait for Svelte to render the new HTML from previewComment
            tick().then(() => {
                mountSpoilers(previewContainer);
            });
        }
    });

    // Effect for image preview
    $effect(() => {
        if (selectedFile) {
            const url = URL.createObjectURL(selectedFile);
            previewUrl = url;

            return () => {
                URL.revokeObjectURL(url);
            };
        }
    });

    // Make template wrap markdown
    async function wrapSelection(prefix: string, suffix: string) {
        if (!textareaElement) return;

        const start = textareaElement.selectionStart;
        const end = textareaElement.selectionEnd;

        // Get the parts of the text: before the selection, the selection itself, and after.
        const before = contentText.substring(0, start);
        const after = contentText.substring(end);
        const selectedText = contentText.substring(start, end);

        let finalSelectionStart;
        let finalSelectionEnd;

        if (before.endsWith(prefix) && after.startsWith(suffix)) {
            // If it is, unwrap it by removing the prefix and suffix from the surrounding text.
            const newBefore = before.slice(0, before.length - prefix.length);
            const newAfter = after.slice(suffix.length);
            contentText = newBefore + selectedText + newAfter;

            // Adjust the final selection to cover the now-unwrapped text.
            finalSelectionStart = start - prefix.length;
            finalSelectionEnd = end - prefix.length;
        } else {
            // If not wrapped, apply the wrapping.
            const newText = prefix + selectedText + suffix;
            contentText = before + newText + after;

            // Set cursor position based on whether text was selected or not.
            if (selectedText) {
                // If text was selected, keep it selected along with the new wrappers.
                finalSelectionStart = start;
                finalSelectionEnd = end + prefix.length + suffix.length;
            } else {
                // If no text was selected, place the cursor inside the wrappers.
                finalSelectionStart = finalSelectionEnd = start + prefix.length;
            }
        }
        await tick();

        // Re-focus the textarea and set the calculated selection range.
        textareaElement.focus();
        textareaElement.setSelectionRange(finalSelectionStart, finalSelectionEnd);
    }

    function handleSend() {
        if (!contentText.trim() && !selectedFile) return;

        // Gunakan FormData untuk mengirim data multipart
        const formData = new FormData();
        formData.append('content_markdown', contentText);

        if (selectedFile) {
            formData.append('attachment', selectedFile);
        }

        submitComment(formData);
        contentText = '';
        activeTab = 'write';
        removeSelectedImage();
    }

    function handleSelectImage() {
        fileInputElement?.click();
    }

    function handleFileSelected(event: Event) {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];

        if (file) {
            if (file.size > 5 * 1024 * 1024) {
                toast.warning('File size cannot exceed 5MB.', {
                    position: "top-center",
                    closeButton: false,
                    duration: 3000,
                })
                return;
            }
            selectedFile = file;
        }
    }

    function removeSelectedImage() {
        selectedFile = null;
        previewUrl = null;
        if (fileInputElement) {
            fileInputElement.value = '';
        }
    }

    function handleLoginClick() {
        const redirectTo = page.url.pathname;
        goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
    }
</script>

<div class="relative">
    {#if !isLoggedIn}
        <div class="absolute inset-0 z-10 flex items-center justify-center rounded-md  bg-gray-100/70 dark:bg-gray-900/70">
            <Button
                    onclick={handleLoginClick}
                    class="cursor-pointer p-2 bg-transparent dark:border dark:border-gray-600 shadow-md
                       hover:scale-105 transition-transform duration-200 ease-in-out
                       flex items-center justify-center"
                    variant="outline"
                    size="lg"
            >
                <LogIn class="!h-6 !w-6 text-gray-700 dark:text-gray-200"/>
            </Button>
        </div>
    {/if}
    <div class="flex flex-col gap-2">
	<Textarea
            bind:ref={textareaElement}
            bind:value={contentText}
            {placeholder}
            rows={4}
            class="w-full wrap-normal min-h-[100px] whitespace-normal rounded-md border border-zinc-300 bg-transparent p-3 text-base text-gray-800 dark:text-gray-200 transition-colors"
    />
        {#if contentText}
            <div class="max-w-none rounded-md border border-dashed border-gray-300 bg-gray-50 p-2 dark:border-gray-700 dark:bg-gray-800">
                <h4 class="text-sm font-semibold text-gray-500">
                    Preview
                </h4>
                <div class="flex flex-col gap-2">
                    {#if contentText.trim()}
                        <div bind:this={previewContainer}
                             class="prose prose-a:text-blue-500 dark:prose-invert max-w-none wrap-normal">
                            {@html previewComment}
                        </div>
                    {/if}

                    {#if previewUrl}
                        <div class="relative w-fit max-w-xs rounded-md">
                            <img src={previewUrl} alt="Image preview" class="max-h-40 rounded-md object-contain"/>
                            <Button
                                    onclick={removeSelectedImage}
                                    variant="destructive"
                                    size="icon"
                                    class="absolute -right-2 -top-2 h-6 w-6 rounded-full shadow-md"
                                    aria-label="Remove image"
                            >
                                &times;
                            </Button>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}

        <div class="flex items-center justify-between">
            <div class="flex items-center gap-1">
                <Button onclick={() => wrapSelection('**', '**')}
                        variant="outline"
                        size="iconLabel"
                        class="font-bold"
                        aria-label="Bold"
                >
                    <Bold/>
                </Button>
                <Button onclick={() => wrapSelection('*', '*')}
                        variant="outline"
                        size="iconLabel"
                        class="italic"
                        aria-label="Italic"
                >
                    <Italic/>
                </Button>
                <Button onclick={() => wrapSelection('[', '](url)')}
                        variant="outline"
                        size="iconLabel"
                        class="italic"
                        aria-label="Italic"
                >
                    <Link/>
                </Button>
                <Button onclick={() => wrapSelection('||', '||')}
                        variant="outline"
                        size="iconLabel"
                        aria-label="Spoiler"
                >
                    <EyeClosed/>
                </Button>
                <input
                        type="file"
                        bind:this={fileInputElement}
                        onchange={handleFileSelected}
                        accept="image/png, image/jpeg, image/gif"
                        class="hidden"
                />
                <Button onclick={handleSelectImage}
                        variant="outline"
                        size="iconLabel"
                        class="italic"
                        aria-label="Italic"
                >
                    <ImagePlus/>
                </Button>
            </div>

            <div class="flex items-center gap-2">
                {#if onCancel}
                    <Button variant="destructive" size="sm" onclick={onCancel} class="text-sm font-medium">
                        {cancelLabel}
                    </Button>
                {/if}
                <Button onclick={handleSend} size="sm"
                        class="cursor-pointer rounded-md  bg-blue-600 px-4 py-2 font-bold text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50">
                    {submitLabel}
                </Button>
            </div>
        </div>
    </div>
</div>
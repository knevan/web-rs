<script lang="ts">
    import {Textarea} from "$lib/components/ui/textarea";
    import {Button} from "$lib/components/ui/button";
    import {mountSpoilers, parseAndSanitize} from "$lib/utils/markdown";
    import {tick} from "svelte";

    let {
        submitText,
        placeholder = 'Send your comment...',
        submitLabel = 'Send'
    } = $props();

    // State for the textarea content
    let contentText = $state('');
    let activeTab = $state('write');
    let textareaElement = $state<HTMLTextAreaElement | null>(null);
    let previewContainer = $state<HTMLElement | null>(null);

    const previewComment = $derived(
        parseAndSanitize(contentText),
    )

    $effect(() => {
        if (previewContainer) {
            // We need to wait for Svelte to render the new HTML from previewComment
            tick().then(() => {
                mountSpoilers(previewContainer);
            });
        }
    });

    // Function to handle form submission
    function handleSend() {
        if (!contentText.trim()) return;
        submitText(contentText);
        contentText = '';
        activeTab = 'write';
    }

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
</script>

<div class="flex flex-col gap-2">
	<Textarea
            bind:ref={textareaElement}
            bind:value={contentText}
            {placeholder}
            rows={4}
            class="w-full wrap-normal whitespace-normal rounded-md border border-zinc-300 bg-transparent p-3 text-base text-gray-800 transition-colors"
    />

    {#if contentText}
        <div class="max-w-none rounded-md border border-dashed border-gray-300 bg-gray-50 p-3 dark:border-gray-700 dark:bg-gray-800">
            <h4 class="text-sm font-semibold text-gray-500">
                Preview
            </h4>
            <div bind:this={previewContainer} class="max-w-none wrap-normal">
                {@html previewComment}
            </div>
        </div>
    {/if}

    <div class="flex items-center justify-between">
        <div class="flex items-center gap-1">
            <Button onclick={() => wrapSelection('**', '**')}
                    variant="outline"
                    size="sm"
                    class="font-bold"
                    aria-label="Bold"
            >
                B
            </Button>
            <Button onclick={() => wrapSelection('*', '*')}
                    variant="outline"
                    size="sm"
                    class="italic"
                    aria-label="Italic"
            >
                I
            </Button>
            <Button onclick={() => wrapSelection('||', '||')}
                    variant="outline"
                    size="sm"
                    aria-label="Spoiler"
            >
                Spoiler
            </Button>
        </div>
        <Button onclick={handleSend}
                class="cursor-pointer rounded-md  bg-blue-600 px-4 py-2 font-bold text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50">
            {submitLabel}
        </Button>
    </div>
</div>
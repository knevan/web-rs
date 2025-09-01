<script lang="ts">
    import {marked} from "marked";
    import DOMPurify from "dompurify";
    import {Textarea} from "$lib/components/ui/textarea";
    import {Button} from "$lib/components/ui/button";

    let {
        submitText,
        placeholder = 'Send your comment...',
        submitLabel = 'Send'
    } = $props();

    // State for the textarea content
    let contentText = $state('');
    let activeTab = $state('write');

    const previewComment = $derived(
        DOMPurify.sanitize(marked.parse(contentText, {breaks: true, gfm: true}) as string),
    )

    // Function to handle form submission
    function handleSend() {
        if (!contentText.trim()) return;
        submitText(contentText);
        contentText = '';
        activeTab = 'write';
    }
</script>

<div class="flex flex-col gap-2">
	<Textarea
            bind:value={contentText}
            {placeholder}
            class="w-full wrap-normal whitespace-normal rounded-md border border-zinc-300 bg-transparent p-3 text-base text-gray-800 transition-colors"
    />

    {#if contentText}
        <div class="max-w-none rounded-md border border-dashed border-gray-300 bg-gray-50 p-3 dark:border-gray-700 dark:bg-gray-800">
            <h4 class="text-sm font-semibold text-gray-500">
                Preview
            </h4>
            <div class="max-w-none wrap-normal">
                {@html previewComment}
            </div>
        </div>
    {/if}

    <div class="flex justify-end">
        <Button onclick={handleSend}
                class="cursor-pointer rounded-md bg-blue-600 px-4 py-2 font-bold text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50">
            {submitLabel}
        </Button>
    </div>
</div>
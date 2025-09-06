<script lang="ts">
    import CommentForm from './CommentForm.svelte';
    import CommentView from './CommentView.svelte';
    import {Button} from "$lib/components/ui/button";
    import {mountSpoilers, parseAndSanitize} from "$lib/utils/markdown";
    import {tick} from "svelte";

    let {comment, addReply} = $props();
    let showReplyForm = $state(false);
    let contentContainer = $state<HTMLElement | null>(null);

    const sanitizedContent = $derived(
        parseAndSanitize(comment.content),
    )

    function handleReplySubmit(contentText: string) {
        addReply(comment.id, contentText);
        showReplyForm = false;
    }

    $effect(() => {
        if (contentContainer) {
            // Use tick to ensure the DOM is ready
            tick().then(() => {
                mountSpoilers(contentContainer);
            });
        }
    });
</script>

<div class="flex flex-col">
    <div class="flex gap-3">
        <img src={comment.user.avatar} alt="User Avatar" class="mt-1 h-9 w-9 rounded-full"/>
        <div class="flex-1">
            <div class="flex items-center gap-2">
                <span class="font-bold text-zinc-800 dark:text-zinc-100">{comment.user.name}</span>
                <span class="text-sm text-zinc-500 dark:text-zinc-400">{comment.timestamp}</span>
            </div>

            <div bind:this={contentContainer} class="prose prose-zinc mt-1 max-w-none dark:prose-invert">
                {@html sanitizedContent}
            </div>

            {#if comment.imageUrl}
                <img src={comment.imageUrl} alt="User content" class="mt-2 max-h-[250px] max-w-full rounded-lg"/>
            {/if}

            <div class="comment-actions">
                <Button class="text-sm bg-transparent text-gray-700 dark:text-gray-200">
                    👍 {comment.upvotes}</Button>
                <Button class="text-sm bg-transparent text-gray-700 dark:text-gray-200">👎</Button>
                <Button class="text-sm font-medium bg-transparent text-gray-700 dark:text-gray-200"
                        onclick={() => (showReplyForm = !showReplyForm)}>
                    Reply to user
                </Button>
            </div>
        </div>
    </div>

    {#if showReplyForm}
        <div class="mt-3 pl-12">
            <CommentForm
                    submitText={handleReplySubmit}
                    placeholder={`Reply to ${comment.user.name}`}
                    submitLabel="Reply Send"
            />
        </div>
    {/if}

    {#if comment.replies?.length > 0}
        <div class="mt-4 border-l-2 border-zinc-200 pl-6 ml-[18px]">
            {#each comment.replies as reply (reply.id)}
                <CommentView comment={reply} {addReply}/>
            {/each}
        </div>
    {/if}
</div>
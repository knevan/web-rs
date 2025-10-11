<script lang="ts">
    import CommentForm from './CommentForm.svelte';
    import CommentView from './CommentView.svelte';
    import {Button} from "$lib/components/ui/button";
    import {mountSpoilers} from "$lib/utils/markdown";
    import {tick} from "svelte";
    import {apiFetch} from "$lib/store/auth";
    import {UserRound} from "@lucide/svelte";
    import {ChevronUp, ChevronDown} from "@lucide/svelte";
    import ModalDialog from "$lib/components/ModalDialog.svelte";
    import type {CommentType} from "$lib/components/comments/comments";

    let {comment = $bindable(), addReply, currentUser, onUpdate} = $props<{
        comment: CommentType;
        addReply: (parentId: number, content: string) => void;
        currentUser: any;
        onUpdate: (updatedCommentData: Partial<CommentType>) => void;
    }>();
    let showReplyForm = $state(false);
    let contentContainer = $state<HTMLElement | null>(null);
    let isEditing = $state(false);
    let showLinkWarningModal = $state(false);
    let targetUrl = $state('');

    const isOwnerComment = $derived(currentUser?.id === comment.user.id);
    // const sanitizedContent = $derived(comment.content_html);

    const renderedContent = $derived.by(() => {
        let finalHtml = comment.content_html;

        // Check if there are attachment URLs and process them
        if (comment.attachment_urls && comment.attachment_urls.length > 0) {
            finalHtml = finalHtml.replace(/\[img:(\d+)]/g, (match: string, indexStr: string) => {
                const index = parseInt(indexStr, 10);
                if (comment.attachment_urls && comment.attachment_urls[index]) {
                    const url = comment.attachment_urls[index];
                    // Return an anchor tag wrapping an image for the attachment
                    return `
                        <a href="${url}" target="_blank" rel="noopener noreferrer">
                           <img src="${url}" alt="User attachment ${index}" class="mt-2 max-h-[250px] max-w-full rounded-lg object-contain" />
                        </a>
                    `;
                }
                return match; // Return the original placeholder if URL not found
            });
        }
        return finalHtml;
    });

    function handleReplySubmit(formData: FormData) {
        const contentText = formData.get('content_markdown') as string;
        addReply(comment.id, contentText);
        showReplyForm = false;
    }

    async function handleUpdateSubmit(formData: FormData) {
        try {
            const newMarkdown = formData.get('content_markdown') as string
            const response = await apiFetch(`/api/comments/${comment.id}/edit`, {
                method: "PATCH",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({content_markdown: newMarkdown})
            });
            if (!response.ok) {
                console.error(response);
                return;
            }
            const updatedComment = await response.json();
            onUpdate(updatedComment);

            isEditing = false;
        } catch (error) {
            console.error('An error occurred while updating the comment:', error);
        }
    }

    async function handleVote(voteType: 1 | -1) {
        if (!currentUser) return;

        // Store original state in case api call fail
        const originalState = {
            upvotes: comment.upvotes,
            downvotes: comment.downvotes,
            currentUserVote: comment.currentUserVote
        };

        const isTogglingOff = comment.current_user_vote == voteType;
        const isSwitchingVote = comment.current_user_vote == -voteType;

        if (isTogglingOff) {
            // User unvoting
            comment.current_user_vote = undefined;
            if (voteType === 1) comment.upvotes--;
            else comment.downvotes--;
        } else if (isSwitchingVote) {
            comment.current_user_vote = voteType;
            if (voteType === 1) {
                comment.upvotes++;
                comment.downvotes--;
            } else {
                comment.downvotes++;
                comment.upvotes--;
            }
        } else {
            comment.current_user_vote = voteType;
            if (voteType === 1) comment.upvotes++;
            else comment.downvotes++;
        }

        try {
            const response = await apiFetch(`/api/comments/${comment.id}/vote`, {
                method: "POST",
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({vote_type: voteType}),
            });

            if (response.ok) {
                const serverState = await response.json();
                comment.upvotes = serverState.new_upvotes;
                comment.downvotes = serverState.new_downvotes;
                comment.current_user_vote = serverState.current_user_vote;
            } else {
                // If API fails, revert to the original state
                Object.assign(comment, originalState);
                console.error(response, 'Failed to vote');
            }
        } catch (err) {
            // On network error, also revert
            Object.assign(comment, originalState);
            console.error('An error occurred while voting:', err);
        }
    }

    function formatRelativeTime(dateString: string) {
        if (!dateString) return '';

        const date = new Date(dateString);
        const now = new Date();
        let seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        const minutes = Math.floor(seconds / 60);
        if (minutes < 1) return "Just now";

        const hours = Math.floor(minutes / 60);
        if (hours < 1) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;

        const days = Math.floor(hours / 24);
        if (days < 1) return `${hours} hour${hours > 1 ? 's' : ''} ago`;

        return `${days} day${days > 1 ? 's' : ''} ago`;
    }

    function handleLinkClick(event: MouseEvent) {
        const link = (event.target as HTMLElement).closest('a');

        if (link && link.href) {
            if (link.protocol.startsWith("http")) {
                event.preventDefault();

                targetUrl = link.href;
                showLinkWarningModal = true;
            }
        }
    }

    function confirmNavigation() {
        if (targetUrl) {
            window.open(targetUrl, '_blank', 'noopener noreferrer');
        }
        showLinkWarningModal = false;
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

<div class="flex flex-col gap-2">
    <div class="flex gap-3">
        {#if comment.user.avatar_url}
            <img src={comment.user.avatar_url} alt="User Avatar" class="mt-1 h-9 w-9 rounded-full"/>
        {:else}
            <div class="mt-1 flex h-9 w-9 rounded-full items-center justify-center bg-gray-200 dark:bg-gray-700">
                <UserRound class="h-5 w-5 text-gray-500 dark:text-gray-400"/>
            </div>
        {/if}

        <div class="flex items-center gap-2">
            <span class="font-bold text-zinc-800 dark:text-zinc-100">{comment.user.username}</span>
            <span class="text-sm text-zinc-500 dark:text-zinc-400">{formatRelativeTime(comment.created_at)}</span>
        </div>
    </div>
    <div class="flex-1">
        {#if isEditing}
            <div class="mt-2">
                <CommentForm submitComment={handleUpdateSubmit}
                             initialContent={comment.content_markdown}
                             submitLabel="Save"
                             {currentUser}
                             onCancel={() => (isEditing = false)}
                             cancelLabel="Cancel"
                />
            </div>
        {:else}
            <div bind:this={contentContainer}
                 onclick={handleLinkClick}
                 role="none"
                 class="prose prose-a:text-blue-500 prose-img:mb-2 mt-1 max-w-none dark:prose-invert">
                {@html renderedContent}
            </div>

            {#if comment.attachment_url}
                <a href={comment.attachment_url} target="_blank" rel="noopener noreferrer">
                    <img src={comment.attachment_url}
                         alt="User content"
                         class="mt-2 max-h-[250px] max-w-full rounded-lg object-contain"
                    />
                </a>
            {/if}

            <div class="comment-actions mt-1 flex items-center gap-1">
                <Button onclick={() => handleVote(1)}
                        variant="ghost"
                        class="text-sm bg-transparent text-gray-700 dark:text-gray-200 hover:bg-transparent dark:hover:bg-transparent"
                        size="sm">
                    {comment.upvotes}
                    <ChevronUp class="!w-5 !h-5"/>
                </Button>
                <Button onclick={() => handleVote(-1)}
                        variant="ghost"
                        class="text-sm bg-transparent text-gray-700 dark:text-gray-200 hover:bg-transparent dark:hover:bg-transparent"
                        size="sm">
                    {comment.downvotes}
                    <ChevronDown class="!w-5 !h-5"/>
                </Button>
                {#if currentUser}
                    <Button variant="ghost"
                            class="text-sm font-medium bg-transparent text-gray-700 dark:text-gray-200 hover:bg-transparent dark:hover:bg-transparent"
                            size="sm"
                            onclick={() => (showReplyForm = !showReplyForm)}>
                        Reply
                    </Button>
                {/if}
                {#if isOwnerComment}
                    <Button variant="ghost"
                            size="sm"
                            class="text-sm font-medium bg-transparent text-gray-700 dark:text-gray-200 hover:bg-transparent dark:hover:bg-transparent"
                            onclick={() => (isEditing = true)}
                    >
                        Edit
                    </Button>
                {/if}
            </div>
        {/if}
    </div>
    {#if showLinkWarningModal}
        <ModalDialog bind:open={showLinkWarningModal}
                     title="Hold On"
                     description="Do you trust this link?"
                     confirmText="Yes, I trust this link!"
                     onConfirm={confirmNavigation}
                     onCancel={() => (showLinkWarningModal = false)}
                     dialogClass=""
        >
            <div class="mt-2 break-all test-left rounded-md bg-gray-100 p-2 text-sm dark:bg-gray-200 text-gray-800 dark:text-gray-700">
                {targetUrl}
            </div>
        </ModalDialog>
    {/if}

    {#if showReplyForm && !isEditing}
        <div class="mt-3 pl-12">
            <CommentForm
                    submitComment={handleReplySubmit}
                    placeholder={`Reply to ${comment.user.username}`}
                    submitLabel="Reply"
                    {currentUser}
            />
        </div>
    {/if}

    {#if comment.replies?.length > 0}
        <div class="mt-1 border-l-1 border-zinc-200 pl-2 ml-2">
            {#each comment.replies as reply, i (reply.id)}
                <CommentView bind:comment={comment.replies[i]} {currentUser} {addReply} {onUpdate}/>
            {/each}
        </div>
    {/if}
</div>
<script lang="ts">
    import Comment from './CommentView.svelte';
    import CommentForm from './CommentForm.svelte';
    import type {CommentType} from "$lib/components/comments/comments";
    import {apiFetch} from "$lib/store/auth";
    import type {User} from "$lib/store/auth"

    let {entityType, entityId, initialComments = [], currentUser = null} = $props<{
        entityType: 'series' | 'chapters';
        entityId: number;
        initialComments?: CommentType[];
        currentUser: User | null;
    }>();

    let comments = $state<CommentType[]>(initialComments);

    function getEndpoint() {
        if (entityType === 'series') {
            return `/api/series/${entityId}/comments`;
        } else {
            return `/api/series/chapter/${entityId}/comments`;
        }
    }

    $effect(() => {
        console.log(`Fetching comments for ${entityType} ID: ${entityId}`);
        fetchComments();
    });

    async function fetchComments() {
        try {
            const endpoint = getEndpoint();
            const response = await fetch(endpoint);
            if (response.ok) {
                comments = await response.json();
            } else {
                console.error(response);
            }
        } catch (error) {
            console.error(error);
        }
    }

    async function addTopLevelComment(formData: FormData) {
        if (!currentUser) return;
        try {
            const endpoint = getEndpoint();
            const response = await apiFetch(endpoint, {
                method: "POST",
                body: formData
            });

            if (response.ok) {
                const newComment = await response.json();
                comments = [newComment, ...comments];
            } else {
                const errorData = await response.json();
                console.error(errorData);
            }
        } catch (error) {
            console.error(error);
        }
    }

    // A recursive function to find a comment by ID and add a reply to it
    function addReplyToComment(commentsArray: CommentType[], parentId: number, newReply: CommentType): boolean {
        for (let comment of commentsArray) {
            if (comment.id === parentId) {
                if (!comment.replies) {
                    comment.replies = [];
                }
                comment.replies.push(newReply);
                return true;
            }
            // If not found, search in its replies recursively
            if (comment.replies && comment.replies.length > 0) {
                if (addReplyToComment(comment.replies, parentId, newReply)) {
                    return true;
                }
            }
        }
        return false;
    }

    // Recursive helper function to find and update a comment in the comment state tree
    function updateCommentInState(commentsArray: CommentType[], updatedComment: Partial<CommentType>): boolean {
        if (typeof updatedComment.id === 'undefined') {
            return false;
        }

        for (let i = 0; i < commentsArray.length; i++) {
            let comment = commentsArray[i];
            if (comment.id === updatedComment.id) {
                if (updatedComment.content_html !== undefined) {
                    comment.content_html = updatedComment.content_html;
                }
                if (updatedComment.content_markdown !== undefined) {
                    comment.content_markdown = updatedComment.content_markdown;
                }
                if (updatedComment.updated_at !== undefined) {
                    comment.updated_at = updatedComment.updated_at;
                }

                return true;
            }

            if (comment.replies && comment.replies.length > 0) {
                if (updateCommentInState(comment.replies, updatedComment)) {
                    return true;
                }
            }
        }
        return false;
    }

    // function that will be passed as a prop
    function handleCommentUpdate(updateComment: Partial<CommentType>) {
        updateCommentInState(comments, updateComment);
    }

    // Function to add a reply to any comment, nested or not
    async function addReply(parentId: number, content: string) {
        if (!currentUser) return;

        // This recursive function will search for the parent comment in the entire tree
        try {
            const endpoint = getEndpoint();
            const response = await apiFetch(endpoint, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    content_markdown: content,
                    parent_id: parentId
                })
            });
            if (response.ok) {
                const newReply = await response.json();
                const updatedComments = [...comments];

                addReplyToComment(updatedComments, parentId, newReply);
                comments = updatedComments;
            } else {
                const errorData = await response.json();
                console.error(errorData);
            }
        } catch (error) {
            console.error(error);
        }
    }
</script>

<div class="mx-auto my-2 max-w-4xl rounded-sm border border-zinc-200 p-1 font-sans sm:p-1 dark:border-zinc-700">
    <div class="mb-1">
        <CommentForm submitComment={addTopLevelComment} {currentUser}/>
    </div>

    <div class="flex flex-col gap-4">
        {#each comments as comment, i (comment.id)}
            <Comment bind:comment={comments[i]} {addReply} {currentUser} onUpdate={handleCommentUpdate}/>
        {/each}
    </div>
</div>
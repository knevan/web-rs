<script lang="ts">
    import Comment from './CommentView.svelte';
    import CommentForm from './CommentForm.svelte';
    import type {CommentType} from "$lib/components/comments/comments";
    import {apiFetch} from "$lib/store/auth";
    import type {User} from "$lib/store/auth"

    let {seriesId, initialComments = [], currentUser = null} = $props<{
        seriesId: number;
        initialComments?: CommentType[];
        currentUser: User | null;
    }>();

    let comments = $state<CommentType[]>(initialComments);

    $effect(() => {
        console.log(`Fetching comments for series ID: ${seriesId}`);
        fetchComments();
    });

    async function fetchComments() {
        try {
            const response = await fetch(`/api/series/${seriesId}/comments`);
            if (response.ok) {
                comments = await response.json();
            } else {
                console.error(response);
            }
        } catch (error) {
            console.error(error);
        }
    }

    async function addTopLevelComment(content: string) {
        if (!currentUser) return;

        try {
            const response = await apiFetch(`/api/series/${seriesId}/comments`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    content_markdown: content,
                    parent_id: null
                })
            })

            if (response.ok) {
                // After successfully posting, refresh the comments list
                const newComments = await response.json();
                comments = [newComments, ...comments];
            } else {
                const errorData = await response.json();
                console.error(errorData);

            }
            //const newCommentFromServer = await response.json();
            //comments.unshift(newCommentFromServer);
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

    // Function to add a reply to any comment, nested or not
    async function addReply(parentId: number, content: string) {
        if (!currentUser) return;

        // This recursive function will search for the parent comment in the entire tree
        try {
            const response = await apiFetch(`/api/series/${seriesId}/comments`, {
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
        <CommentForm submitText={addTopLevelComment} {currentUser}/>
    </div>

    <div class="flex flex-col gap-4">
        {#each comments as comment (comment.id)}
            <Comment {comment} {addReply} {currentUser}/>
        {/each}
    </div>
</div>
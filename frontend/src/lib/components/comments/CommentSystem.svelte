<script lang="ts">
    import Comment from './CommentView.svelte';
    import CommentForm from './CommentForm.svelte';

    type CommentType = {
        id: number;
        user: {
            name: string;
            avatar: string;
        };
        timestamp: string;
        content: string;
        upvotes: number;
        replies: CommentType[];
    };

    let comments = $state<CommentType[]>([]);

    function addTopLevelComment(content: string) {
        const newComment = createCommentObject(content);
        comments.push(newComment);
    }

    // Function to add a reply to any comment, nested or not
    function addReply(parentId: number, content: string) {
        // This recursive function will search for the parent comment in the entire tree
        function findAndAddReply(nodes: CommentType[], targetId: number) {
            for (const node of nodes) {
                if (node.id === targetId) {
                    node.replies.push(createCommentObject(content));
                    return true;
                }
                if (node.replies && findAndAddReply(node.replies, targetId)) {
                    return true;
                }
            }
            return false;
        }

        findAndAddReply(comments, parentId);
    }

    // Helper function to create a new comment object
    function createCommentObject(content: string): CommentType {
        return {
            id: Date.now(),
            user: {name: 'CurrentUser', avatar: 'https://i.pravatar.cc/40?u=current'},
            timestamp: 'Baru saja',
            content: content,
            upvotes: 0,
            replies: []
        };
    }

</script>

<div class="mx-auto my-2 max-w-4xl rounded-sm border border-zinc-200 p-1 font-sans sm:p-1 dark:border-zinc-700">
    <div class="mb-1">
        <CommentForm submitText={addTopLevelComment}/>
    </div>

    <div class="flex flex-col gap-4">
        {#each comments as comment (comment.id)}
            <Comment {comment} {addReply}/>
        {/each}
    </div>
</div>
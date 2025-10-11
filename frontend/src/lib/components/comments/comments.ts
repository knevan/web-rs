export type CommentType = {
	id: number;
	parent_id: number | null;
	user: {
		username: string;
		avatar_url: string;
	};
	created_at: string;
	updated_at: string;
	content_html: string;
	content_markdown: string;
	upvotes: number;
	downvotes: number;
	current_user_vote: number | null;
	replies: CommentType[];
	attachment_url?: string[] | null;
};

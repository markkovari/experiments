// Domain types for Post module (functional approach)

export type Post = {
  id: string;
  title: string;
  content: string;
  published: boolean;
  authorId: string;
  createdAt: Date;
  updatedAt: Date;
};

export type CreatePostInput = {
  title: string;
  content: string;
  authorId: string;
  published?: boolean;
};

export type UpdatePostInput = {
  title?: string;
  content?: string;
  published?: boolean;
};

export type PostDTO = Omit<Post, 'authorId'> & {
  authorId: string;
};

// Pure function to create PostDTO
export const toPostDTO = (post: Post): PostDTO => ({
  id: post.id,
  title: post.title,
  content: post.content,
  published: post.published,
  authorId: post.authorId,
  createdAt: post.createdAt,
  updatedAt: post.updatedAt,
});

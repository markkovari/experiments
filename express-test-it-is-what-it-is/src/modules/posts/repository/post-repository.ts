import { PrismaClient } from '@prisma/client';
import { Post, CreatePostInput, UpdatePostInput } from '../domain/types';

// Repository functions (functional approach with dependency injection)

export type PostRepository = {
  findById: (id: string) => Promise<Post | null>;
  findAll: () => Promise<Post[]>;
  findByAuthorId: (authorId: string) => Promise<Post[]>;
  findPublished: () => Promise<Post[]>;
  create: (input: CreatePostInput) => Promise<Post>;
  update: (id: string, input: UpdatePostInput) => Promise<Post>;
  delete: (id: string) => Promise<void>;
};

// Factory function to create repository with Prisma dependency
export const createPostRepository = (prisma: PrismaClient): PostRepository => {
  const toDomain = (post: any): Post => ({
    id: post.id,
    title: post.title,
    content: post.content,
    published: post.published,
    authorId: post.authorId,
    createdAt: post.createdAt,
    updatedAt: post.updatedAt,
  });

  return {
    findById: async (id: string) => {
      const post = await prisma.post.findUnique({ where: { id } });
      return post ? toDomain(post) : null;
    },

    findAll: async () => {
      const posts = await prisma.post.findMany();
      return posts.map(toDomain);
    },

    findByAuthorId: async (authorId: string) => {
      const posts = await prisma.post.findMany({ where: { authorId } });
      return posts.map(toDomain);
    },

    findPublished: async () => {
      const posts = await prisma.post.findMany({ where: { published: true } });
      return posts.map(toDomain);
    },

    create: async (input: CreatePostInput) => {
      const post = await prisma.post.create({
        data: {
          title: input.title,
          content: input.content,
          authorId: input.authorId,
          published: input.published ?? false,
        },
      });
      return toDomain(post);
    },

    update: async (id: string, input: UpdatePostInput) => {
      const post = await prisma.post.update({
        where: { id },
        data: input,
      });
      return toDomain(post);
    },

    delete: async (id: string) => {
      await prisma.post.delete({ where: { id } });
    },
  };
};

import { z } from 'zod';
/**
 * Core Todo domain model
 */
export declare const todoSchema: z.ZodObject<{
    id: z.ZodString;
    text: z.ZodString;
    isDone: z.ZodBoolean;
}, "strip", z.ZodTypeAny, {
    id: string;
    text: string;
    isDone: boolean;
}, {
    id: string;
    text: string;
    isDone: boolean;
}>;
export type Todo = z.infer<typeof todoSchema>;

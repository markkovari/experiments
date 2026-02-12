import { z } from 'zod';
/**
 * Response schemas for Todo endpoints
 */
export declare const todoResponseSchema: z.ZodObject<{
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
export declare const todoListResponseSchema: z.ZodArray<z.ZodObject<{
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
}>, "many">;
export declare const deleteResponseSchema: z.ZodObject<{
    success: z.ZodBoolean;
}, "strip", z.ZodTypeAny, {
    success: boolean;
}, {
    success: boolean;
}>;
export type TodoResponse = z.infer<typeof todoResponseSchema>;
export type TodoListResponse = z.infer<typeof todoListResponseSchema>;
export type DeleteResponse = z.infer<typeof deleteResponseSchema>;

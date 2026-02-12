import { z } from 'zod';
/**
 * Request schemas for Todo endpoints
 */
export declare const addTodoRequestSchema: z.ZodObject<{
    text: z.ZodString;
}, "strip", z.ZodTypeAny, {
    text: string;
}, {
    text: string;
}>;
export declare const todoIdParamsSchema: z.ZodObject<{
    id: z.ZodString;
}, "strip", z.ZodTypeAny, {
    id: string;
}, {
    id: string;
}>;
export declare const toggleTodoRequestSchema: z.ZodObject<{
    id: z.ZodString;
}, "strip", z.ZodTypeAny, {
    id: string;
}, {
    id: string;
}>;
export declare const updateTodoRequestSchema: z.ZodObject<{
    id: z.ZodString;
    text: z.ZodString;
}, "strip", z.ZodTypeAny, {
    id: string;
    text: string;
}, {
    id: string;
    text: string;
}>;
export declare const deleteTodoRequestSchema: z.ZodObject<{
    id: z.ZodString;
}, "strip", z.ZodTypeAny, {
    id: string;
}, {
    id: string;
}>;
export type AddTodoRequest = z.infer<typeof addTodoRequestSchema>;
export type TodoIdParams = z.infer<typeof todoIdParamsSchema>;
export type ToggleTodoRequest = z.infer<typeof toggleTodoRequestSchema>;
export type UpdateTodoRequest = z.infer<typeof updateTodoRequestSchema>;
export type DeleteTodoRequest = z.infer<typeof deleteTodoRequestSchema>;

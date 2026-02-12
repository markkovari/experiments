"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deleteTodoRequestSchema = exports.updateTodoRequestSchema = exports.toggleTodoRequestSchema = exports.todoIdParamsSchema = exports.addTodoRequestSchema = void 0;
const zod_1 = require("zod");
/**
 * Request schemas for Todo endpoints
 */
exports.addTodoRequestSchema = zod_1.z.object({
    text: zod_1.z.string().min(1, 'Text is required'),
});
exports.todoIdParamsSchema = zod_1.z.object({
    id: zod_1.z.string(),
});
exports.toggleTodoRequestSchema = zod_1.z.object({
    id: zod_1.z.string(),
});
exports.updateTodoRequestSchema = zod_1.z.object({
    id: zod_1.z.string(),
    text: zod_1.z.string().min(1, 'Text is required'),
});
exports.deleteTodoRequestSchema = zod_1.z.object({
    id: zod_1.z.string(),
});

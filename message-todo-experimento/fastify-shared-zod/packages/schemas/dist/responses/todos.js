"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deleteResponseSchema = exports.todoListResponseSchema = exports.todoResponseSchema = void 0;
const zod_1 = require("zod");
const todo_1 = require("../models/todo");
/**
 * Response schemas for Todo endpoints
 */
exports.todoResponseSchema = todo_1.todoSchema;
exports.todoListResponseSchema = zod_1.z.array(todo_1.todoSchema);
exports.deleteResponseSchema = zod_1.z.object({
    success: zod_1.z.boolean(),
});

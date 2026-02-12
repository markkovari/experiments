"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.todoSchema = void 0;
const zod_1 = require("zod");
/**
 * Core Todo domain model
 */
exports.todoSchema = zod_1.z.object({
    id: zod_1.z.string(),
    text: zod_1.z.string(),
    isDone: zod_1.z.boolean(),
});

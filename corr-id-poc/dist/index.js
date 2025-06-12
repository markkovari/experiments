"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const winston_1 = __importStar(require("winston"));
const node_async_hooks_1 = require("node:async_hooks");
const node_crypto_1 = require("node:crypto");
const transport = new winston_1.default.transports.Console();
const logger = (0, winston_1.createLogger)({ transports: [transport] });
const app = (0, express_1.default)();
const as = new node_async_hooks_1.AsyncLocalStorage();
app.use((req, res, next) => {
    const store = new Map();
    store.set("x-correlation-id", (0, node_crypto_1.randomUUID)());
    as.run(store, () => next());
});
const cLogger = () => {
    const store = as.getStore();
    return {
        info: (message, meta = {}) => { logger.info(message, Object.assign(Object.assign({}, meta), Object.fromEntries(store.entries()))); },
        erro: (message, meta = {}) => { logger.error(message, Object.assign(Object.assign({}, meta), Object.fromEntries(store.entries()))); },
    };
};
app.get("/", (req, res) => {
    cLogger().info("req started");
    setTimeout(() => {
        const cl = cLogger().info("req ended");
        res.json({ ok: "ok" });
        return;
    }, 2000);
});
app.listen(8000, () => console.log("runs"));

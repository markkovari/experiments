"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.readConf = void 0;
const readConf = () => {
    const kv = process.env;
    return {
        email: {
            host: kv.MAIL_HOST || "",
            password: kv.MAIL_PASSWORD || "",
            user: kv.MAIL_USER || "",
            port: Number(kv.MAIL_PORT),
        },
        app: {
            port: Number(kv.APP_PORT)
        }
    };
};
exports.readConf = readConf;

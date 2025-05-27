"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.sendWith = void 0;
const nodemailer_1 = require("nodemailer");
const config_1 = require("../config/config");
const transport = ({ email: { host, password: pass, port, user } } = (0, config_1.readConf)()) => (0, nodemailer_1.createTransport)({
    host,
    port,
    auth: {
        user,
        pass
    },
    secure: false,
    tls: {
        rejectUnauthorized: false
    }
});
const sendWith = (config = (0, config_1.readConf)()) => (opts) => __awaiter(void 0, void 0, void 0, function* () { return yield transport(config).sendMail(opts); });
exports.sendWith = sendWith;

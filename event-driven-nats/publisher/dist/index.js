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
const nats_1 = require("nats");
const sc = (0, nats_1.StringCodec)();
function publisher() {
    return __awaiter(this, void 0, void 0, function* () {
        const nc = yield (0, nats_1.connect)({ servers: 'nats://localhost:4222' });
        setInterval(() => {
            const event = {
                id: `user_${Date.now()}`,
                name: 'John Doe',
                email: `john.doe.${Date.now()}@example.com`,
            };
            nc.publish('UserCreated', sc.encode(JSON.stringify(event)));
            console.log(`Published event: ${JSON.stringify(event)}`);
        }, 2000);
    });
}
publisher().catch(console.error);

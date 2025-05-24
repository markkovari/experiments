import { createTransport } from 'nodemailer';
import { readConf } from "./config"
import type { MailOptions } from 'nodemailer/lib/json-transport';


const transport = ({ email: { host, password: pass, port, user } } = readConf()) => createTransport({
    host,
    port,
    auth:
    {
        user,
        pass
    },
    secure: false,
    tls: {
        rejectUnauthorized: false
    }
})
const sendWith = (config = readConf()) => async (opts: MailOptions) => await transport(config).sendMail(opts)

export {
    // transport,
    sendWith
}
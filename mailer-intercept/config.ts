export type Config = {
    email: MailConfig;
    app: Appconfig
}

export type Appconfig = {
    port: number;
}

export type MailConfig = {
    user: string;
    password: string;
    port: number;
    host: string;
}

const readConf = (): Config => {
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
    }
}

export {
    readConf
}
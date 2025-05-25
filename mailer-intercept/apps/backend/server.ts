import express, { json } from "express";
import { sendWith } from "../../packages/email/send";

const app = express();
app.use(json({ limit: '50mb' }));

app.post("/", async (req, res) => {
    const { emailTo } = req.body;
    const senderWithConf = sendWith();
    const passwordMagic = "some-magical-password" as const;
    try {
        const thing = await senderWithConf({ to: emailTo, text: `Password: ${passwordMagic}`, from: "asdasd@gmail.com", sender: "asdasd@gmail.com", subject: "asdasd" })
        res.json({ message: `Email successfully sent to ${emailTo}`, thing });
        return;
    } catch (e) {
        res.status(500).json({ e })
        return;
    }
})

export {
    app
}

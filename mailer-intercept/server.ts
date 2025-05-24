import express, { json } from "express";
import { sendWith } from "./send";

const app = express();
app.use(json({ limit: '50mb' }));

app.post("/", async (req, res) => {
    const { emailTo } = req.body;
    console.log({ b: req.body });
    const senderWithConf = sendWith();
    console.log(
        { to: emailTo, text: "121312312312", from: "asdasd@gmail.com", sender: "asdasd@gmail.com", subject: "asdasd" }
    )
    try {
        const thing = await senderWithConf({ to: emailTo, text: "121312312312", from: "asdasd@gmail.com", sender: "asdasd@gmail.com", subject: "asdasd" })
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
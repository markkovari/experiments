import { app } from "./server"
import { readConf } from "config"

const { app: { port } } = readConf();

app.listen(port, () => console.info(`Server running on port ${port}`))


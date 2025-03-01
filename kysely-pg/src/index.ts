import { App } from './app'
import { config } from './config'

const app = new App(config);

app.start()
    .finally(() => console.log(`Server started on port: ${config.port}`))
    .catch(console.error)
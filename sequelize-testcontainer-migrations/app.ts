import express from "express";

import { mainRouter } from "./src/routes/";

const app = express();

app.use(mainRouter);

export { app };

import { json } from "body-parser";
import express from "express";

import { mainRouter } from "./src/routes/";

const app = express();

app.use(json());
app.use(mainRouter);

export { app };

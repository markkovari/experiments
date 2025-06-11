# Graceful shutdown


https://expressjs.com/en/advanced/healthcheck-graceful-shutdown.html


Steps to reproduce

1. Install deps

```console
pnpm install
```


2. Build the app

```console
pnpm run build
```


3. Run the app

```console
pnpm run start

# or

node /dist/index.js
```


4. Send a request

```console
curl localhost:8000
```


5. Send a SIGTERM to the node process


```console

ps aux | grep "node " | awk 'NR==1 {print $2; exit}' | xargs kill -SIGTERM
# or in any other way (Ctlr+C e.g.)
```


6. Observe the logs

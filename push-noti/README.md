# push-noti

Tiny Web Push demo: Next.js (route handlers) + `node:sqlite` + `web-push`, client is a PWA.

- Log in with any name + password (first login registers the name).
- "Enable notifications" subscribes this device; "Send push" pushes to a user or everyone.
- VAPID keys are generated on first boot and stored in `data.sqlite`.

```sh
npm run build && npm start         # http://localhost:3000 (build bundles sw/sw.ts -> public/sw.js)
npm run typecheck                  # app + service worker (separate tsconfig, webworker lib)
npm run e2e                        # Playwright, port 3100/3101
ANDROID_SERIAL=<serial> npm run e2e:android   # real phone over adb (USB or Wi-Fi)
```

## Layout

- `app/api/<action>/route.ts`: login, logout, me, subscribe, send, read; bodies validated with zod (`lib/shared/api.ts`)
- `lib/server/`: auth (scrypt + session cookie), messages (per-message read state), push (VAPID, web-push, subscription cleanup)
- `lib/shared/push.ts`: payload types + `applyUnreadState`, used by the page and the service worker
- `sw/sw.ts`: service worker (TypeScript, bundled by esbuild)
- `app/components/`: shadcn/ui (base-nova) + Tailwind v4, "cosmic jellyfish" dark theme

## Read state, notifications and the icon badge

Every push carries the user's unread state (`badge`, `unread` ids, `latest` id). Each device closes notifications of
read messages and sets the app-icon badge (`navigator.setAppBadge`: iOS/iPadOS home-screen apps, macOS Dock apps,
desktop Chrome/Edge installed apps; Android shows a launcher dot instead). Tapping a notification marks it read.
Reading on one device sends a silent `sync` push to the user's other devices, except Apple ones (Apple revokes
subscriptions whose pushes show nothing); those catch up when the app opens or the next message arrives.

## On a real device over Tailscale

Push needs HTTPS (a secure context). `tailscale serve` gives a valid cert on the tailnet:

```sh
npx next start -p 3200
tailscale serve --bg --https=8443 3200   # https://<machine>.<tailnet>.ts.net:8443
tailscale serve --https=8443 off         # teardown
```

iOS/iPadOS 16.4+: open the URL in Safari → Share → **Add to Home Screen** → open from the
home screen → log in → Enable notifications. Push is not available in a regular Safari tab on iOS.

## E2E testing options

| Layer | How | Real push service? |
|---|---|---|
| Server + SW (this repo, `npm run e2e`) | Playwright Chromium; `subscribe()` stubbed to a local fake push service that decrypts the payload; CDP `ServiceWorker.deliverPushMessage` hands it to the real SW; assert `getNotifications()` | No |
| Desktop real push | Headed Chrome/Edge (FCM) or Safari on macOS (APNs). Playwright's bundled Chromium has no FCM keys, so `subscribe()` fails there | Yes |
| Real Android (`npm run e2e:android`, set `ANDROID_SERIAL` for Wi-Fi adb) | `adb reverse` + Chrome DevTools over adb, Playwright `connectOverCDP`; asserts the text in `dumpsys notification` | Yes (FCM) |
| Android emulator | Google Play system image + Chrome, reach the host via `10.0.2.2` or Tailscale on the emulator; drive with Appium/adb or Chrome DevTools over adb | Yes (FCM) |
| iOS simulator | Xcode 14.3+ / macOS 13+ simulators support home-screen web push; drive with `simctl` + Appium Safari | Yes |
| Real iPhone | Tailscale URL above; manual, or Appium + `safaridriver` for the in-browser part | Yes (APNs) |

The home-screen install and the OS permission prompt can't be automated cleanly on iOS; plan on those being manual.

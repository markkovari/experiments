import { applyUnreadState, type UnreadState } from "@/lib/shared/push";
import { api, isError } from "./api";

const registration = () => navigator.serviceWorker?.getRegistration();

/** Close this device's notifications for read messages and set the icon badge. */
export const reconcile = async (state: UnreadState) => applyUnreadState(await registration(), navigator, state);

export async function currentEndpoint() {
  return (await (await registration())?.pushManager.getSubscription())?.endpoint;
}

const urlBase64ToBytes = (b64: string) => Uint8Array.from(atob(b64.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0));

/** Must run from a user gesture (iOS requirement). Returns a status line for the UI. */
export async function enablePush(vapidPublicKey: string): Promise<string> {
  if (!("PushManager" in window)) return "No Push API here. On iOS: Share → Add to Home Screen, then open the app from the home screen.";
  if ((await Notification.requestPermission()) !== "granted") return "Notification permission denied.";
  const reg = await navigator.serviceWorker.ready;
  const sub = await reg.pushManager.subscribe({ userVisibleOnly: true, applicationServerKey: urlBase64ToBytes(vapidPublicKey) });
  const res = await api.subscribe(sub.toJSON());
  return isError(res) ? res.error : "Notifications enabled on this device.";
}

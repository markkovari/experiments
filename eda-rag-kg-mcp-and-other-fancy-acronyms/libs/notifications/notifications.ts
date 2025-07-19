import { connect } from "@nats-io/transport-node";

import { StringCodec } from "nats"

import { Kvm } from "@nats-io/kv";
import { env } from "@magic/env";

type UserId = string;
type NotificationId = string;
type NotificationEnvelope<T> = { id: NotificationId, value: T }

const connection = await connect({ servers: [env.NATS_URL] })

const kvManager = new Kvm(connection);

const notificationBucket = await kvManager.open("user_notifications");
const sc = StringCodec();

const getUserNotifications = async <T = unknown>(userId: UserId): Promise<NotificationEnvelope<T>[]> => {
    const prefix = `notifications.${userId}.>`;
    const notificationKeys = await notificationBucket.keys(prefix);
    const userNotifications: NotificationEnvelope<T>[] = [];
    for await (const key of notificationKeys) {
        try {
            const value = await notificationBucket.get(key);
            if (!value) continue;
            const notificationId = key.split(".")[key.split(".").length - 1];
            userNotifications.push({ id: notificationId, value: JSON.parse(sc.decode(value.value)) });
        } catch {
            continue;
        }
    }
    return userNotifications;
};

const getUserNotificationById = async <T = unknown>(userId: UserId, id: NotificationId): Promise<NotificationEnvelope<T> | null> => {
    const rawEntry = await notificationBucket.get(`notifications.${userId}/${id}`);
    if (rawEntry === null || rawEntry === undefined) return rawEntry;
    try {
        return { id, value: JSON.parse(sc.decode(rawEntry.value)) as T };
    } catch (error) {
        return null
    }
}

const deleteUserNotificationById = async (userId: UserId, id: NotificationId) => notificationBucket.delete(`notifications.${userId}.${id}`);


const addUserNotificationById = async <T = unknown>(userId: UserId, id: NotificationId, value: T) => {
    const asStringified = typeof value === "string" ? value : JSON.stringify(value);
    return notificationBucket.put(`notifications.${userId}.${id}`, sc.encode(asStringified));
}

export { getUserNotificationById, getUserNotifications, deleteUserNotificationById, addUserNotificationById }

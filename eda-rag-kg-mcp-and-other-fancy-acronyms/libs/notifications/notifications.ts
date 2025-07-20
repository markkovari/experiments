import { connect } from "@nats-io/transport-node";

import { StringCodec } from "nats"

import { Kvm } from "@nats-io/kv";
import { env } from "@magic/env";

type UserId = string;
type NotificationId = string;
type NotificationEnvelope<T> = { id: NotificationId, value: T }

const userNotificationsBucketName = "user_notifications";


const notificiations = async <T = unknown>(urls: string[] = [env.NATS_URL], user: string, pass: string, bucketName: string = userNotificationsBucketName) => {
    const connection = await connect({ servers: urls, user, pass })

    const kvManager = new Kvm(connection);

    const notificationBucket = await kvManager.open(bucketName);
    const sc = StringCodec();

    const getUserNotifications = async (userId: UserId): Promise<NotificationEnvelope<T>[]> => {
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

    const getUserNotificationById = async (userId: UserId, id: NotificationId): Promise<NotificationEnvelope<T> | null> => {
        const rawEntry = await notificationBucket.get(`notifications.${userId}/${id}`);
        if (rawEntry === null || rawEntry === undefined) return rawEntry;
        try {
            return { id, value: JSON.parse(sc.decode(rawEntry.value)) as T };
        } catch (error) {
            return null
        }
    }

    const deleteUserNotificationById = async (userId: UserId, id: NotificationId) => notificationBucket.delete(`notifications.${userId}.${id}`);


    const addUserNotificationById = async (userId: UserId, id: NotificationId, value: T) => {
        const asStringified = typeof value === "string" ? value : JSON.stringify(value);
        return notificationBucket.put(`notifications.${userId}.${id}`, sc.encode(asStringified));
    }
    return { getUserNotificationById, getUserNotifications, deleteUserNotificationById, addUserNotificationById, userNotificationsBucketName }
}


export { notificiations, userNotificationsBucketName }
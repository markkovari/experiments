import { env } from "@magic/env";
import { Kvm } from "@nats-io/kv";
import { connect } from "@nats-io/transport-node";
import { StringCodec } from "nats";

type UserId = string;
type NotificationId = string;

export type NotificationEnvelope = {
	id: NotificationId;
	value: string;
	created?: Date;
};

export type NotificationService = {
	getUserNotificationById: (
		userId: UserId,
		id: NotificationId,
	) => Promise<NotificationEnvelope | null>;
	getUserNotifications: (userId: UserId) => Promise<NotificationEnvelope[]>;
	deleteUserNotificationById: (
		userId: UserId,
		id: NotificationId,
	) => Promise<void>;
	addUserNotificationById: (
		userId: UserId,
		id: NotificationId,
		value: string,
	) => Promise<number>;
	userNotificationsBucketName: string;
};

const userNotificationsBucketName = "user_notifications";

const notificiations = async (
	urls: string[] = [env.NATS_URL],
	user: string,
	pass: string,
	bucketName: string = userNotificationsBucketName,
): Promise<NotificationService> => {
	const connection = await connect({ servers: urls, user, pass });

	const kvManager = new Kvm(connection);

	const notificationBucket = await kvManager.open(bucketName);
	const sc = StringCodec();

	const getUserNotifications = async (
		userId: UserId,
	): Promise<NotificationEnvelope[]> => {
		const prefix = `notifications.${userId}.>`;
		const notificationKeys = await notificationBucket.keys(prefix);
		const userNotifications: NotificationEnvelope[] = [];
		for await (const key of notificationKeys) {
			try {
				const entry = await notificationBucket.get(key);
				if (!entry) continue;
				const notificationId = key.split(".")[key.split(".").length - 1];
				const decodedValue = sc.decode(entry.value);
				userNotifications.push({
					id: notificationId,
					value: decodedValue,
				});
			} catch {}
		}
		return userNotifications;
	};

	const getUserNotificationById = async (
		userId: UserId,
		id: NotificationId,
	): Promise<NotificationEnvelope | null> => {
		const rawEntry = await notificationBucket.get(
			`notifications.${userId}/${id}`,
		);
		if (rawEntry === null || rawEntry === undefined) return rawEntry;
		try {
			return {
				id,
				value: sc.decode(rawEntry.value),
				created: rawEntry.created,
			};
		} catch {
			return null;
		}
	};

	const deleteUserNotificationById = async (
		userId: UserId,
		id: NotificationId,
	) => notificationBucket.delete(`notifications.${userId}.${id}`);

	const addUserNotificationById = async (
		userId: UserId,
		id: NotificationId,
		value: string,
	) => {
		return notificationBucket.put(
			`notifications.${userId}.${id}`,
			sc.encode(value),
		);
	};
	return {
		getUserNotificationById,
		getUserNotifications,
		deleteUserNotificationById,
		addUserNotificationById,
		userNotificationsBucketName,
	};
};

export { notificiations, userNotificationsBucketName };

import { randomUUID } from "node:crypto";
import { beforeEach } from "vitest";
import { type NotificationService, notificiations } from "../notifications";
import { type CustomTestContext, describe, expect, it } from "./setup";

describe("notifications", async () => {
	let userId = randomUUID();
	let noti: NotificationService;
	beforeEach<CustomTestContext>(
		async ({
			context: {
				natsAccess: { pass, url, user },
			},
		}) => {
			noti = await notificiations([url], user, pass);
			userId = randomUUID();
		},
	);
	it("should be able to get initial empty", async () => {
		const userNotifications = await noti.getUserNotifications(userId);
		expect(userNotifications).toStrictEqual([]);
	});

	it("should increase the number of elements", async () => {
		const { addUserNotificationById, getUserNotifications } = noti;
		const theKeyToInsertValue = "the_key";
		const theValueAdded = "The value";
		await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);

		const userNotifications = await getUserNotifications(userId);
		console.log({ userNotifications });
		expect(userNotifications.length).not.toBe(0);
	});

	it("adding element to the same key should not increase the messages (upsert)", async () => {
		const { addUserNotificationById, getUserNotifications } = noti;

		const theKeyToInsertValue = "the_key";
		const theValueAdded = "The value";
		await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
		await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
		const userNotifications = await getUserNotifications(userId);
		expect(userNotifications.length).toBe(1);
	});

	it("adding element to different key should increase the messages length", async () => {
		const { addUserNotificationById, getUserNotifications } = noti;

		const theKeyToInsertValue = "the_key";
		const theKeyToInsertValueOther = "the_other_key";
		const theValueAdded = "The value";
		await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
		await addUserNotificationById(
			userId,
			theKeyToInsertValueOther,
			theValueAdded,
		);
		const userNotifications = await getUserNotifications(userId);
		expect(userNotifications.length).toBe(2);
	});

	it("should retrieve the element", async () => {
		const { addUserNotificationById, getUserNotifications } = noti;

		const theKeyToInsertValue = "the_key";
		const theValueAdded = "The value";
		await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
		const userNotifications = await getUserNotifications(userId);
		expect(userNotifications[0].value).toEqual(theValueAdded);
	});

	it("should retrieve the elements created field", async () => {
		const { addUserNotificationById, getUserNotifications } = noti;
		const nonExistentUserId = "213123123123";
		const theKeyToInsertValue = "the_key";
		const theValueAdded = "The value";
		await addUserNotificationById(
			nonExistentUserId,
			theKeyToInsertValue,
			theValueAdded,
		);
		const userNotifications = await getUserNotifications(nonExistentUserId);
		console.log({ first: userNotifications[0] });
		expect(userNotifications[0].created).toEqual(expect.any(Date));
	});
});

import { describe, expect, it } from './testbed';
import {
    notificiations,
} from "../notifications"
import { randomUUID } from 'crypto';
import { beforeEach } from 'vitest';


describe("notifications", () => {
    describe("initial connection", () => {
        it("should be able to get initial empty", async ({ natsUrl }) => {
            const noti = await notificiations<string>([natsUrl], user, pass)
            const nonExistentUserId = Math.floor(Math.random() * 10000000).toString();
            const userNotifications = await noti.getUserNotifications(nonExistentUserId);
            expect(userNotifications).toStrictEqual([]);
        });
    })
})

describe("notifications details", () => {
    let userId = randomUUID();

    beforeEach(() => {
        userId = randomUUID()
    })
    it.only("should increase the number of elements", async ({ natsUrl }) => {
        const { addUserNotificationById, getUserNotifications } = await notificiations<string>([natsUrl], user, pass)
        const theKeyToInsertValue = "the_key";
        const theValueAdded = "The value";
        await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);

        const userNotifications = await getUserNotifications(userId);
        console.log({ userNotifications })
        expect(userNotifications.length).not.toBe(0);
    });

    it("adding element to the same key should not increase the messages (upsert)", async ({ natsUrl }) => {
        const { addUserNotificationById, getUserNotifications } = await notificiations<string>([natsUrl], user, pass)

        const theKeyToInsertValue = "the_key";
        const theValueAdded = "The value";
        await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
        await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
        const userNotifications = await getUserNotifications(userId);
        expect(userNotifications.length).toBe(1);
    });

    it("adding element to different key should increase the messages length", async ({ natsUrl }) => {
        const { addUserNotificationById, getUserNotifications } = await notificiations<string>([natsUrl], user, pass)

        const theKeyToInsertValue = "the_key";
        const theKeyToInsertValueOther = "the_other_key";
        const theValueAdded = "The value";
        await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
        await addUserNotificationById(userId, theKeyToInsertValueOther, theValueAdded);
        const userNotifications = await getUserNotifications(userId);
        expect(userNotifications.length).toBe(2);
    });

    it("should retrieve the element", async ({ natsUrl }) => {
        const { addUserNotificationById, getUserNotifications } = await notificiations<string>([natsUrl], user, pass)

        const theKeyToInsertValue = "the_key";
        const theValueAdded = "The value";
        await addUserNotificationById(userId, theKeyToInsertValue, theValueAdded);
        const userNotifications = await getUserNotifications(userId);
        expect(userNotifications[0].value).toEqual(theValueAdded);
    });

    // it("should retrieve the elements created field", async ({ natsUrl }) => {
    //     const { addUserNotificationById, getUserNotifications } = await notificiations<string>([natsUrl], user, pass)
    //     const nonExistentUserId = "213123123123";
    //     const theKeyToInsertValue = "the_key";
    //     const theValueAdded = "The value";
    //     await addUserNotificationById(nonExistentUserId, theKeyToInsertValue, theValueAdded);
    //     const userNotifications = await getUserNotifications(nonExistentUserId);
    //     console.log({ first: userNotifications[0] })
    //     expect(userNotifications[0].created).toEqual(expect.any(Number));
    // });
})
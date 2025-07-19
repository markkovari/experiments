"use client";

import { useSession } from "next-auth/react";
import { useState } from "react";

import { api } from "~/trpc/react";

export function Notifications() {
    const query = api.notifications.getUserNotifications.useQuery();

    const notifications = query.data?.notifications;
    const [value, setValue] = useState("some-value")
    const [notificationId, setNotificationId] = useState("some-notification-id")
    const utils = api.useUtils();
    const createNotification = api.notifications.createUserNotification.useMutation();
    const deleteNotification = api.notifications.deleteUserNotification.useMutation();

    return (
        <div className="w-full max-w-xs">
            {notifications?.length === 0 ? <p>🎉 No notifications 🎉</p> :
                notifications?.length !== 0 && <>
                    {notifications?.map((notification) =>
                        <div id={`div-${notification.id}`}>
                            <p key={notification.id}>{notification.value.message}</p>
                            <button
                                id={`del-${notification.id}-button`}
                                onClick={(e) => {
                                    deleteNotification.mutate({ notificationId })
                                    utils.notifications.invalidate();
                                }}
                            >DEL</button>
                        </div>
                    )}
                </>}
            <form
                onSubmit={(e) => {
                    e.preventDefault();
                    createNotification.mutate({ notificationId, value });
                    utils.notifications.invalidate()
                }}
                className="flex flex-col gap-2"
            >
                <input
                    type="text"
                    placeholder="Value"
                    value={value}
                    onChange={(e) => setValue(e.target.value)}
                    className="w-full rounded-full bg-white/10 px-4 py-2 text-white"
                />
                <input
                    type="text"
                    placeholder="NotificationId"
                    value={notificationId}
                    onChange={(e) => setNotificationId(e.target.value)}
                    className="w-full rounded-full bg-white/10 px-4 py-2 text-white"
                />
                <button
                    type="submit"
                    className="rounded-full bg-white/10 px-10 py-3 font-semibold transition hover:bg-white/20"
                    disabled={createNotification.isPending}
                >
                    {createNotification.isPending ? "Submitting..." : "Submit"}
                </button>
            </form>
        </div >
    );
}

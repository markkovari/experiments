import { z } from "zod";

import {
	createTRPCRouter,
	protectedProcedure,
} from "~/server/api/trpc";
import { getUserNotificationById, getUserNotifications, addUserNotificationById, deleteUserNotificationById } from "@magic/notifications";

type UserNotification = { message: string };
export const notificationRouter = createTRPCRouter({
	getUserNotifications: protectedProcedure
		.query(async ({ ctx }) => {
			const notifications = await getUserNotifications<UserNotification>(ctx.session.user.id);
			return {
				notifications,
			};
		}),

	getUserNotificationsById: protectedProcedure
		.input(z.object({ id: z.string().min(1) }))
		.query(async ({ ctx, input }) => {
			const notification = await getUserNotificationById(ctx.session.user.id, input.id)
			return {
				notification,
			};
		}),

	createUserNotification: protectedProcedure
		.input(z.object({ value: z.string().min(1), notificationId: z.string().min(1) }))
		.mutation(async ({ ctx, input }) => {
			const notification = await addUserNotificationById<UserNotification>(ctx.session.user.id, input.notificationId, { message: input.value })
			return {
				notification,
			};
		}),

	deleteUserNotification: protectedProcedure
		.input(z.object({ notificationId: z.string().min(1) }))
		.mutation(async ({ ctx, input }) => {
			const notification = await deleteUserNotificationById(ctx.session.user.id, input.notificationId);
			return {
				notification,
			};
		}),
});

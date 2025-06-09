import { z } from "zod"; // Add new import

const UserSchema = z.object({
	email: z.string().email(),
	password: z
		.string()
		.min(8, { message: "Password is too short" })
		.max(20, { message: "Password is too long" }),
});
export { UserSchema };

import "./App.css";
import "./globals.css";
import { Input } from "./components/ui/input";
import { Button } from "./components/ui/button";
import {
	Form,
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "./components/ui/form";

import type { z } from "zod";
import { useForm, type SubmitHandler } from "react-hook-form";
import { UserSchema } from "./registration";
import { zodResolver } from "@hookform/resolvers/zod";
import { ModeToggle } from "./components/ui/mode-toggle";

type FormData = z.infer<typeof UserSchema>;

function App() {
	const form = useForm<FormData>({
		resolver: zodResolver(UserSchema),
	});
	const onSubmit: SubmitHandler<FormData> = (data) => console.log(data);

	return (
		<>
			<ModeToggle />
			<Form {...form}>
				<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
					<FormField
						control={form.control}
						name="email"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Email</FormLabel>
								<FormControl>
									<Input
										autoComplete="home email"
										type="email"
										placeholder="email"
										{...field}
									/>
								</FormControl>
								<FormDescription>This is your email name.</FormDescription>
								<FormMessage />
							</FormItem>
						)}
					/>
					<FormField
						control={form.control}
						name="password"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Password</FormLabel>
								<FormControl>
									<Input type="password" placeholder="password" {...field} />
								</FormControl>
								<FormDescription>This is your password.</FormDescription>
								<FormMessage />
							</FormItem>
						)}
					/>
					<Button variant="default" type="submit">
						Submit
					</Button>
				</form>
			</Form>
		</>
	);
}

export default App;

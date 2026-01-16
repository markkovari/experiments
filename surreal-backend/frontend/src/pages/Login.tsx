import { ArrowRight, Heart } from "lucide-react";
import { FormEvent, useState } from "react";
import { Link, Navigate, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/contexts/AuthContext";

export function Login() {
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [error, setError] = useState("");
	const [loading, setLoading] = useState(false);

	const { login, isAuthenticated } = useAuth();
	const navigate = useNavigate();

	if (isAuthenticated) {
		return <Navigate to="/" replace />;
	}

	const handleSubmit = async (e: FormEvent) => {
		e.preventDefault();
		setError("");
		setLoading(true);

		try {
			await login(email, password);
			navigate("/");
		} catch (err) {
			setError(err instanceof Error ? err.message : "Login failed");
		} finally {
			setLoading(false);
		}
	};

	return (
		<div className="flex min-h-screen items-center justify-center bg-gray-50 p-6">
			<div className="w-full max-w-md space-y-8">
				{/* Logo and Header */}
				<div className="text-center">
					<div className="mx-auto flex h-12 w-12 items-center justify-center rounded-xl bg-primary mb-4">
						<Heart className="h-6 w-6 text-primary-foreground" />
					</div>
					<h1 className="text-3xl font-bold text-gray-900">Welcome back</h1>
					<p className="mt-2 text-sm text-gray-600">
						Sign in to your PetCare account
					</p>
				</div>

				{/* Form Card */}
				<div className="rounded-xl border bg-white p-8 shadow-sm">
					<form onSubmit={handleSubmit} className="space-y-6">
						{error && (
							<div className="rounded-lg bg-red-50 border border-red-200 p-3">
								<p className="text-sm text-red-600">{error}</p>
							</div>
						)}

						<div className="space-y-2">
							<Label htmlFor="email" className="text-sm font-medium text-gray-900">
								Email address
							</Label>
							<Input
								id="email"
								name="email"
								type="email"
								autoComplete="email"
								required
								className="h-11"
								placeholder="you@example.com"
								value={email}
								onChange={(e) => setEmail(e.target.value)}
							/>
						</div>

						<div className="space-y-2">
							<Label htmlFor="password" className="text-sm font-medium text-gray-900">
								Password
							</Label>
							<Input
								id="password"
								name="password"
								type="password"
								autoComplete="current-password"
								required
								className="h-11"
								placeholder="••••••••"
								value={password}
								onChange={(e) => setPassword(e.target.value)}
							/>
						</div>

						<Button
							type="submit"
							disabled={loading}
							className="w-full h-11 gap-2"
						>
							{loading ? "Signing in..." : "Sign in"}
							<ArrowRight className="h-4 w-4" />
						</Button>
					</form>
				</div>

				{/* Footer Links */}
				<div className="space-y-4 text-center text-sm">
					<div className="flex items-center justify-center gap-1">
						<span className="text-gray-600">Don't have an account?</span>
						<Link
							to="/register/user"
							className="font-medium text-primary hover:text-primary/80"
						>
							Sign up as pet owner
						</Link>
					</div>
					<div className="flex items-center justify-center gap-1">
						<span className="text-gray-600">Are you a veterinarian?</span>
						<Link
							to="/register/doctor"
							className="font-medium text-primary hover:text-primary/80"
						>
							Register here
						</Link>
					</div>
				</div>
			</div>
		</div>
	);
}

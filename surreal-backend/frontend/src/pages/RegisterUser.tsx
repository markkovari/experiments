import { FormEvent, useState } from "react";
import { Link, Navigate, useNavigate } from "react-router-dom";
import { useAuth } from "../contexts/AuthContext";

export function RegisterUser() {
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [name, setName] = useState("");
	const [phone, setPhone] = useState("");
	const [address, setAddress] = useState("");
	const [error, setError] = useState("");
	const [loading, setLoading] = useState(false);

	const { registerUser, isAuthenticated } = useAuth();
	const navigate = useNavigate();

	if (isAuthenticated) {
		return <Navigate to="/" replace />;
	}

	const handleSubmit = async (e: FormEvent) => {
		e.preventDefault();
		setError("");
		setLoading(true);

		try {
			await registerUser(email, password, name, phone, address);
			navigate("/");
		} catch (err) {
			setError(err instanceof Error ? err.message : "Registration failed");
		} finally {
			setLoading(false);
		}
	};

	return (
		<div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
			<div className="max-w-md w-full space-y-8">
				<div>
					<h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
						Register as Pet Owner
					</h2>
				</div>
				<form className="mt-8 space-y-6" onSubmit={handleSubmit}>
					{error && (
						<div className="rounded-md bg-red-50 p-4">
							<p className="text-sm text-red-800">{error}</p>
						</div>
					)}
					<div className="space-y-4">
						<div>
							<label
								htmlFor="name"
								className="block text-sm font-medium text-gray-700"
							>
								Full Name *
							</label>
							<input
								id="name"
								name="name"
								type="text"
								required
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={name}
								onChange={(e) => setName(e.target.value)}
							/>
						</div>
						<div>
							<label
								htmlFor="email"
								className="block text-sm font-medium text-gray-700"
							>
								Email *
							</label>
							<input
								id="email"
								name="email"
								type="email"
								required
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={email}
								onChange={(e) => setEmail(e.target.value)}
							/>
						</div>
						<div>
							<label
								htmlFor="password"
								className="block text-sm font-medium text-gray-700"
							>
								Password *
							</label>
							<input
								id="password"
								name="password"
								type="password"
								required
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={password}
								onChange={(e) => setPassword(e.target.value)}
							/>
						</div>
						<div>
							<label
								htmlFor="phone"
								className="block text-sm font-medium text-gray-700"
							>
								Phone
							</label>
							<input
								id="phone"
								name="phone"
								type="tel"
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={phone}
								onChange={(e) => setPhone(e.target.value)}
							/>
						</div>
						<div>
							<label
								htmlFor="address"
								className="block text-sm font-medium text-gray-700"
							>
								Address
							</label>
							<textarea
								id="address"
								name="address"
								rows={3}
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={address}
								onChange={(e) => setAddress(e.target.value)}
							/>
						</div>
					</div>

					<div>
						<button
							type="submit"
							disabled={loading}
							className="w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
						>
							{loading ? "Registering..." : "Register"}
						</button>
					</div>

					<div className="text-center text-sm">
						Already have an account?{" "}
						<Link
							to="/login"
							className="font-medium text-indigo-600 hover:text-indigo-500"
						>
							Sign in
						</Link>
					</div>
				</form>
			</div>
		</div>
	);
}

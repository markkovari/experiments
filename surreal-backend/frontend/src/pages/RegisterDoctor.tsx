import { FormEvent, useState } from "react";
import { Link, Navigate, useNavigate } from "react-router-dom";
import { useAuth } from "../contexts/AuthContext";

export function RegisterDoctor() {
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [name, setName] = useState("");
	const [phone, setPhone] = useState("");
	const [specialization, setSpecialization] = useState("general");
	const [licenseNumber, setLicenseNumber] = useState("");
	const [yearsExperience, setYearsExperience] = useState(0);
	const [error, setError] = useState("");
	const [loading, setLoading] = useState(false);

	const { registerDoctor, isAuthenticated } = useAuth();
	const navigate = useNavigate();

	if (isAuthenticated) {
		return <Navigate to="/" replace />;
	}

	const handleSubmit = async (e: FormEvent) => {
		e.preventDefault();
		setError("");
		setLoading(true);

		try {
			await registerDoctor(
				email,
				password,
				name,
				phone,
				specialization,
				licenseNumber,
				yearsExperience,
			);
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
						Register as Doctor
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
								Phone *
							</label>
							<input
								id="phone"
								type="tel"
								required
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={phone}
								onChange={(e) => setPhone(e.target.value)}
							/>
						</div>
						<div>
							<label
								htmlFor="specialization"
								className="block text-sm font-medium text-gray-700"
							>
								Specialization *
							</label>
							<select
								id="specialization"
								required
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={specialization}
								onChange={(e) => setSpecialization(e.target.value)}
							>
								<option value="general">General Practitioner</option>
								<option value="surgery">Surgery</option>
								<option value="dentistry">Dentistry</option>
								<option value="cardiology">Cardiology</option>
								<option value="dermatology">Dermatology</option>
								<option value="ophthalmology">Ophthalmology</option>
							</select>
						</div>
						<div>
							<label
								htmlFor="licenseNumber"
								className="block text-sm font-medium text-gray-700"
							>
								License Number *
							</label>
							<input
								id="licenseNumber"
								type="text"
								required
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={licenseNumber}
								onChange={(e) => setLicenseNumber(e.target.value)}
							/>
						</div>
						<div>
							<label
								htmlFor="yearsExperience"
								className="block text-sm font-medium text-gray-700"
							>
								Years of Experience *
							</label>
							<input
								id="yearsExperience"
								type="number"
								required
								min="0"
								className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								value={yearsExperience}
								onChange={(e) => setYearsExperience(Number(e.target.value))}
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

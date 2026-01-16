import { ArrowLeft, ArrowRight, Stethoscope } from "lucide-react";
import { FormEvent, useState } from "react";
import { Link, Navigate, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { useAuth } from "@/contexts/AuthContext";

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
		<div className="flex min-h-screen items-center justify-center bg-gray-50 p-6">
			<div className="w-full max-w-2xl space-y-8">
				{/* Header */}
				<div className="text-center">
					<div className="mx-auto flex h-12 w-12 items-center justify-center rounded-xl bg-primary mb-4">
						<Stethoscope className="h-6 w-6 text-primary-foreground" />
					</div>
					<h1 className="text-3xl font-bold text-gray-900">Join as Veterinarian</h1>
					<p className="mt-2 text-sm text-gray-600">
						Register to join our network of professional veterinarians
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

						<div className="grid md:grid-cols-2 gap-6">
							<div className="space-y-2">
								<Label htmlFor="name" className="text-sm font-medium text-gray-900">
									Full Name
								</Label>
								<Input
									id="name"
									name="name"
									type="text"
									required
									className="h-11"
									placeholder="Dr. Jane Smith"
									value={name}
									onChange={(e) => setName(e.target.value)}
								/>
							</div>

							<div className="space-y-2">
								<Label htmlFor="email" className="text-sm font-medium text-gray-900">
									Email
								</Label>
								<Input
									id="email"
									name="email"
									type="email"
									required
									className="h-11"
									placeholder="you@example.com"
									value={email}
									onChange={(e) => setEmail(e.target.value)}
								/>
							</div>
						</div>

						<div className="grid md:grid-cols-2 gap-6">
							<div className="space-y-2">
								<Label htmlFor="password" className="text-sm font-medium text-gray-900">
									Password
								</Label>
								<Input
									id="password"
									name="password"
									type="password"
									required
									className="h-11"
									placeholder="••••••••"
									value={password}
									onChange={(e) => setPassword(e.target.value)}
								/>
							</div>

							<div className="space-y-2">
								<Label htmlFor="phone" className="text-sm font-medium text-gray-900">
									Phone
								</Label>
								<Input
									id="phone"
									name="phone"
									type="tel"
									required
									className="h-11"
									placeholder="+1 234 567 890"
									value={phone}
									onChange={(e) => setPhone(e.target.value)}
								/>
							</div>
						</div>

						<div className="grid md:grid-cols-2 gap-6">
							<div className="space-y-2">
								<Label htmlFor="specialization" className="text-sm font-medium text-gray-900">
									Specialization
								</Label>
								<Select value={specialization} onValueChange={setSpecialization} required>
									<SelectTrigger className="h-11">
										<SelectValue placeholder="Select specialization" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="general">General Practitioner</SelectItem>
										<SelectItem value="surgery">Surgery</SelectItem>
										<SelectItem value="dentistry">Dentistry</SelectItem>
										<SelectItem value="cardiology">Cardiology</SelectItem>
										<SelectItem value="dermatology">Dermatology</SelectItem>
										<SelectItem value="ophthalmology">Ophthalmology</SelectItem>
									</SelectContent>
								</Select>
							</div>

							<div className="space-y-2">
								<Label htmlFor="licenseNumber" className="text-sm font-medium text-gray-900">
									License Number
								</Label>
								<Input
									id="licenseNumber"
									name="licenseNumber"
									type="text"
									required
									className="h-11"
									placeholder="VET-123456"
									value={licenseNumber}
									onChange={(e) => setLicenseNumber(e.target.value)}
								/>
							</div>
						</div>

						<div className="space-y-2">
							<Label htmlFor="yearsExperience" className="text-sm font-medium text-gray-900">
								Years of Experience
							</Label>
							<Input
								id="yearsExperience"
								name="yearsExperience"
								type="number"
								required
								min="0"
								className="h-11"
								placeholder="5"
								value={yearsExperience || ""}
								onChange={(e) =>
									setYearsExperience(Number(e.target.value) || 0)
								}
							/>
						</div>

						<Button
							type="submit"
							disabled={loading}
							className="w-full h-11 gap-2"
						>
							{loading ? "Creating account..." : "Create account"}
							<ArrowRight className="h-4 w-4" />
						</Button>
					</form>
				</div>

				{/* Footer */}
				<div className="text-center">
					<Link
						to="/login"
						className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900"
					>
						<ArrowLeft className="w-4 h-4" />
						Back to login
					</Link>
				</div>
			</div>
		</div>
	);
}

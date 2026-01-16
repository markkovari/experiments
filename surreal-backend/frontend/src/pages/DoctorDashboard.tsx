import { Calendar, FileText, LogOut, PawPrint, Stethoscope, Users } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Pagination } from "../components/Pagination";
import { useAuth } from "../contexts/AuthContext";

interface HealthCheck {
	id: string;
	pet_id: string;
	doctor_id: string;
	scheduled_at: string;
	status: string;
	reason: string;
	diagnosis?: string;
	treatment?: string;
	notes?: string;
}

interface Pet {
	id: string;
	owner_id: string;
	name: string;
	species: string;
	breed?: string;
	age?: number;
	weight_kg?: number;
}

interface User {
	id: string;
	email: string;
	name: string;
	phone?: string;
	address?: string;
}

interface PaginationMeta {
	page: number;
	page_size: number;
	total_items: number;
	total_pages: number;
}

interface PaginatedResponse<T> {
	data: T[];
	pagination: PaginationMeta;
}

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

type Tab = "appointments" | "pets" | "users";

export function DoctorDashboard() {
	const { token, user, logout } = useAuth();
	const [activeTab, setActiveTab] = useState<Tab>("appointments");
	const [checks, setChecks] = useState<HealthCheck[]>([]);
	const [pets, setPets] = useState<Pet[]>([]);
	const [users, setUsers] = useState<User[]>([]);
	const [pagination, setPagination] = useState<PaginationMeta>({
		page: 1,
		page_size: 20,
		total_items: 0,
		total_pages: 0,
	});
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState("");

	const fetchAppointments = useCallback(
		async (page: number) => {
			try {
				setLoading(true);
				const response = await fetch(
					`${API_BASE_URL}/doctor/checks?page=${page}&page_size=10`,
					{
						headers: {
							Authorization: `Bearer ${token}`,
						},
					},
				);

				if (!response.ok) {
					throw new Error("Failed to fetch appointments");
				}

				const data: PaginatedResponse<HealthCheck> = await response.json();
				setChecks(data.data);
				setPagination(data.pagination);
			} catch (err) {
				setError(
					err instanceof Error ? err.message : "Failed to fetch appointments",
				);
			} finally {
				setLoading(false);
			}
		},
		[token],
	);

	const fetchPets = useCallback(
		async (page: number) => {
			try {
				setLoading(true);
				const response = await fetch(
					`${API_BASE_URL}/doctor/pets?page=${page}&page_size=10`,
					{
						headers: {
							Authorization: `Bearer ${token}`,
						},
					},
				);

				if (!response.ok) {
					throw new Error("Failed to fetch pets");
				}

				const data: PaginatedResponse<Pet> = await response.json();
				setPets(data.data);
				setPagination(data.pagination);
			} catch (err) {
				setError(err instanceof Error ? err.message : "Failed to fetch pets");
			} finally {
				setLoading(false);
			}
		},
		[token],
	);

	const fetchUsers = useCallback(
		async (page: number) => {
			try {
				setLoading(true);
				const response = await fetch(
					`${API_BASE_URL}/doctor/users?page=${page}&page_size=10`,
					{
						headers: {
							Authorization: `Bearer ${token}`,
						},
					},
				);

				if (!response.ok) {
					throw new Error("Failed to fetch users");
				}

				const data: PaginatedResponse<User> = await response.json();
				setUsers(data.data);
				setPagination(data.pagination);
			} catch (err) {
				setError(err instanceof Error ? err.message : "Failed to fetch users");
			} finally {
				setLoading(false);
			}
		},
		[token],
	);

	useEffect(() => {
		if (activeTab === "appointments") {
			fetchAppointments(1);
		} else if (activeTab === "pets") {
			fetchPets(1);
		} else if (activeTab === "users") {
			fetchUsers(1);
		}
	}, [activeTab, fetchAppointments, fetchPets, fetchUsers]);

	const handlePageChange = (page: number) => {
		if (activeTab === "appointments") {
			fetchAppointments(page);
		} else if (activeTab === "pets") {
			fetchPets(page);
		} else if (activeTab === "users") {
			fetchUsers(page);
		}
	};

	const formatDate = (dateString: string) => {
		return new Date(dateString).toLocaleString();
	};

	const getStatusColor = (status: string) => {
		switch (status) {
			case "scheduled":
				return "bg-blue-50 text-blue-700 border-blue-200";
			case "in_progress":
				return "bg-yellow-50 text-yellow-700 border-yellow-200";
			case "completed":
				return "bg-green-50 text-green-700 border-green-200";
			case "cancelled":
				return "bg-red-50 text-red-700 border-red-200";
			default:
				return "bg-gray-50 text-gray-700 border-gray-200";
		}
	};

	return (
		<div className="flex h-screen overflow-hidden bg-gray-50/50">
			{/* Sidebar */}
			<aside className="hidden w-64 flex-col border-r bg-white lg:flex">
				<div className="flex h-16 items-center border-b px-6">
					<div className="flex items-center gap-2">
						<div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
							<Stethoscope className="h-4 w-4 text-primary-foreground" />
						</div>
						<span className="text-lg font-semibold">PetCare</span>
					</div>
				</div>
				<nav className="flex-1 space-y-1 p-4">
					<button
						onClick={() => setActiveTab("appointments")}
						className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium ${
							activeTab === "appointments"
								? "bg-gray-100 text-gray-900"
								: "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
						}`}
					>
						<Calendar className="h-4 w-4" />
						Appointments
					</button>
					<button
						onClick={() => setActiveTab("pets")}
						className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium ${
							activeTab === "pets"
								? "bg-gray-100 text-gray-900"
								: "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
						}`}
					>
						<PawPrint className="h-4 w-4" />
						Pets
					</button>
					<button
						onClick={() => setActiveTab("users")}
						className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium ${
							activeTab === "users"
								? "bg-gray-100 text-gray-900"
								: "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
						}`}
					>
						<Users className="h-4 w-4" />
						Pet Owners
					</button>
				</nav>
				<Separator />
				<div className="p-4">
					<button
						onClick={logout}
						className="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-gray-900"
					>
						<LogOut className="h-4 w-4" />
						Logout
					</button>
				</div>
				<div className="border-t p-4">
					<div className="flex items-center gap-3">
						<div className="flex h-8 w-8 items-center justify-center rounded-full bg-gray-100">
							<Stethoscope className="h-4 w-4 text-gray-600" />
						</div>
						<div className="flex-1 overflow-hidden">
							<p className="truncate text-sm font-medium text-gray-900">{user?.email}</p>
							<p className="text-xs text-gray-500">Veterinarian</p>
						</div>
					</div>
				</div>
			</aside>

			{/* Main Content */}
			<div className="flex flex-1 flex-col overflow-hidden">
				{/* Top Header */}
				<header className="flex h-16 items-center justify-between border-b bg-white px-6">
					<div>
						<h1 className="text-2xl font-bold text-gray-900">
							{activeTab === "appointments" && "Appointments"}
							{activeTab === "pets" && "Pets"}
							{activeTab === "users" && "Pet Owners"}
						</h1>
					</div>
					<div className="flex items-center gap-2">
						<Button
							variant="ghost"
							size="icon"
							className="lg:hidden"
							onClick={logout}
						>
							<LogOut className="h-4 w-4" />
						</Button>
					</div>
				</header>

				{/* Scrollable Content */}
				<main className="flex-1 overflow-y-auto p-6 md:p-8">
					<div className="mx-auto max-w-6xl">
						{error && (
							<div className="mb-6 rounded-lg border border-red-200 bg-red-50 p-4">
								<p className="text-sm text-red-600">{error}</p>
							</div>
						)}

						{loading ? (
							<div className="flex flex-col items-center justify-center py-24">
								<div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
								<p className="mt-4 text-sm text-gray-500">Loading...</p>
							</div>
						) : (
							<>
								{activeTab === "appointments" && (
									<div className="space-y-4">
										{checks.length === 0 ? (
											<div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 bg-white p-16">
												<div className="flex h-20 w-20 items-center justify-center rounded-full bg-gray-100">
													<Calendar className="h-10 w-10 text-gray-400" />
												</div>
												<h3 className="mt-4 text-lg font-semibold text-gray-900">No appointments found</h3>
												<p className="mt-2 text-sm text-gray-500">
													Appointments will appear here when scheduled
												</p>
											</div>
										) : (
											checks.map((check) => (
												<div
													key={check.id}
													className="rounded-xl border bg-white p-6 shadow-sm hover:shadow-md transition-shadow"
												>
													<div className="flex items-start justify-between">
														<div className="flex-1">
															<div className="flex items-center gap-2 mb-2">
																<FileText className="h-5 w-5 text-gray-400" />
																<h4 className="text-lg font-semibold text-gray-900">
																	{check.reason}
																</h4>
																<span
																	className={`px-2.5 py-0.5 text-xs font-medium rounded-full border ${getStatusColor(check.status)}`}
																>
																	{check.status}
																</span>
															</div>
															<p className="text-sm text-gray-600 mb-3">
																<Calendar className="inline h-4 w-4 mr-1" />
																{formatDate(check.scheduled_at)}
															</p>
															<p className="text-sm text-gray-500">
																Pet ID: <span className="font-mono text-gray-700">{check.pet_id}</span>
															</p>
															{check.diagnosis && (
																<div className="mt-4 rounded-lg bg-gray-50 p-3">
																	<p className="text-xs font-medium text-gray-600 mb-1">
																		Diagnosis
																	</p>
																	<p className="text-sm text-gray-900">{check.diagnosis}</p>
																</div>
															)}
															{check.treatment && (
																<div className="mt-3 rounded-lg bg-gray-50 p-3">
																	<p className="text-xs font-medium text-gray-600 mb-1">
																		Treatment
																	</p>
																	<p className="text-sm text-gray-900">{check.treatment}</p>
																</div>
															)}
														</div>
													</div>
												</div>
											))
										)}
									</div>
								)}

								{activeTab === "pets" && (
									<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
										{pets.length === 0 ? (
											<div className="col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 bg-white p-16">
												<div className="flex h-20 w-20 items-center justify-center rounded-full bg-gray-100">
													<PawPrint className="h-10 w-10 text-gray-400" />
												</div>
												<h3 className="mt-4 text-lg font-semibold text-gray-900">No pets found</h3>
												<p className="mt-2 text-sm text-gray-500">
													Pet records will appear here
												</p>
											</div>
										) : (
											pets.map((pet) => (
												<div
													key={pet.id}
													className="rounded-xl border bg-white p-6 shadow-sm hover:shadow-md transition-shadow"
												>
													<div className="flex items-start justify-between mb-4">
														<div className="flex-1">
															<h4 className="font-semibold text-lg text-gray-900">
																{pet.name}
															</h4>
															<p className="text-sm text-gray-500 mt-1">
																{pet.species}
																{pet.breed && ` • ${pet.breed}`}
															</p>
														</div>
														<div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gray-50">
															<PawPrint className="h-5 w-5 text-gray-400" />
														</div>
													</div>
													<div className="space-y-2">
														{pet.age !== undefined && (
															<div className="flex items-center justify-between text-sm">
																<span className="text-gray-600">Age</span>
																<span className="font-medium text-gray-900">{pet.age} years</span>
															</div>
														)}
														{pet.weight_kg !== undefined && (
															<div className="flex items-center justify-between text-sm">
																<span className="text-gray-600">Weight</span>
																<span className="font-medium text-gray-900">{pet.weight_kg} kg</span>
															</div>
														)}
														<div className="pt-2 border-t">
															<p className="text-xs text-gray-500">
																Owner ID: <span className="font-mono">{pet.owner_id}</span>
															</p>
														</div>
													</div>
												</div>
											))
										)}
									</div>
								)}

								{activeTab === "users" && (
									<div className="space-y-4">
										{users.length === 0 ? (
											<div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 bg-white p-16">
												<div className="flex h-20 w-20 items-center justify-center rounded-full bg-gray-100">
													<Users className="h-10 w-10 text-gray-400" />
												</div>
												<h3 className="mt-4 text-lg font-semibold text-gray-900">No pet owners found</h3>
												<p className="mt-2 text-sm text-gray-500">
													Pet owner records will appear here
												</p>
											</div>
										) : (
											users.map((user) => (
												<div
													key={user.id}
													className="rounded-xl border bg-white p-6 shadow-sm hover:shadow-md transition-shadow"
												>
													<div className="flex items-start gap-4">
														<div className="flex h-12 w-12 items-center justify-center rounded-full bg-gray-100">
															<Users className="h-6 w-6 text-gray-400" />
														</div>
														<div className="flex-1">
															<h4 className="text-lg font-semibold text-gray-900">
																{user.name}
															</h4>
															<p className="text-sm text-gray-600 mt-1">
																{user.email}
															</p>
															<div className="mt-3 space-y-1">
																{user.phone && (
																	<p className="text-sm text-gray-500">
																		Phone: <span className="text-gray-700">{user.phone}</span>
																	</p>
																)}
																{user.address && (
																	<p className="text-sm text-gray-500">
																		Address: <span className="text-gray-700">{user.address}</span>
																	</p>
																)}
															</div>
														</div>
													</div>
												</div>
											))
										)}
									</div>
								)}

								{pagination.total_pages > 1 && (
									<div className="mt-8">
										<Pagination
											currentPage={pagination.page}
											totalPages={pagination.total_pages}
											onPageChange={handlePageChange}
										/>
									</div>
								)}
							</>
						)}
					</div>
				</main>
			</div>
		</div>
	);
}

import { useEffect, useState } from "react";
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

	const fetchAppointments = async (page: number) => {
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
	};

	const fetchPets = async (page: number) => {
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
	};

	const fetchUsers = async (page: number) => {
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
	};

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
				return "bg-blue-100 text-blue-800";
			case "in_progress":
				return "bg-yellow-100 text-yellow-800";
			case "completed":
				return "bg-green-100 text-green-800";
			case "cancelled":
				return "bg-red-100 text-red-800";
			default:
				return "bg-gray-100 text-gray-800";
		}
	};

	return (
		<div className="min-h-screen bg-gray-50">
			<nav className="bg-white shadow">
				<div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
					<div className="flex justify-between h-16">
						<div className="flex items-center">
							<h1 className="text-xl font-bold">Doctor Dashboard</h1>
						</div>
						<div className="flex items-center space-x-4">
							<span className="text-sm text-gray-700">{user?.email}</span>
							<button
								onClick={logout}
								className="text-sm text-red-600 hover:text-red-800"
							>
								Logout
							</button>
						</div>
					</div>
				</div>
			</nav>

			<main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
				{error && (
					<div className="mb-4 rounded-md bg-red-50 p-4">
						<p className="text-sm text-red-800">{error}</p>
					</div>
				)}

				<div className="mb-6">
					<div className="border-b border-gray-200">
						<nav className="-mb-px flex space-x-8">
							<button
								onClick={() => setActiveTab("appointments")}
								className={`${
									activeTab === "appointments"
										? "border-indigo-500 text-indigo-600"
										: "border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700"
								} whitespace-nowrap border-b-2 py-4 px-1 text-sm font-medium`}
							>
								Appointments
							</button>
							<button
								onClick={() => setActiveTab("pets")}
								className={`${
									activeTab === "pets"
										? "border-indigo-500 text-indigo-600"
										: "border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700"
								} whitespace-nowrap border-b-2 py-4 px-1 text-sm font-medium`}
							>
								Pets
							</button>
							<button
								onClick={() => setActiveTab("users")}
								className={`${
									activeTab === "users"
										? "border-indigo-500 text-indigo-600"
										: "border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700"
								} whitespace-nowrap border-b-2 py-4 px-1 text-sm font-medium`}
							>
								Users
							</button>
						</nav>
					</div>
				</div>

				{loading ? (
					<div className="text-center py-12">
						<p className="text-gray-500">Loading...</p>
					</div>
				) : (
					<>
						{activeTab === "appointments" && (
							<div className="bg-white shadow overflow-hidden sm:rounded-md">
								{checks.length === 0 ? (
									<div className="text-center py-12">
										<p className="text-gray-500">No appointments found.</p>
									</div>
								) : (
									<ul className="divide-y divide-gray-200">
										{checks.map((check) => (
											<li key={check.id} className="px-4 py-4 sm:px-6">
												<div className="flex items-start justify-between">
													<div className="flex-1">
														<div className="flex items-center space-x-2">
															<h4 className="text-lg font-medium text-gray-900">
																{check.reason}
															</h4>
															<span
																className={`px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(check.status)}`}
															>
																{check.status}
															</span>
														</div>
														<p className="mt-1 text-sm text-gray-500">
															Scheduled: {formatDate(check.scheduled_at)}
														</p>
														<p className="mt-1 text-sm text-gray-500">
															Pet ID: {check.pet_id}
														</p>
														{check.diagnosis && (
															<p className="mt-2 text-sm text-gray-700">
																<span className="font-medium">Diagnosis:</span>{" "}
																{check.diagnosis}
															</p>
														)}
														{check.treatment && (
															<p className="mt-1 text-sm text-gray-700">
																<span className="font-medium">Treatment:</span>{" "}
																{check.treatment}
															</p>
														)}
													</div>
												</div>
											</li>
										))}
									</ul>
								)}
							</div>
						)}

						{activeTab === "pets" && (
							<div className="bg-white shadow overflow-hidden sm:rounded-md">
								{pets.length === 0 ? (
									<div className="text-center py-12">
										<p className="text-gray-500">No pets found.</p>
									</div>
								) : (
									<ul className="divide-y divide-gray-200">
										{pets.map((pet) => (
											<li key={pet.id} className="px-4 py-4 sm:px-6">
												<div className="flex items-start justify-between">
													<div className="flex-1">
														<h4 className="text-lg font-medium text-indigo-600">
															{pet.name}
														</h4>
														<div className="mt-2 flex flex-col sm:flex-row sm:space-x-4">
															<p className="text-sm text-gray-500">
																Species: {pet.species}
															</p>
															{pet.breed && (
																<p className="text-sm text-gray-500">
																	Breed: {pet.breed}
																</p>
															)}
															{pet.age !== undefined && (
																<p className="text-sm text-gray-500">
																	Age: {pet.age} years
																</p>
															)}
															{pet.weight_kg !== undefined && (
																<p className="text-sm text-gray-500">
																	Weight: {pet.weight_kg} kg
																</p>
															)}
														</div>
														<p className="mt-1 text-sm text-gray-500">
															Owner ID: {pet.owner_id}
														</p>
													</div>
												</div>
											</li>
										))}
									</ul>
								)}
							</div>
						)}

						{activeTab === "users" && (
							<div className="bg-white shadow overflow-hidden sm:rounded-md">
								{users.length === 0 ? (
									<div className="text-center py-12">
										<p className="text-gray-500">No users found.</p>
									</div>
								) : (
									<ul className="divide-y divide-gray-200">
										{users.map((user) => (
											<li key={user.id} className="px-4 py-4 sm:px-6">
												<div className="flex items-start justify-between">
													<div className="flex-1">
														<h4 className="text-lg font-medium text-gray-900">
															{user.name}
														</h4>
														<p className="mt-1 text-sm text-gray-500">
															Email: {user.email}
														</p>
														{user.phone && (
															<p className="mt-1 text-sm text-gray-500">
																Phone: {user.phone}
															</p>
														)}
														{user.address && (
															<p className="mt-1 text-sm text-gray-500">
																Address: {user.address}
															</p>
														)}
													</div>
												</div>
											</li>
										))}
									</ul>
								)}
							</div>
						)}

						{pagination.total_pages > 1 && (
							<Pagination
								currentPage={pagination.page}
								totalPages={pagination.total_pages}
								onPageChange={handlePageChange}
							/>
						)}
					</>
				)}
			</main>
		</div>
	);
}

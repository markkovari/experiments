import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Pagination } from "../components/Pagination";
import { useAuth } from "../contexts/AuthContext";

interface Pet {
	id: string;
	name: string;
	species: string;
	breed?: string;
	age?: number;
	weight_kg?: number;
}

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

interface PaginationMeta {
	page: number;
	page_size: number;
	total_items: number;
	total_pages: number;
}

interface PaginatedResponse {
	data: HealthCheck[];
	pagination: PaginationMeta;
}

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

export function PetDetail() {
	const { petId } = useParams();
	const { token } = useAuth();
	const [pet, setPet] = useState<Pet | null>(null);
	const [checks, setChecks] = useState<HealthCheck[]>([]);
	const [pagination, setPagination] = useState<PaginationMeta>({
		page: 1,
		page_size: 20,
		total_items: 0,
		total_pages: 0,
	});
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState("");

	const fetchPet = async () => {
		try {
			const response = await fetch(`${API_BASE_URL}/me/pets/${petId}`, {
				headers: {
					Authorization: `Bearer ${token}`,
				},
			});

			if (!response.ok) {
				throw new Error("Failed to fetch pet");
			}

			const data: Pet = await response.json();
			setPet(data);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Failed to fetch pet");
		}
	};

	const fetchChecks = async (page: number) => {
		try {
			setLoading(true);
			const response = await fetch(
				`${API_BASE_URL}/me/pets/${petId}/checks?page=${page}&page_size=10`,
				{
					headers: {
						Authorization: `Bearer ${token}`,
					},
				},
			);

			if (!response.ok) {
				throw new Error("Failed to fetch appointments");
			}

			const data: PaginatedResponse = await response.json();
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

	useEffect(() => {
		fetchPet();
		fetchChecks(1);
	}, [fetchChecks, fetchPet]);

	const handlePageChange = (page: number) => {
		fetchChecks(page);
	};

	const isPastAppointment = (scheduledAt: string) => {
		return new Date(scheduledAt) < new Date();
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
							<Link
								to="/"
								className="text-indigo-600 hover:text-indigo-800 mr-4"
							>
								← Back to My Pets
							</Link>
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

				{pet && (
					<div className="bg-white shadow sm:rounded-lg mb-6">
						<div className="px-4 py-5 sm:p-6">
							<h2 className="text-2xl font-bold text-gray-900 mb-4">
								{pet.name}
							</h2>
							<dl className="grid grid-cols-1 gap-x-4 gap-y-4 sm:grid-cols-2">
								<div>
									<dt className="text-sm font-medium text-gray-500">Species</dt>
									<dd className="mt-1 text-sm text-gray-900">{pet.species}</dd>
								</div>
								{pet.breed && (
									<div>
										<dt className="text-sm font-medium text-gray-500">Breed</dt>
										<dd className="mt-1 text-sm text-gray-900">{pet.breed}</dd>
									</div>
								)}
								{pet.age !== undefined && (
									<div>
										<dt className="text-sm font-medium text-gray-500">Age</dt>
										<dd className="mt-1 text-sm text-gray-900">
											{pet.age} years
										</dd>
									</div>
								)}
								{pet.weight_kg !== undefined && (
									<div>
										<dt className="text-sm font-medium text-gray-500">
											Weight
										</dt>
										<dd className="mt-1 text-sm text-gray-900">
											{pet.weight_kg} kg
										</dd>
									</div>
								)}
							</dl>
						</div>
					</div>
				)}

				<div className="bg-white shadow sm:rounded-lg">
					<div className="px-4 py-5 sm:p-6">
						<h3 className="text-lg font-medium text-gray-900 mb-4">
							Appointments
						</h3>

						{loading ? (
							<div className="text-center py-12">
								<p className="text-gray-500">Loading appointments...</p>
							</div>
						) : checks.length === 0 ? (
							<div className="text-center py-12">
								<p className="text-gray-500">No appointments found.</p>
							</div>
						) : (
							<>
								<div className="space-y-4">
									{checks.map((check) => (
										<div
											key={check.id}
											className="border border-gray-200 rounded-lg p-4"
										>
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
														{isPastAppointment(check.scheduled_at) ? (
															<span className="px-2 py-1 text-xs font-semibold rounded-full bg-gray-100 text-gray-800">
																Past
															</span>
														) : (
															<span className="px-2 py-1 text-xs font-semibold rounded-full bg-blue-100 text-blue-800">
																Upcoming
															</span>
														)}
													</div>
													<p className="mt-1 text-sm text-gray-500">
														Scheduled: {formatDate(check.scheduled_at)}
													</p>
													{check.diagnosis && (
														<div className="mt-3">
															<p className="text-sm font-medium text-gray-700">
																Diagnosis:
															</p>
															<p className="mt-1 text-sm text-gray-900">
																{check.diagnosis}
															</p>
														</div>
													)}
													{check.treatment && (
														<div className="mt-3">
															<p className="text-sm font-medium text-gray-700">
																Treatment:
															</p>
															<p className="mt-1 text-sm text-gray-900">
																{check.treatment}
															</p>
														</div>
													)}
													{check.notes && (
														<div className="mt-3">
															<p className="text-sm font-medium text-gray-700">
																Notes:
															</p>
															<p className="mt-1 text-sm text-gray-900">
																{check.notes}
															</p>
														</div>
													)}
												</div>
											</div>
										</div>
									))}
								</div>

								{pagination.total_pages > 1 && (
									<div className="mt-6">
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
				</div>
			</main>
		</div>
	);
}

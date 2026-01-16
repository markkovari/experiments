import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
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

interface PaginationMeta {
	page: number;
	page_size: number;
	total_items: number;
	total_pages: number;
}

interface PaginatedResponse {
	data: Pet[];
	pagination: PaginationMeta;
}

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

export function UserDashboard() {
	const { token, user, logout } = useAuth();
	const [pets, setPets] = useState<Pet[]>([]);
	const [pagination, setPagination] = useState<PaginationMeta>({
		page: 1,
		page_size: 20,
		total_items: 0,
		total_pages: 0,
	});
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState("");

	const fetchPets = async (page: number) => {
		try {
			setLoading(true);
			const response = await fetch(
				`${API_BASE_URL}/me/pets?page=${page}&page_size=10`,
				{
					headers: {
						Authorization: `Bearer ${token}`,
					},
				},
			);

			if (!response.ok) {
				throw new Error("Failed to fetch pets");
			}

			const data: PaginatedResponse = await response.json();
			setPets(data.data);
			setPagination(data.pagination);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Failed to fetch pets");
		} finally {
			setLoading(false);
		}
	};

	useEffect(() => {
		fetchPets(1);
	}, [fetchPets]);

	const handlePageChange = (page: number) => {
		fetchPets(page);
	};

	return (
		<div className="min-h-screen bg-gray-50">
			<nav className="bg-white shadow">
				<div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
					<div className="flex justify-between h-16">
						<div className="flex items-center">
							<h1 className="text-xl font-bold">My Pets</h1>
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

				{loading ? (
					<div className="text-center py-12">
						<p className="text-gray-500">Loading pets...</p>
					</div>
				) : pets.length === 0 ? (
					<div className="text-center py-12">
						<p className="text-gray-500">
							No pets found. Register your first pet!
						</p>
					</div>
				) : (
					<>
						<div className="bg-white shadow overflow-hidden sm:rounded-md">
							<ul className="divide-y divide-gray-200">
								{pets.map((pet) => (
									<li key={pet.id}>
										<Link
											to={`/pets/${pet.id}`}
											className="block hover:bg-gray-50"
										>
											<div className="px-4 py-4 sm:px-6">
												<div className="flex items-center justify-between">
													<div className="flex-1">
														<p className="text-lg font-medium text-indigo-600 truncate">
															{pet.name}
														</p>
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
													</div>
													<div>
														<svg
															className="h-5 w-5 text-gray-400"
															fill="none"
															strokeLinecap="round"
															strokeLinejoin="round"
															strokeWidth="2"
															viewBox="0 0 24 24"
															stroke="currentColor"
														>
															<path d="M9 5l7 7-7 7" />
														</svg>
													</div>
												</div>
											</div>
										</Link>
									</li>
								))}
							</ul>
						</div>

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

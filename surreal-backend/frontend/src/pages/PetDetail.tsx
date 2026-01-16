import {
	ArrowLeft,
	Calendar,
	FileText,
	Heart,
	LayoutDashboard,
	LogOut,
	PawPrint,
	Settings,
	User,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
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
	const { token, user, logout } = useAuth();
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

	const fetchPet = useCallback(async () => {
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
	}, [petId, token]);

	const fetchChecks = useCallback(
		async (page: number) => {
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
		},
		[petId, token],
	);

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

	const getStatusVariant = (status: string): "default" | "secondary" | "outline" | "destructive" => {
		switch (status) {
			case "scheduled":
				return "default";
			case "in_progress":
				return "secondary";
			case "completed":
				return "outline";
			case "cancelled":
				return "destructive";
			default:
				return "secondary";
		}
	};

	return (
		<div className="flex h-screen overflow-hidden bg-gray-50/50">
			{/* Sidebar */}
			<aside className="hidden w-64 flex-col border-r bg-white lg:flex">
				<div className="flex h-16 items-center border-b px-6">
					<Link to="/" className="flex items-center gap-2">
						<div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
							<Heart className="h-4 w-4 text-primary-foreground" />
						</div>
						<span className="text-lg font-semibold">PetCare</span>
					</Link>
				</div>
				<nav className="flex-1 space-y-1 p-4">
					<Link
						to="/"
						className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-gray-900"
					>
						<LayoutDashboard className="h-4 w-4" />
						Dashboard
					</Link>
					<Link
						to="/"
						className="flex items-center gap-3 rounded-lg bg-gray-100 px-3 py-2 text-sm font-medium text-gray-900"
					>
						<PawPrint className="h-4 w-4" />
						My Pets
					</Link>
				</nav>
				<Separator />
				<div className="p-4 space-y-1">
					<button className="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-gray-900">
						<Settings className="h-4 w-4" />
						Settings
					</button>
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
							<User className="h-4 w-4 text-gray-600" />
						</div>
						<div className="flex-1 overflow-hidden">
							<p className="truncate text-sm font-medium text-gray-900">{user?.email}</p>
							<p className="text-xs text-gray-500">Pet Owner</p>
						</div>
					</div>
				</div>
			</aside>

			{/* Main Content */}
			<div className="flex flex-1 flex-col overflow-hidden">
				{/* Top Header */}
				<header className="flex h-16 items-center justify-between border-b bg-white px-6">
					<div className="flex items-center gap-4">
						<Link
							to="/"
							className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900"
						>
							<ArrowLeft className="h-4 w-4" />
							<span className="hidden sm:inline">Back to My Pets</span>
						</Link>
					</div>
				</header>

				{/* Scrollable Content */}
				<main className="flex-1 overflow-y-auto p-6 md:p-8">
					<div className="mx-auto max-w-6xl space-y-6">
						{error && (
							<div className="mb-6 rounded-lg border border-red-200 bg-red-50 p-4">
								<p className="text-sm text-red-600">{error}</p>
							</div>
						)}

						{pet && (
							<div className="rounded-xl border bg-white p-6 shadow-sm">
								<div className="flex items-start gap-4 mb-6">
									<div className="flex h-12 w-12 items-center justify-center rounded-lg bg-gray-100">
										<PawPrint className="h-6 w-6 text-gray-400" />
									</div>
									<div>
										<h2 className="text-2xl font-bold text-gray-900">{pet.name}</h2>
										<p className="text-sm text-gray-600 mt-1">Pet Details</p>
									</div>
								</div>
								<div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
									<div className="rounded-lg bg-gray-50 p-4">
										<p className="text-xs text-gray-600 mb-1">Species</p>
										<p className="text-sm font-medium text-gray-900">{pet.species}</p>
									</div>
									{pet.breed && (
										<div className="rounded-lg bg-gray-50 p-4">
											<p className="text-xs text-gray-600 mb-1">Breed</p>
											<p className="text-sm font-medium text-gray-900">{pet.breed}</p>
										</div>
									)}
									{pet.age !== undefined && (
										<div className="rounded-lg bg-gray-50 p-4">
											<p className="text-xs text-gray-600 mb-1">Age</p>
											<p className="text-sm font-medium text-gray-900">{pet.age} years</p>
										</div>
									)}
									{pet.weight_kg !== undefined && (
										<div className="rounded-lg bg-gray-50 p-4">
											<p className="text-xs text-gray-600 mb-1">Weight</p>
											<p className="text-sm font-medium text-gray-900">{pet.weight_kg} kg</p>
										</div>
									)}
								</div>
							</div>
						)}

						<div className="rounded-xl border bg-white p-6 shadow-sm">
							<div className="flex items-center gap-3 mb-6">
								<Calendar className="h-5 w-5 text-gray-400" />
								<h3 className="text-xl font-semibold text-gray-900">Health Check Appointments</h3>
							</div>

							{loading ? (
								<div className="flex flex-col items-center justify-center py-16">
									<div className="animate-spin rounded-full h-10 w-10 border-b-2 border-primary" />
									<p className="mt-4 text-sm text-gray-500">
										Loading appointments...
									</p>
								</div>
							) : checks.length === 0 ? (
								<div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-gray-300 bg-gray-50 p-12">
									<div className="flex h-16 w-16 items-center justify-center rounded-full bg-white">
										<Calendar className="h-8 w-8 text-gray-400" />
									</div>
									<h4 className="mt-4 text-base font-semibold text-gray-900">No appointments yet</h4>
									<p className="mt-2 text-sm text-gray-600">
										Health check appointments will appear here
									</p>
								</div>
							) : (
								<>
									<div className="space-y-4">
										{checks.map((check) => (
											<div
												key={check.id}
												className="rounded-lg border bg-gray-50 p-4 hover:bg-gray-100 transition-colors"
											>
												<div className="flex items-start justify-between mb-3">
													<div className="flex items-center gap-2 flex-wrap">
														<FileText className="h-4 w-4 text-gray-400" />
														<h4 className="text-sm font-semibold text-gray-900">{check.reason}</h4>
														<Badge variant={getStatusVariant(check.status)}>
															{check.status}
														</Badge>
														{isPastAppointment(check.scheduled_at) ? (
															<Badge variant="secondary">Past</Badge>
														) : (
															<Badge variant="default">Upcoming</Badge>
														)}
													</div>
												</div>
												<div className="space-y-2 text-sm">
													<div className="flex items-center gap-2 text-gray-600">
														<Calendar className="h-3.5 w-3.5" />
														<span>{formatDate(check.scheduled_at)}</span>
													</div>
													{check.diagnosis && (
														<div className="rounded-md bg-white p-3 mt-3">
															<p className="text-xs font-medium text-gray-600 mb-1">
																Diagnosis
															</p>
															<p className="text-sm text-gray-900">{check.diagnosis}</p>
														</div>
													)}
													{check.treatment && (
														<div className="rounded-md bg-white p-3 mt-2">
															<p className="text-xs font-medium text-gray-600 mb-1">
																Treatment
															</p>
															<p className="text-sm text-gray-900">{check.treatment}</p>
														</div>
													)}
													{check.notes && (
														<div className="rounded-md bg-white p-3 mt-2">
															<p className="text-xs font-medium text-gray-600 mb-1">
																Notes
															</p>
															<p className="text-sm text-gray-900">{check.notes}</p>
														</div>
													)}
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
		</div>
	);
}

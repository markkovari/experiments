import {
	Dog,
	Heart,
	Home,
	LayoutDashboard,
	LogOut,
	PawPrint,
	Plus,
	Settings,
	User,
} from "lucide-react";
import { FormEvent, useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
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
	const [dialogOpen, setDialogOpen] = useState(false);
	const [submitLoading, setSubmitLoading] = useState(false);
	const [submitError, setSubmitError] = useState("");

	// Form state
	const [name, setName] = useState("");
	const [species, setSpecies] = useState("Dog");
	const [breed, setBreed] = useState("");
	const [age, setAge] = useState<number | "">("");
	const [weightKg, setWeightKg] = useState<number | "">("");

	const fetchPets = useCallback(
		async (page: number) => {
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
		},
		[token],
	);

	useEffect(() => {
		fetchPets(1);
	}, [fetchPets]);

	const handlePageChange = (page: number) => {
		fetchPets(page);
	};

	const handleSubmit = async (e: FormEvent) => {
		e.preventDefault();
		setSubmitError("");
		setSubmitLoading(true);

		try {
			if (!user?.reference_id) {
				throw new Error("User information not available");
			}

			const response = await fetch(`${API_BASE_URL}/pets`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
					Authorization: `Bearer ${token}`,
				},
				body: JSON.stringify({
					owner_id: user.reference_id,
					name,
					species,
					breed: breed || undefined,
					age: age === "" ? undefined : age,
					weight_kg: weightKg === "" ? undefined : weightKg,
				}),
			});

			if (!response.ok) {
				const errorData = await response.json();
				throw new Error(errorData.message || "Failed to add pet");
			}

			// Reset form
			setName("");
			setSpecies("Dog");
			setBreed("");
			setAge("");
			setWeightKg("");
			setDialogOpen(false);

			// Refresh pets list
			fetchPets(pagination.page);
		} catch (err) {
			setSubmitError(err instanceof Error ? err.message : "Failed to add pet");
		} finally {
			setSubmitLoading(false);
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
						className="flex items-center gap-3 rounded-lg bg-gray-100 px-3 py-2 text-sm font-medium text-gray-900"
					>
						<LayoutDashboard className="h-4 w-4" />
						Dashboard
					</Link>
					<Link
						to="/"
						className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-gray-900"
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
					<div>
						<h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
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
					<div className="mx-auto max-w-6xl space-y-8">
						{/* Stats Cards */}
						<div className="grid gap-6 sm:grid-cols-2">
							<div className="rounded-xl border bg-white p-6 shadow-sm">
								<div className="flex items-center justify-between mb-4">
									<p className="text-sm font-medium text-gray-600">
										Total Pets
									</p>
									<PawPrint className="h-5 w-5 text-gray-400" />
								</div>
								<div>
									<p className="text-3xl font-bold text-gray-900">{pagination.total_items}</p>
									<p className="mt-1 text-sm text-gray-500">
										Registered pets
									</p>
								</div>
							</div>
							<div className="rounded-xl border bg-white p-6 shadow-sm">
								<div className="flex items-center justify-between mb-4">
									<p className="text-sm font-medium text-gray-600">
										Active Records
									</p>
									<LayoutDashboard className="h-5 w-5 text-gray-400" />
								</div>
								<div>
									<p className="text-3xl font-bold text-gray-900">{pets.length}</p>
									<p className="mt-1 text-sm text-gray-500">
										On this page
									</p>
								</div>
							</div>
						</div>

						{/* Page Header with Add Button */}
						<div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
							<div className="space-y-1">
								<h2 className="text-2xl font-semibold text-gray-900">Your Pets</h2>
								<p className="text-gray-600">
									Manage and track your pets' health records
								</p>
							</div>
							<Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
								<DialogTrigger asChild>
									<Button size="default" className="gap-2">
										<Plus className="h-4 w-4" />
										Add Pet
									</Button>
								</DialogTrigger>
								<DialogContent className="sm:max-w-[500px]">
									<DialogHeader>
										<DialogTitle className="flex items-center gap-2 text-xl">
											<Dog className="h-5 w-5" />
											Register New Pet
										</DialogTitle>
										<DialogDescription>
											Add a new pet to your health records dashboard
										</DialogDescription>
									</DialogHeader>
									<form onSubmit={handleSubmit} className="space-y-6 mt-4">
										{submitError && (
											<div className="rounded-lg bg-red-50 border border-red-200 p-3">
												<p className="text-sm text-red-600">{submitError}</p>
											</div>
										)}

										<div className="space-y-2">
											<Label htmlFor="pet-name" className="text-sm font-medium">Pet Name *</Label>
											<Input
												id="pet-name"
												required
												className="h-11"
												placeholder="e.g., Max, Luna, Charlie"
												value={name}
												onChange={(e) => setName(e.target.value)}
											/>
										</div>

										<div className="grid grid-cols-2 gap-4">
											<div className="space-y-2">
												<Label htmlFor="species" className="text-sm font-medium">Species *</Label>
												<Select value={species} onValueChange={setSpecies} required>
													<SelectTrigger className="h-11">
														<SelectValue placeholder="Select species" />
													</SelectTrigger>
													<SelectContent>
														<SelectItem value="Dog">Dog</SelectItem>
														<SelectItem value="Cat">Cat</SelectItem>
														<SelectItem value="Bird">Bird</SelectItem>
														<SelectItem value="Rabbit">Rabbit</SelectItem>
														<SelectItem value="Hamster">Hamster</SelectItem>
														<SelectItem value="Other">Other</SelectItem>
													</SelectContent>
												</Select>
											</div>

											<div className="space-y-2">
												<Label htmlFor="breed" className="text-sm font-medium">Breed</Label>
												<Input
													id="breed"
													className="h-11"
													placeholder="e.g., Golden Retriever"
													value={breed}
													onChange={(e) => setBreed(e.target.value)}
												/>
											</div>
										</div>

										<div className="grid grid-cols-2 gap-4">
											<div className="space-y-2">
												<Label htmlFor="age" className="text-sm font-medium">Age (years)</Label>
												<Input
													id="age"
													type="number"
													min="0"
													step="0.1"
													className="h-11"
													placeholder="e.g., 3"
													value={age}
													onChange={(e) =>
														setAge(e.target.value ? Number(e.target.value) : "")
													}
												/>
											</div>

											<div className="space-y-2">
												<Label htmlFor="weight" className="text-sm font-medium">Weight (kg)</Label>
												<Input
													id="weight"
													type="number"
													min="0"
													step="0.1"
													className="h-11"
													placeholder="e.g., 25.5"
													value={weightKg}
													onChange={(e) =>
														setWeightKg(e.target.value ? Number(e.target.value) : "")
													}
												/>
											</div>
										</div>

										<div className="flex gap-3 pt-2">
											<Button
												type="button"
												variant="outline"
												onClick={() => setDialogOpen(false)}
												className="flex-1"
											>
												Cancel
											</Button>
											<Button type="submit" disabled={submitLoading} className="flex-1">
												{submitLoading ? "Adding..." : "Add Pet"}
											</Button>
										</div>
									</form>
								</DialogContent>
							</Dialog>
						</div>

						{/* Error Display */}
						{error && (
							<div className="mb-6 rounded-lg border border-red-200 bg-red-50 p-4">
								<p className="text-sm text-red-600">{error}</p>
							</div>
						)}

						{/* Loading State */}
						{loading ? (
							<div className="flex flex-col items-center justify-center py-24">
								<div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
								<p className="mt-4 text-sm text-gray-500">
									Loading your pets...
								</p>
							</div>
						) : pets.length === 0 ? (
							/* Empty State */
							<div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 bg-white p-16">
								<div className="flex h-20 w-20 items-center justify-center rounded-full bg-gray-100">
									<Dog className="h-10 w-10 text-gray-400" />
								</div>
								<h3 className="mt-4 text-lg font-semibold text-gray-900">No pets yet</h3>
								<p className="mb-6 mt-2 text-sm text-gray-500 text-center max-w-sm">
									Get started by adding your first pet to the dashboard
								</p>
								<Button onClick={() => setDialogOpen(true)} size="default" className="gap-2">
									<Plus className="h-4 w-4" />
									Add Your First Pet
								</Button>
							</div>
						) : (
							/* Pets Grid */
							<>
								<div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
									{pets.map((pet) => (
										<Link
											key={pet.id}
											to={`/pets/${pet.id}`}
											className="group relative overflow-hidden rounded-xl border bg-white shadow-sm transition-all hover:shadow-md"
										>
											<div className="p-6">
												<div className="flex items-start justify-between mb-4">
													<div className="flex-1 min-w-0">
														<h3 className="font-semibold text-gray-900 truncate text-lg">
															{pet.name}
														</h3>
														<p className="text-sm text-gray-500 mt-1 truncate">
															{pet.species}
															{pet.breed && ` • ${pet.breed}`}
														</p>
													</div>
													<div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-gray-50">
														<PawPrint className="h-5 w-5 text-gray-400" />
													</div>
												</div>

												{(pet.age !== undefined || pet.weight_kg !== undefined) && (
													<div className="mt-4 space-y-2 rounded-lg bg-gray-50 p-3">
														{pet.age !== undefined && (
															<div className="flex items-center justify-between text-sm">
																<span className="text-gray-600">Age</span>
																<span className="font-medium text-gray-900">
																	{pet.age} {pet.age === 1 ? "year" : "years"}
																</span>
															</div>
														)}
														{pet.weight_kg !== undefined && (
															<div className="flex items-center justify-between text-sm">
																<span className="text-gray-600">Weight</span>
																<span className="font-medium text-gray-900">{pet.weight_kg} kg</span>
															</div>
														)}
													</div>
												)}

												<div className="mt-4 text-sm text-primary group-hover:text-primary/80 transition-colors font-medium">
													View details →
												</div>
											</div>
										</Link>
									))}
								</div>

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

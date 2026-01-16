import { jwtDecode } from "jwt-decode";
import {
	createContext,
	ReactNode,
	useContext,
	useEffect,
	useState,
} from "react";

interface AuthToken {
	access_token: string;
	token_type: string;
	expires_in: number;
}

interface UserInfo {
	id: string;
	email: string;
	role: "user" | "doctor";
	reference_id: string;
}

interface AuthResponse {
	token: AuthToken;
	user: UserInfo;
}

interface AuthContextType {
	user: UserInfo | null;
	token: string | null;
	login: (email: string, password: string) => Promise<void>;
	registerUser: (
		email: string,
		password: string,
		name: string,
		phone?: string,
		address?: string,
	) => Promise<void>;
	registerDoctor: (
		email: string,
		password: string,
		name: string,
		phone: string,
		specialization: string,
		license_number: string,
		years_experience: number,
	) => Promise<void>;
	logout: () => void;
	isAuthenticated: boolean;
	isDoctor: boolean;
	isUser: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

export function AuthProvider({ children }: { children: ReactNode }) {
	const [user, setUser] = useState<UserInfo | null>(null);
	const [token, setToken] = useState<string | null>(null);

	// Load auth state from localStorage on mount
	useEffect(() => {
		const storedToken = localStorage.getItem("auth_token");
		const storedUser = localStorage.getItem("auth_user");

		if (storedToken && storedUser) {
			try {
				const decoded = jwtDecode<{ exp: number }>(storedToken);
				// Check if token is expired
				if (decoded.exp * 1000 > Date.now()) {
					setToken(storedToken);
					setUser(JSON.parse(storedUser));
				} else {
					// Token expired, clear storage
					localStorage.removeItem("auth_token");
					localStorage.removeItem("auth_user");
				}
			} catch {
				// Invalid token, clear storage
				localStorage.removeItem("auth_token");
				localStorage.removeItem("auth_user");
			}
		}
	}, []);

	const login = async (email: string, password: string) => {
		const response = await fetch(`${API_BASE_URL}/auth/login`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ email, password }),
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || "Login failed");
		}

		const data: AuthResponse = await response.json();
		setToken(data.token.access_token);
		setUser(data.user);

		localStorage.setItem("auth_token", data.token.access_token);
		localStorage.setItem("auth_user", JSON.stringify(data.user));
	};

	const registerUser = async (
		email: string,
		password: string,
		name: string,
		phone?: string,
		address?: string,
	) => {
		const response = await fetch(`${API_BASE_URL}/auth/register/user`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ email, password, name, phone, address }),
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || "Registration failed");
		}

		const data: AuthResponse = await response.json();
		setToken(data.token.access_token);
		setUser(data.user);

		localStorage.setItem("auth_token", data.token.access_token);
		localStorage.setItem("auth_user", JSON.stringify(data.user));
	};

	const registerDoctor = async (
		email: string,
		password: string,
		name: string,
		phone: string,
		specialization: string,
		license_number: string,
		years_experience: number,
	) => {
		const response = await fetch(`${API_BASE_URL}/auth/register/doctor`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				email,
				password,
				name,
				phone,
				specialization,
				license_number,
				years_experience,
			}),
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || "Registration failed");
		}

		const data: AuthResponse = await response.json();
		setToken(data.token.access_token);
		setUser(data.user);

		localStorage.setItem("auth_token", data.token.access_token);
		localStorage.setItem("auth_user", JSON.stringify(data.user));
	};

	const logout = () => {
		setToken(null);
		setUser(null);
		localStorage.removeItem("auth_token");
		localStorage.removeItem("auth_user");
	};

	return (
		<AuthContext.Provider
			value={{
				user,
				token,
				login,
				registerUser,
				registerDoctor,
				logout,
				isAuthenticated: !!token && !!user,
				isDoctor: user?.role === "doctor",
				isUser: user?.role === "user",
			}}
		>
			{children}
		</AuthContext.Provider>
	);
}

export function useAuth() {
	const context = useContext(AuthContext);
	if (context === undefined) {
		throw new Error("useAuth must be used within an AuthProvider");
	}
	return context;
}

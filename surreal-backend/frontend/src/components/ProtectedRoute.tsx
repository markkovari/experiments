import { ReactNode } from "react";
import { Navigate } from "react-router-dom";
import { useAuth } from "../contexts/AuthContext";

interface ProtectedRouteProps {
	children: ReactNode;
	requireRole?: "user" | "doctor";
}

export function ProtectedRoute({ children, requireRole }: ProtectedRouteProps) {
	const { isAuthenticated, user } = useAuth();

	if (!isAuthenticated) {
		return <Navigate to="/login" replace />;
	}

	if (requireRole && user?.role !== requireRole) {
		return <Navigate to="/" replace />;
	}

	return <>{children}</>;
}

import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { AuthProvider, useAuth } from "./contexts/AuthContext";
import { DoctorDashboard } from "./pages/DoctorDashboard";
import { Login } from "./pages/Login";
import { PetDetail } from "./pages/PetDetail";
import { RegisterDoctor } from "./pages/RegisterDoctor";
import { RegisterUser } from "./pages/RegisterUser";
import { UserDashboard } from "./pages/UserDashboard";

function DashboardRouter() {
	const { isDoctor, isUser } = useAuth();

	if (isDoctor) {
		return <DoctorDashboard />;
	}

	if (isUser) {
		return <UserDashboard />;
	}

	return <Navigate to="/login" replace />;
}

function App() {
	return (
		<BrowserRouter>
			<AuthProvider>
				<Routes>
					<Route path="/login" element={<Login />} />
					<Route path="/register/user" element={<RegisterUser />} />
					<Route path="/register/doctor" element={<RegisterDoctor />} />
					<Route
						path="/"
						element={
							<ProtectedRoute>
								<DashboardRouter />
							</ProtectedRoute>
						}
					/>
					<Route
						path="/pets/:petId"
						element={
							<ProtectedRoute requireRole="user">
								<PetDetail />
							</ProtectedRoute>
						}
					/>
					<Route path="*" element={<Navigate to="/login" replace />} />
				</Routes>
			</AuthProvider>
		</BrowserRouter>
	);
}

export default App;

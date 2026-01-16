pub mod user;
pub mod pet;
pub mod doctor;
pub mod check;
pub mod auth;

pub use user::User;
pub use pet::{Pet, PetSpecies};
pub use doctor::{Doctor, Specialization};
pub use check::{HealthCheck, CheckStatus};
pub use auth::{
    AuthUser, UserRole, LoginCredentials, RegisterUserRequest, RegisterDoctorRequest,
    AuthToken, AuthResponse, UserInfo, Claims,
};

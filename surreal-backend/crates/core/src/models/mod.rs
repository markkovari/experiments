pub mod auth;
pub mod check;
pub mod doctor;
pub mod pet;
pub mod user;

pub use auth::{
    AuthResponse, AuthToken, AuthUser, Claims, LoginCredentials, RegisterDoctorRequest,
    RegisterUserRequest, UserInfo, UserRole,
};
pub use check::{CheckStatus, HealthCheck};
pub use doctor::{Doctor, Specialization};
pub use pet::{Pet, PetSpecies};
pub use user::User;

pub mod connection;
pub mod error;
pub mod repository;

pub use connection::Database;
pub use error::{DbError, Result};
pub use repository::{
    AuthRepository, CheckRepository, DoctorRepository, PetRepository, Repository, UserRepository,
};

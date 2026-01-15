use chrono::{Duration, Utc};
use surreal_core::{
    CheckStatus, Doctor, HealthCheck, Pet, PetSpecies, Specialization, User,
};
use surreal_db::{
    CheckRepository, Database, DoctorRepository, PetRepository, Repository, UserRepository,
};
use tracing::info;

pub async fn seed_database(db: &Database) -> anyhow::Result<()> {
    info!("Seeding database with sample data");

    // Create repositories
    let user_repo = UserRepository::new(db.clone());
    let pet_repo = PetRepository::new(db.clone());
    let doctor_repo = DoctorRepository::new(db.clone());
    let check_repo = CheckRepository::new(db.clone());

    // Seed users
    info!("Creating sample users");
    let user1 = User::new("john.doe@example.com".to_string(), "John Doe".to_string())?
        .with_phone("+1234567890".to_string())?
        .with_address("123 Main St, New York, NY".to_string());

    let user2 = User::new("jane.smith@example.com".to_string(), "Jane Smith".to_string())?
        .with_phone("+1987654321".to_string())?
        .with_address("456 Oak Ave, Los Angeles, CA".to_string());

    let user3 = User::new("bob.wilson@example.com".to_string(), "Bob Wilson".to_string())?
        .with_phone("+1555555555".to_string())?;

    let created_user1 = user_repo.create(&user1).await?;
    let created_user2 = user_repo.create(&user2).await?;
    let created_user3 = user_repo.create(&user3).await?;

    // Seed pets
    info!("Creating sample pets");
    let pet1 = Pet::new(
        created_user1.id.clone().unwrap(),
        "Buddy".to_string(),
        PetSpecies::Dog,
    )?
    .with_breed("Golden Retriever".to_string())
    .with_weight(30.5)?
    .with_medical_notes("Allergic to chicken".to_string());

    let pet2 = Pet::new(
        created_user1.id.clone().unwrap(),
        "Max".to_string(),
        PetSpecies::Dog,
    )?
    .with_breed("German Shepherd".to_string())
    .with_weight(35.0)?;

    let pet3 = Pet::new(
        created_user2.id.clone().unwrap(),
        "Whiskers".to_string(),
        PetSpecies::Cat,
    )?
    .with_breed("Persian".to_string())
    .with_weight(4.5)?;

    let pet4 = Pet::new(
        created_user3.id.clone().unwrap(),
        "Tweety".to_string(),
        PetSpecies::Bird,
    )?
    .with_weight(0.2)?
    .with_medical_notes("Requires special diet".to_string());

    let created_pet1 = pet_repo.create(&pet1).await?;
    let created_pet2 = pet_repo.create(&pet2).await?;
    let created_pet3 = pet_repo.create(&pet3).await?;
    let _created_pet4 = pet_repo.create(&pet4).await?;

    // Seed doctors
    info!("Creating sample doctors");
    let doctor1 = Doctor::new(
        "Dr. Sarah Johnson".to_string(),
        "sarah.johnson@clinic.com".to_string(),
        "+1222333444".to_string(),
        Specialization::GeneralPractice,
        "VET-001-2020".to_string(),
        12,
    )?;

    let doctor2 = Doctor::new(
        "Dr. Michael Chen".to_string(),
        "michael.chen@clinic.com".to_string(),
        "+1333444555".to_string(),
        Specialization::Surgery,
        "VET-002-2018".to_string(),
        15,
    )?;

    let doctor3 = Doctor::new(
        "Dr. Emily Rodriguez".to_string(),
        "emily.rodriguez@clinic.com".to_string(),
        "+1444555666".to_string(),
        Specialization::Cardiology,
        "VET-003-2021".to_string(),
        8,
    )?;

    let created_doctor1 = doctor_repo.create(&doctor1).await?;
    let created_doctor2 = doctor_repo.create(&doctor2).await?;
    let _created_doctor3 = doctor_repo.create(&doctor3).await?;

    // Seed health checks
    info!("Creating sample health checks");

    // Future appointments
    let check1 = HealthCheck::new(
        created_pet1.id.clone().unwrap(),
        created_doctor1.id.clone().unwrap(),
        Utc::now() + Duration::days(3),
    )?;

    let check2 = HealthCheck::new(
        created_pet2.id.clone().unwrap(),
        created_doctor2.id.clone().unwrap(),
        Utc::now() + Duration::days(5),
    )?;

    let check3 = HealthCheck::new(
        created_pet3.id.clone().unwrap(),
        created_doctor1.id.clone().unwrap(),
        Utc::now() + Duration::days(7),
    )?;

    // Completed check
    let mut check4 = HealthCheck::new(
        created_pet1.id.clone().unwrap(),
        created_doctor1.id.clone().unwrap(),
        Utc::now() + Duration::hours(1),
    )?;
    check4.status = CheckStatus::Completed;
    check4.diagnosis = Some("Annual checkup - healthy".to_string());
    check4.treatment = Some("Vaccination updated".to_string());
    check4.cost = Some(75.0);

    check_repo.create(&check1).await?;
    check_repo.create(&check2).await?;
    check_repo.create(&check3).await?;
    check_repo.create(&check4).await?;

    info!("Database seeding completed successfully");
    info!("Created: 3 users, 4 pets, 3 doctors, 4 health checks");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_seed_database() {
        let db = Database::new_in_memory().await.unwrap();

        // Run migrations first would be needed in real scenario
        let result = seed_database(&db).await;
        assert!(result.is_ok());

        // Verify seeded data
        let user_repo = UserRepository::new(db.clone());
        let users = user_repo.find_all().await.unwrap();
        assert_eq!(users.len(), 3);
    }
}

use chrono::{Duration, Utc};
use surreal_core::{
    CheckStatus, Doctor, HealthCheck, Pet, PetSpecies, Specialization, User,
};
use surreal_db::{
    CheckRepository, Database, DoctorRepository, PetRepository, Repository, UserRepository,
};
use surreal_migrations::MigrationRunner;

async fn setup_test_db() -> Database {
    let db = Database::new_in_memory().await.unwrap();
    let runner = MigrationRunner::new(db.clone());
    runner.run().await.unwrap();
    db
}

#[tokio::test]
async fn test_user_workflow() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Create user
    let user = User::new("test@example.com".to_string(), "Test User".to_string())
        .unwrap()
        .with_phone("+1234567890".to_string())
        .unwrap();

    let created = repo.create(&user).await.unwrap();
    assert_eq!(created.email, user.email);
    assert!(created.id.is_some());

    let user_id = created.id.as_ref().unwrap();

    // Find user
    let found = repo.find_by_id(user_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test User");

    // Find by email
    let found_by_email = repo.find_by_email("test@example.com").await.unwrap();
    assert!(found_by_email.is_some());

    // Update user
    let mut updated_user = created.clone();
    updated_user.name = "Updated User".to_string();
    let updated = repo.update(&updated_user).await.unwrap();
    assert_eq!(updated.name, "Updated User");

    // Delete user
    let deleted = repo.delete(user_id).await.unwrap();
    assert!(deleted);

    // Verify deletion
    let not_found = repo.find_by_id(user_id).await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_pet_workflow() {
    let db = setup_test_db().await;
    let user_repo = UserRepository::new(db.clone());
    let pet_repo = PetRepository::new(db);

    // Create owner
    let owner = User::new("owner@example.com".to_string(), "Pet Owner".to_string()).unwrap();
    let created_owner = user_repo.create(&owner).await.unwrap();
    let owner_id = created_owner.id.clone().unwrap();

    // Create pet
    let pet = Pet::new(owner_id.clone(), "Buddy".to_string(), PetSpecies::Dog)
        .unwrap()
        .with_breed("Golden Retriever".to_string())
        .with_weight(30.0)
        .unwrap();

    let created = pet_repo.create(&pet).await.unwrap();
    assert_eq!(created.name, "Buddy");

    // Find pets by owner
    let pets = pet_repo.find_by_owner(&owner_id).await.unwrap();
    assert_eq!(pets.len(), 1);
    assert_eq!(pets[0].owner_id, owner_id);

    // Update pet
    let mut updated_pet = created.clone();
    updated_pet.weight_kg = Some(32.0);
    let updated = pet_repo.update(&updated_pet).await.unwrap();
    assert_eq!(updated.weight_kg, Some(32.0));
}

#[tokio::test]
async fn test_doctor_workflow() {
    let db = setup_test_db().await;
    let repo = DoctorRepository::new(db);

    // Create doctor
    let doctor = Doctor::new(
        "Dr. Test".to_string(),
        "doctor@clinic.com".to_string(),
        "+9876543210".to_string(),
        Specialization::GeneralPractice,
        "LIC-TEST-001".to_string(),
        10,
    )
    .unwrap();

    let created = repo.create(&doctor).await.unwrap();
    assert_eq!(created.name, "Dr. Test");
    assert!(created.is_available);

    // Find available doctors
    let available = repo.find_available().await.unwrap();
    assert_eq!(available.len(), 1);

    // Update availability
    let mut updated_doctor = created.clone();
    updated_doctor.set_availability(false);
    let updated = repo.update(&updated_doctor).await.unwrap();
    assert!(!updated.is_available);

    // Check available doctors again
    let available_after = repo.find_available().await.unwrap();
    assert_eq!(available_after.len(), 0);
}

#[tokio::test]
async fn test_health_check_workflow() {
    let db = setup_test_db().await;
    let user_repo = UserRepository::new(db.clone());
    let pet_repo = PetRepository::new(db.clone());
    let doctor_repo = DoctorRepository::new(db.clone());
    let check_repo = CheckRepository::new(db);

    // Create owner
    let owner = User::new("owner@test.com".to_string(), "Owner".to_string()).unwrap();
    let created_owner = user_repo.create(&owner).await.unwrap();
    let owner_id = created_owner.id.clone().unwrap();

    // Create pet
    let pet = Pet::new(owner_id, "Max".to_string(), PetSpecies::Dog).unwrap();
    let created_pet = pet_repo.create(&pet).await.unwrap();
    let pet_id = created_pet.id.clone().unwrap();

    // Create doctor
    let doctor = Doctor::new(
        "Dr. Vet".to_string(),
        "vet@clinic.com".to_string(),
        "+1111111111".to_string(),
        Specialization::GeneralPractice,
        "LIC-VET-001".to_string(),
        5,
    )
    .unwrap();
    let created_doctor = doctor_repo.create(&doctor).await.unwrap();
    let doctor_id = created_doctor.id.clone().unwrap();

    // Create health check
    let scheduled = Utc::now() + Duration::hours(2);
    let check = HealthCheck::new(pet_id.clone(), doctor_id.clone(), scheduled).unwrap();
    let created_check = check_repo.create(&check).await.unwrap();

    assert_eq!(created_check.status, CheckStatus::Scheduled);

    // Find checks by pet
    let pet_checks = check_repo.find_by_pet(&pet_id).await.unwrap();
    assert_eq!(pet_checks.len(), 1);

    // Find checks by doctor
    let doctor_checks = check_repo.find_by_doctor(&doctor_id).await.unwrap();
    assert_eq!(doctor_checks.len(), 1);

    // Start the check
    let mut started_check = created_check.clone();
    started_check.start().unwrap();
    let updated_check = check_repo.update(&started_check).await.unwrap();
    assert_eq!(updated_check.status, CheckStatus::InProgress);

    // Complete the check
    let mut completed_check = updated_check.clone();
    completed_check
        .complete(
            "Healthy".to_string(),
            Some("Vaccination".to_string()),
            Some(100.0),
        )
        .unwrap();
    let final_check = check_repo.update(&completed_check).await.unwrap();
    assert_eq!(final_check.status, CheckStatus::Completed);
    assert_eq!(final_check.diagnosis, Some("Healthy".to_string()));
    assert_eq!(final_check.cost, Some(100.0));
}

#[tokio::test]
async fn test_full_clinic_workflow() {
    let db = setup_test_db().await;

    // Run migrations with seed data
    let runner = MigrationRunner::new(db.clone());
    runner.run_with_seed().await.unwrap();

    // Verify seeded data
    let user_repo = UserRepository::new(db.clone());
    let pet_repo = PetRepository::new(db.clone());
    let doctor_repo = DoctorRepository::new(db.clone());
    let check_repo = CheckRepository::new(db);

    let users = user_repo.find_all().await.unwrap();
    assert_eq!(users.len(), 3);

    let pets = pet_repo.find_all().await.unwrap();
    assert_eq!(pets.len(), 4);

    let doctors = doctor_repo.find_all().await.unwrap();
    assert_eq!(doctors.len(), 3);

    let checks = check_repo.find_all().await.unwrap();
    assert_eq!(checks.len(), 4);

    // Verify relationships
    let first_user = &users[0];
    let first_user_id = first_user.id.as_ref().unwrap();
    let user_pets = pet_repo.find_by_owner(first_user_id).await.unwrap();
    assert!(!user_pets.is_empty());

    // Verify available doctors
    let available = doctor_repo.find_available().await.unwrap();
    assert!(!available.is_empty());
}

#[tokio::test]
async fn test_cascade_operations() {
    let db = setup_test_db().await;
    let user_repo = UserRepository::new(db.clone());
    let pet_repo = PetRepository::new(db);

    // Create user with multiple pets
    let owner = User::new("multi@test.com".to_string(), "Multi Pet Owner".to_string()).unwrap();
    let created_owner = user_repo.create(&owner).await.unwrap();
    let owner_id = created_owner.id.clone().unwrap();

    let pet1 = Pet::new(owner_id.clone(), "Pet 1".to_string(), PetSpecies::Dog).unwrap();
    let pet2 = Pet::new(owner_id.clone(), "Pet 2".to_string(), PetSpecies::Cat).unwrap();
    let pet3 = Pet::new(owner_id.clone(), "Pet 3".to_string(), PetSpecies::Bird).unwrap();

    pet_repo.create(&pet1).await.unwrap();
    pet_repo.create(&pet2).await.unwrap();
    pet_repo.create(&pet3).await.unwrap();

    // Verify all pets are created
    let owner_pets = pet_repo.find_by_owner(&owner_id).await.unwrap();
    assert_eq!(owner_pets.len(), 3);
}

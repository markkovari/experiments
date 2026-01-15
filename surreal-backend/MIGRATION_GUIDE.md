# UUID to Auto-Generated ID Migration Guide

## Overview

We're migrating from custom UUIDs to SurrealDB's auto-generated IDs to fix serialization issues.

## Changes Pattern

### 1. Domain Models (✅ COMPLETED)
All models now use:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub id: Option<String>,
```

### 2. Repository Pattern

**Before:**
```rust
async fn find_by_id(&self, id: Uuid) -> Result<Option<T>>;
async fn delete(&self, id: Uuid) -> Result<bool>;
async fn create(&self, entity: &T) -> Result<T> {
    self.db.client.create((TABLE, entity.id.to_string())).content(entity).await
}
```

**After:**
```rust
async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
async fn delete(&self, id: &str) -> Result<bool>;
async fn create(&self, entity: &T) -> Result<T> {
    self.db.client.create(TABLE).content(entity).await  // No ID specified!
}
```

### 3. Repository Tests

**Before:**
```rust
let owner_id = Uuid::new_v4();
let pet = Pet::new(owner_id, "Buddy".to_string(), PetSpecies::Dog).unwrap();
let created = repo.create(&pet).await.unwrap();
let found = repo.find_by_id(created.id).await.unwrap();
```

**After:**
```rust
let owner_id = "users:test123".to_string();
let pet = Pet::new(owner_id, "Buddy".to_string(), PetSpecies::Dog).unwrap();
let created = repo.create(&pet).await.unwrap();
let id = created.id.as_ref().unwrap();  // Extract the auto-generated ID
let found = repo.find_by_id(id).await.unwrap();
```

### 4. API DTOs

**Before:**
```rust
pub struct CreatePetRequest {
    pub owner_id: Uuid,
    // ...
}
```

**After:**
```rust
pub struct CreatePetRequest {
    pub owner_id: String,
    // ...
}
```

### 5. API Handlers

**Before:**
```rust
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    let user = repo.find_by_id(id).await?;
}
```

**After:**
```rust
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<User>> {
    let user = repo.find_by_id(&id).await?;
}
```

## Automated Fix Scripts

### Script 1: Update Remaining Repositories

Save this as `fix_repositories.sh`:

```bash
#!/bin/bash

# Fix Pet Repository Tests
sed -i '' 's/let owner_id = Uuid::new_v4();/let owner_id = "users:test123".to_string();/g' \
  crates/db/src/repository/pet.rs

sed -i '' 's/repo.find_by_id(pet.id)/repo.find_by_id(created.id.as_ref().unwrap())/g' \
  crates/db/src/repository/pet.rs

sed -i '' 's/repo.find_by_owner(owner_id)/repo.find_by_owner(\&owner_id)/g' \
  crates/db/src/repository/pet.rs

# Fix Doctor Repository
sed -i '' '1,/^use uuid::Uuid;/s/^use uuid::Uuid;$//' \
  crates/db/src/repository/doctor.rs

# Fix Check Repository
sed -i '' '1,/^use uuid::Uuid;/s/^use uuid::Uuid;$//' \
  crates/db/src/repository/check.rs

echo "Repositories updated! Review changes before committing."
```

### Script 2: Update API Handlers

Save this as `fix_api_handlers.sh`:

```bash
#!/bin/bash

# Update all handler Path parameters from Uuid to String
find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/Path(id): Path<Uuid>/Path(id): Path<String>/g' {} \;

# Update repository calls to use &id
find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/find_by_id(id)/find_by_id(\&id)/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/delete(id)/delete(\&id)/g' {} \;

echo "API handlers updated!"
```

### Script 3: Update DTOs

Save this as `fix_dtos.sh`:

```bash
#!/bin/bash

# Update all Uuid imports and types in DTOs
find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/use uuid::Uuid;//g' {} \;

find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/pub owner_id: Uuid,/pub owner_id: String,/g' {} \;

find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/pub pet_id: Uuid,/pub pet_id: String,/g' {} \;

find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/pub doctor_id: Uuid,/pub doctor_id: String,/g' {} \;

echo "DTOs updated!"
```

## Manual Updates Needed

### 1. Doctor Repository (`crates/db/src/repository/doctor.rs`)

Update the implementation following the User repository pattern:

```rust
// Remove: use uuid::Uuid;

#[async_trait]
impl Repository<Doctor> for DoctorRepository {
    async fn create(&self, doctor: &Doctor) -> Result<Doctor> {
        let created: Option<Doctor> = self.db.client
            .create(TABLE)  // Changed: no ID parameter
            .content(doctor.clone())
            .await?;
        created.ok_or_else(|| DbError::Other("Failed to create doctor".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Doctor>> {
        Ok(self.db.client.select((TABLE, id)).await?)
    }

    async fn update(&self, doctor: &Doctor) -> Result<Doctor> {
        let id = doctor.id.as_ref()
            .ok_or_else(|| DbError::Other("Doctor ID is required for update".to_string()))?;
        let updated: Option<Doctor> = self.db.client
            .update((TABLE, id.as_str()))
            .content(doctor.clone())
            .await?;
        updated.ok_or_else(|| DbError::NotFound(format!("Doctor with id {} not found", id)))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let deleted: Option<Doctor> = self.db.client.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}

// Update tests to use String IDs
#[tokio::test]
async fn test_create_and_find_doctor() {
    let db = Database::new_in_memory().await.unwrap();
    let repo = DoctorRepository::new(db);
    let doctor = Doctor::new(/*...*/).unwrap();
    let created = repo.create(&doctor).await.unwrap();

    let id = created.id.as_ref().unwrap();
    let found = repo.find_by_id(id).await.unwrap();
    // ...
}
```

### 2. Check Repository (`crates/db/src/repository/check.rs`)

Same pattern as Doctor repository.

### 3. Pet Repository Tests (`crates/db/src/repository/pet.rs`)

```rust
#[tokio::test]
async fn test_create_and_find_pet() {
    let db = Database::new_in_memory().await.unwrap();
    let repo = PetRepository::new(db);

    let owner_id = "users:test123".to_string();
    let pet = Pet::new(owner_id, "Buddy".to_string(), PetSpecies::Dog).unwrap();
    let created = repo.create(&pet).await.unwrap();

    assert_eq!(created.name, pet.name);
    assert!(created.id.is_some());

    let id = created.id.as_ref().unwrap();
    let found = repo.find_by_id(id).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_find_by_owner() {
    let db = Database::new_in_memory().await.unwrap();
    let repo = PetRepository::new(db);

    let owner_id = "users:test123".to_string();
    let pet1 = Pet::new(owner_id.clone(), "Max".to_string(), PetSpecies::Dog).unwrap();
    let pet2 = Pet::new(owner_id.clone(), "Whiskers".to_string(), PetSpecies::Cat).unwrap();

    repo.create(&pet1).await.unwrap();
    repo.create(&pet2).await.unwrap();

    let pets = repo.find_by_owner(&owner_id).await.unwrap();
    assert_eq!(pets.len(), 2);
}
```

### 4. Migrations Seed Data (`crates/migrations/src/seed.rs`)

Update to use created IDs:

```rust
// Create users and capture their IDs
let created_user1 = user_repo.create(&user1).await?;
let created_user2 = user_repo.create(&user2).await?;
let created_user3 = user_repo.create(&user3).await?;

// Use the auto-generated IDs for relationships
let pet1 = Pet::new(
    created_user1.id.clone().unwrap(),  // Use the created ID!
    "Buddy".to_string(),
    PetSpecies::Dog,
)?;

let created_pet1 = pet_repo.create(&pet1).await?;

// For health checks
let check1 = HealthCheck::new(
    created_pet1.id.clone().unwrap(),
    created_doctor1.id.clone().unwrap(),
    Utc::now() + Duration::days(3),
)?;
```

### 5. Integration Tests (`tests/integration/src/tests.rs`)

Replace all `Uuid::new_v4()` with string IDs and update to use created IDs:

```rust
#[tokio::test]
async fn test_user_workflow() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    let user = User::new("test@example.com".to_string(), "Test User".to_string()).unwrap();
    let created = repo.create(&user).await.unwrap();

    let id = created.id.as_ref().unwrap();
    let found = repo.find_by_id(id).await.unwrap();
    assert!(found.is_some());

    // For updates
    let mut updated_user = created.clone();
    updated_user.name = "Updated User".to_string();
    let updated = repo.update(&updated_user).await.unwrap();

    // For deletes
    let deleted = repo.delete(id).await.unwrap();
    assert!(deleted);
}
```

### 6. E2E Tests (`tests/e2e/src/tests.rs`)

Update to extract IDs from JSON responses:

```rust
// After creating a user
let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
let created_user: serde_json::Value = serde_json::from_slice(&body).unwrap();
let user_id = created_user["id"].as_str().unwrap();

// Use the extracted ID
let response = app
    .clone()
    .oneshot(
        Request::builder()
            .uri(format!("/users/{}", user_id))
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
```

## Checklist

- [ ] Run `./fix_repositories.sh`
- [ ] Run `./fix_api_handlers.sh`
- [ ] Run `./fix_dtos.sh`
- [ ] Manually update doctor repository implementation
- [ ] Manually update check repository implementation
- [ ] Update pet repository tests
- [ ] Update migrations seed data
- [ ] Update all integration tests
- [ ] Update all e2e tests
- [ ] Run `cargo test --package surreal-core --lib` (should pass)
- [ ] Run `cargo test --package surreal-db --lib` (should pass after repo fixes)
- [ ] Run `cargo nextest run` (should pass after all fixes)

## Quick Test Commands

```bash
# Test just core models (should already pass)
cargo test --package surreal-core --lib

# Test database layer (fix repositories first)
cargo test --package surreal-db --lib

# Test all with nextest
cargo nextest run

# Test specific files
cargo test --package surreal-db --lib repository::user::tests
cargo test --package surreal-db --lib repository::pet::tests
```

## Common Patterns to Find & Replace

Use these grep commands to find remaining issues:

```bash
# Find remaining Uuid usage
grep -r "Uuid::new_v4()" crates/ tests/

# Find remaining Uuid imports
grep -r "use uuid::Uuid" crates/db crates/api tests/

# Find places that need id.as_ref().unwrap()
grep -r "\.id)" crates/db/src/repository

# Find Path<Uuid> in handlers
grep -r "Path<Uuid>" crates/api/src/handlers
```

## Summary

The key principle: **Let SurrealDB generate IDs automatically**. Never specify an ID when creating records, and always extract IDs from created/returned records when you need to use them.

Good luck! Run the scripts, make the manual updates, and the tests should pass.

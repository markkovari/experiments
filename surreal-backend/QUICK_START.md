# Quick Start: Completing the UUID Migration

## Current Status

✅ **Completed:**
- Domain models updated (User, Pet, Doctor, HealthCheck)
- Core tests fixed (18/18 passing)
- User repository fully updated
- Repository trait updated

⚠️ **Needs Completion:**
- Doctor & Check repositories
- API handlers & DTOs
- Migrations seed data
- Integration & E2E tests

## Step-by-Step Guide

### Step 1: Run Automated Fixes (2 minutes)

```bash
cd /Users/markkovari/DEV/markkovari/experiments/surreal-backend
./scripts/run_all_fixes.sh
```

This will automatically update:
- API handlers (Path parameters)
- DTOs (Uuid → String)
- Pet repository tests (partially)

### Step 2: Manual Repository Updates (10 minutes)

Copy the pattern from `crates/db/src/repository/user.rs` to update:

**Doctor Repository** (`crates/db/src/repository/doctor.rs`):
```rust
// Line ~46: Update create
async fn create(&self, doctor: &Doctor) -> Result<Doctor> {
    let created: Option<Doctor> = self.db.client
        .create(TABLE)  // Remove ID parameter
        .content(doctor.clone())
        .await?;
    created.ok_or_else(|| DbError::Other("Failed to create doctor".to_string()))
}

// Line ~53: Update find_by_id
async fn find_by_id(&self, id: &str) -> Result<Option<Doctor>> {
    Ok(self.db.client.select((TABLE, id)).await?)
}

// Line ~62: Update update
async fn update(&self, doctor: &Doctor) -> Result<Doctor> {
    let id = doctor.id.as_ref()
        .ok_or_else(|| DbError::Other("Doctor ID required".to_string()))?;
    let updated: Option<Doctor> = self.db.client
        .update((TABLE, id.as_str()))
        .content(doctor.clone())
        .await?;
    updated.ok_or_else(|| DbError::NotFound(format!("Doctor {} not found", id)))
}

// Line ~71: Update delete
async fn delete(&self, id: &str) -> Result<bool> {
    let deleted: Option<Doctor> = self.db.client.delete((TABLE, id)).await?;
    Ok(deleted.is_some())
}
```

**Check Repository** (`crates/db/src/repository/check.rs`):
- Apply the same pattern as Doctor repository

**Repository Tests:**
Update tests in all repository files to extract IDs:
```rust
let created = repo.create(&entity).await.unwrap();
let id = created.id.as_ref().unwrap();
let found = repo.find_by_id(id).await.unwrap();
```

### Step 3: Update Migrations (5 minutes)

File: `crates/migrations/src/seed.rs`

Change relationships to use created IDs:
```rust
let created_user1 = user_repo.create(&user1).await?;
let created_pet1 = pet_repo.create(&Pet::new(
    created_user1.id.clone().unwrap(),  // Use auto-generated ID
    "Buddy".to_string(),
    PetSpecies::Dog
)?).await?;

let check1 = HealthCheck::new(
    created_pet1.id.clone().unwrap(),
    created_doctor1.id.clone().unwrap(),
    scheduled_time
)?;
```

### Step 4: Update Integration Tests (10 minutes)

File: `tests/integration/src/tests.rs`

Replace all `Uuid::new_v4()` with created IDs:
```rust
let user = User::new("test@example.com".to_string(), "User".to_string()).unwrap();
let created_user = user_repo.create(&user).await.unwrap();

let pet = Pet::new(
    created_user.id.clone().unwrap(),
    "Pet".to_string(),
    PetSpecies::Dog
).unwrap();
let created_pet = pet_repo.create(&pet).await.unwrap();

// Use created_pet.id.clone().unwrap() for health checks, etc.
```

### Step 5: Update E2E Tests (10 minutes)

File: `tests/e2e/src/tests.rs`

Already uses in-memory database, but needs ID extraction:
```rust
let response = app.clone().oneshot(/*create user*/).await.unwrap();
let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
let user_id = user["id"].as_str().unwrap();

// Use user_id for subsequent requests
```

## Verification Commands

After each step, run these to check progress:

```bash
# Test core models (should already pass)
cargo test --package surreal-core --lib

# Test database repositories
cargo test --package surreal-db --lib

# Check API compilation
cargo check --package surreal-api

# Test migrations
cargo test --package surreal-migrations --lib

# Run all tests
cargo nextest run
```

## Quick Reference

**Find remaining issues:**
```bash
# Find Uuid::new_v4() usage
grep -r "Uuid::new_v4()" crates/ tests/

# Find Uuid imports
grep -r "use uuid::Uuid" crates/db crates/api

# Find Path<Uuid> in handlers
grep -r "Path<Uuid>" crates/api
```

**Common patterns:**
```rust
// ❌ Old
let id = Uuid::new_v4();
.create((TABLE, id.to_string()))
Path(id): Path<Uuid>

// ✅ New
.create(TABLE)
let id = created.id.as_ref().unwrap();
Path(id): Path<String>
```

## Estimated Time

- **Automated scripts:** 2 minutes
- **Manual updates:** 35-40 minutes
- **Testing & fixes:** 10-15 minutes
- **Total:** ~1 hour

## Need Help?

See `MIGRATION_GUIDE.md` for detailed examples and patterns.

Good luck! 🚀

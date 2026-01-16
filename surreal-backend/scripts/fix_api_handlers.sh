#!/bin/bash
set -e

echo "🔧 Updating API Handlers..."

# Update all handler Path parameters from Uuid to String
find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/Path(id): Path<Uuid>/Path(id): Path<String>/g' {} \;

# Update repository calls to use &id
find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/\.find_by_id(id)/\.find_by_id(\&id)/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/\.delete(id)/\.delete(\&id)/g' {} \;

# Update owner_id, pet_id, doctor_id extractions
find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/Path(owner_id): Path<Uuid>/Path(owner_id): Path<String>/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/Path(pet_id): Path<Uuid>/Path(pet_id): Path<String>/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/Path(doctor_id): Path<Uuid>/Path(doctor_id): Path<String>/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/\.find_by_owner(owner_id)/\.find_by_owner(\&owner_id)/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/\.find_by_pet(pet_id)/\.find_by_pet(\&pet_id)/g' {} \;

find crates/api/src/handlers -name "*.rs" -exec sed -i '' \
  's/\.find_by_doctor(doctor_id)/\.find_by_doctor(\&doctor_id)/g' {} \;

echo "✅ API handlers updated!"
echo ""
echo "Run 'cargo check --package surreal-api' to verify changes"

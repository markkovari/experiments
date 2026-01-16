#!/bin/bash
set -e

echo "🔧 Updating API DTOs..."

# Remove Uuid imports from DTOs
find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  '/^use uuid::Uuid;$/d' {} \;

# Update all Uuid fields to String
find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/pub owner_id: Uuid,/pub owner_id: String,/g' {} \;

find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/pub pet_id: Uuid,/pub pet_id: String,/g' {} \;

find crates/api/src/dto -name "*.rs" -exec sed -i '' \
  's/pub doctor_id: Uuid,/pub doctor_id: String,/g' {} \;

echo "✅ DTOs updated!"
echo ""
echo "Run 'cargo check --package surreal-api' to verify changes"

pub const SCHEMA_DEFINITIONS: &[&str] = &[
    // Users table
    r#"
    DEFINE TABLE users SCHEMAFULL;
    DEFINE FIELD email ON users TYPE string ASSERT string::is::email($value);
    DEFINE FIELD name ON users TYPE string;
    DEFINE FIELD phone ON users TYPE option<string>;
    DEFINE FIELD address ON users TYPE option<string>;
    DEFINE FIELD created_at ON users TYPE datetime;
    DEFINE FIELD updated_at ON users TYPE datetime;
    DEFINE INDEX idx_users_email ON users COLUMNS email UNIQUE;
    "#,
    // Pets table
    r#"
    DEFINE TABLE pets SCHEMAFULL;
    DEFINE FIELD owner_id ON pets TYPE string;
    DEFINE FIELD name ON pets TYPE string;
    DEFINE FIELD species ON pets TYPE string;
    DEFINE FIELD breed ON pets TYPE option<string>;
    DEFINE FIELD birth_date ON pets TYPE option<string>;
    DEFINE FIELD weight_kg ON pets TYPE option<float>;
    DEFINE FIELD medical_notes ON pets TYPE option<string>;
    DEFINE FIELD created_at ON pets TYPE datetime;
    DEFINE FIELD updated_at ON pets TYPE datetime;
    DEFINE INDEX idx_pets_owner ON pets COLUMNS owner_id;
    "#,
    // Doctors table
    r#"
    DEFINE TABLE doctors SCHEMAFULL;
    DEFINE FIELD name ON doctors TYPE string;
    DEFINE FIELD email ON doctors TYPE string ASSERT string::is::email($value);
    DEFINE FIELD phone ON doctors TYPE string;
    DEFINE FIELD specialization ON doctors TYPE string;
    DEFINE FIELD license_number ON doctors TYPE string;
    DEFINE FIELD years_experience ON doctors TYPE int;
    DEFINE FIELD is_available ON doctors TYPE bool;
    DEFINE FIELD created_at ON doctors TYPE datetime;
    DEFINE FIELD updated_at ON doctors TYPE datetime;
    DEFINE INDEX idx_doctors_email ON doctors COLUMNS email UNIQUE;
    DEFINE INDEX idx_doctors_license ON doctors COLUMNS license_number UNIQUE;
    DEFINE INDEX idx_doctors_available ON doctors COLUMNS is_available;
    "#,
    // Health checks table
    r#"
    DEFINE TABLE health_checks SCHEMAFULL;
    DEFINE FIELD pet_id ON health_checks TYPE string;
    DEFINE FIELD doctor_id ON health_checks TYPE string;
    DEFINE FIELD scheduled_at ON health_checks TYPE datetime;
    DEFINE FIELD status ON health_checks TYPE string;
    DEFINE FIELD diagnosis ON health_checks TYPE option<string>;
    DEFINE FIELD treatment ON health_checks TYPE option<string>;
    DEFINE FIELD notes ON health_checks TYPE option<string>;
    DEFINE FIELD cost ON health_checks TYPE option<float>;
    DEFINE FIELD created_at ON health_checks TYPE datetime;
    DEFINE FIELD updated_at ON health_checks TYPE datetime;
    DEFINE INDEX idx_checks_pet ON health_checks COLUMNS pet_id;
    DEFINE INDEX idx_checks_doctor ON health_checks COLUMNS doctor_id;
    DEFINE INDEX idx_checks_status ON health_checks COLUMNS status;
    DEFINE INDEX idx_checks_scheduled ON health_checks COLUMNS scheduled_at;
    "#,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_definitions_not_empty() {
        assert!(!SCHEMA_DEFINITIONS.is_empty());
        assert_eq!(SCHEMA_DEFINITIONS.len(), 4);
    }
}

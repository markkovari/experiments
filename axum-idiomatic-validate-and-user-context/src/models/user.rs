use chrono::{DateTime, Utc};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Role {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "admin" => Ok(Role::Admin),
            _ => Ok(Role::User),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for Role {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <&str as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            role: u.role,
            created_at: u.created_at,
        }
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    Role,
    CreatedAt,
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    let cost = if cfg!(any(test, feature = "fast-hash")) {
        4
    } else {
        bcrypt::DEFAULT_COST
    };
    let password_hash = bcrypt::hash(password, cost)?;

    let (sql, values) = Query::insert()
        .into_table(Users::Table)
        .columns([Users::Username, Users::Email, Users::PasswordHash])
        .values_panic([username.into(), email.into(), password_hash.into()])
        .returning_all()
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, User, _>(&sql, values)
        .fetch_one(pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict("Username or email already exists".into())
            }
            _ => AppError::from(e),
        })
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, AppError> {
    let (sql, values) = Query::select()
        .columns([
            Users::Id,
            Users::Username,
            Users::Email,
            Users::PasswordHash,
            Users::Role,
            Users::CreatedAt,
        ])
        .from(Users::Table)
        .and_where(Expr::col(Users::Username).eq(username))
        .build_sqlx(PostgresQueryBuilder);

    let user = sqlx::query_as_with::<_, User, _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

#[cfg(test)]
mod tests {
    use sea_query::{PostgresQueryBuilder, Query};

    use super::Users;

    #[test]
    fn insert_query_builds() {
        let sql = Query::insert()
            .into_table(Users::Table)
            .columns([Users::Username, Users::Email, Users::PasswordHash])
            .values_panic(["alice".into(), "alice@test.com".into(), "hash".into()])
            .returning_all()
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("INSERT INTO"));
        assert!(sql.contains("\"users\""));
        assert!(sql.contains("RETURNING"));
    }

    #[test]
    fn select_by_username_builds() {
        use sea_query::Expr;

        let sql = Query::select()
            .columns([
                Users::Id,
                Users::Username,
                Users::Email,
                Users::PasswordHash,
                Users::Role,
                Users::CreatedAt,
            ])
            .from(Users::Table)
            .and_where(Expr::col(Users::Username).eq("alice"))
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM \"users\""));
        assert!(sql.contains("\"username\" = 'alice'"));
    }
}

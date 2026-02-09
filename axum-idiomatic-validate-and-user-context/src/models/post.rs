use chrono::{DateTime, Utc};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
    Title,
    Content,
    UserId,
    CreatedAt,
    UpdatedAt,
}

pub async fn create_post(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    content: &str,
) -> Result<Post, AppError> {
    let (sql, values) = Query::insert()
        .into_table(Posts::Table)
        .columns([Posts::Title, Posts::Content, Posts::UserId])
        .values_panic([title.into(), content.into(), user_id.into()])
        .returning_all()
        .build_sqlx(PostgresQueryBuilder);

    let post = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_one(pool)
        .await?;

    Ok(post)
}

pub async fn list_posts(pool: &PgPool, user_id: Uuid) -> Result<Vec<Post>, AppError> {
    let (sql, values) = Query::select()
        .columns([
            Posts::Id,
            Posts::Title,
            Posts::Content,
            Posts::UserId,
            Posts::CreatedAt,
            Posts::UpdatedAt,
        ])
        .from(Posts::Table)
        .and_where(Expr::col(Posts::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    let posts = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_all(pool)
        .await?;

    Ok(posts)
}

pub async fn get_post(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<Post>, AppError> {
    let (sql, values) = Query::select()
        .columns([
            Posts::Id,
            Posts::Title,
            Posts::Content,
            Posts::UserId,
            Posts::CreatedAt,
            Posts::UpdatedAt,
        ])
        .from(Posts::Table)
        .and_where(Expr::col(Posts::Id).eq(id))
        .and_where(Expr::col(Posts::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    let post = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(post)
}

pub async fn update_post(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    title: Option<&str>,
    content: Option<&str>,
) -> Result<Option<Post>, AppError> {
    if title.is_none() && content.is_none() {
        return get_post(pool, id, user_id).await;
    }

    let mut query = Query::update();
    query.table(Posts::Table);

    if let Some(t) = title {
        query.value(Posts::Title, t);
    }
    if let Some(c) = content {
        query.value(Posts::Content, c);
    }
    query.value(Posts::UpdatedAt, Expr::current_timestamp());
    query
        .and_where(Expr::col(Posts::Id).eq(id))
        .and_where(Expr::col(Posts::UserId).eq(user_id))
        .returning_all();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let post = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(post)
}

pub async fn list_all_posts(pool: &PgPool) -> Result<Vec<Post>, AppError> {
    let (sql, values) = Query::select()
        .columns([
            Posts::Id,
            Posts::Title,
            Posts::Content,
            Posts::UserId,
            Posts::CreatedAt,
            Posts::UpdatedAt,
        ])
        .from(Posts::Table)
        .build_sqlx(PostgresQueryBuilder);

    let posts = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_all(pool)
        .await?;

    Ok(posts)
}

pub async fn get_any_post(pool: &PgPool, id: Uuid) -> Result<Option<Post>, AppError> {
    let (sql, values) = Query::select()
        .columns([
            Posts::Id,
            Posts::Title,
            Posts::Content,
            Posts::UserId,
            Posts::CreatedAt,
            Posts::UpdatedAt,
        ])
        .from(Posts::Table)
        .and_where(Expr::col(Posts::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    let post = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(post)
}

pub async fn update_any_post(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    content: Option<&str>,
) -> Result<Option<Post>, AppError> {
    if title.is_none() && content.is_none() {
        return get_any_post(pool, id).await;
    }

    let mut query = Query::update();
    query.table(Posts::Table);

    if let Some(t) = title {
        query.value(Posts::Title, t);
    }
    if let Some(c) = content {
        query.value(Posts::Content, c);
    }
    query.value(Posts::UpdatedAt, Expr::current_timestamp());
    query
        .and_where(Expr::col(Posts::Id).eq(id))
        .returning_all();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let post = sqlx::query_as_with::<_, Post, _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(post)
}

pub async fn delete_any_post(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let (sql, values) = Query::delete()
        .from_table(Posts::Table)
        .and_where(Expr::col(Posts::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    let result = sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_post(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
    let (sql, values) = Query::delete()
        .from_table(Posts::Table)
        .and_where(Expr::col(Posts::Id).eq(id))
        .and_where(Expr::col(Posts::UserId).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);

    let result = sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use sea_query::{Expr, PostgresQueryBuilder, Query};
    use uuid::Uuid;

    use super::Posts;

    #[test]
    fn insert_query_builds() {
        let user_id = Uuid::new_v4();
        let sql = Query::insert()
            .into_table(Posts::Table)
            .columns([Posts::Title, Posts::Content, Posts::UserId])
            .values_panic(["hello".into(), "world".into(), user_id.into()])
            .returning_all()
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("INSERT INTO"));
        assert!(sql.contains("\"posts\""));
        assert!(sql.contains("RETURNING"));
    }

    #[test]
    fn select_by_user_builds() {
        let user_id = Uuid::new_v4();
        let sql = Query::select()
            .columns([
                Posts::Id,
                Posts::Title,
                Posts::Content,
                Posts::UserId,
                Posts::CreatedAt,
                Posts::UpdatedAt,
            ])
            .from(Posts::Table)
            .and_where(Expr::col(Posts::UserId).eq(user_id))
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM \"posts\""));
        assert!(sql.contains("\"user_id\""));
    }

    #[test]
    fn delete_query_builds() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let sql = Query::delete()
            .from_table(Posts::Table)
            .and_where(Expr::col(Posts::Id).eq(id))
            .and_where(Expr::col(Posts::UserId).eq(user_id))
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("DELETE FROM"));
        assert!(sql.contains("\"posts\""));
        assert!(sql.contains("\"id\""));
    }

    #[test]
    fn update_query_builds() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut query = Query::update();
        query.table(Posts::Table);
        query.value(Posts::Title, "new title");
        query.value(Posts::UpdatedAt, Expr::current_timestamp());
        query
            .and_where(Expr::col(Posts::Id).eq(id))
            .and_where(Expr::col(Posts::UserId).eq(user_id))
            .returning_all();

        let sql = query.to_string(PostgresQueryBuilder);

        assert!(sql.contains("UPDATE \"posts\""));
        assert!(sql.contains("SET"));
        assert!(sql.contains("\"title\""));
        assert!(sql.contains("RETURNING"));
    }

    #[test]
    fn select_all_posts_builds() {
        let sql = Query::select()
            .columns([
                Posts::Id,
                Posts::Title,
                Posts::Content,
                Posts::UserId,
                Posts::CreatedAt,
                Posts::UpdatedAt,
            ])
            .from(Posts::Table)
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM \"posts\""));
        assert!(!sql.contains("WHERE"));
    }

    #[test]
    fn select_any_post_by_id_builds() {
        let id = Uuid::new_v4();
        let sql = Query::select()
            .columns([
                Posts::Id,
                Posts::Title,
                Posts::Content,
                Posts::UserId,
                Posts::CreatedAt,
                Posts::UpdatedAt,
            ])
            .from(Posts::Table)
            .and_where(Expr::col(Posts::Id).eq(id))
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("\"id\""));
        assert!(!sql.contains("\"user_id\" ="));
    }

    #[test]
    fn delete_any_post_query_builds() {
        let id = Uuid::new_v4();
        let sql = Query::delete()
            .from_table(Posts::Table)
            .and_where(Expr::col(Posts::Id).eq(id))
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("DELETE FROM"));
        assert!(sql.contains("\"id\""));
        assert!(!sql.contains("\"user_id\""));
    }
}

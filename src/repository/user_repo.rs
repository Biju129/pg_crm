use crate::models::user::User;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    pub async fn create<'e, E>(
        executor: E,
        username_or_login: &str,
        password_hash: &str,
        role: &str,
        tenant_id: Option<Uuid>,
        is_active: bool,
    ) -> Result<User, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username_or_login, password_hash, role, tenant_id, is_active)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username_or_login, password_hash, role, tenant_id, is_active, must_change_password, last_login_at, created_at
            "#,
        )
        .bind(username_or_login)
        .bind(password_hash)
        .bind(role)
        .bind(tenant_id)
        .bind(is_active)
        .fetch_one(executor)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(pool: &sqlx::PgPool, username_or_login: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username_or_login, password_hash, role, tenant_id, is_active, must_change_password, last_login_at, created_at
            FROM users
            WHERE username_or_login = $1
            "#,
        )
        .bind(username_or_login)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username_or_login, password_hash, role, tenant_id, is_active, must_change_password, last_login_at, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}

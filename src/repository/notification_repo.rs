use crate::models::notification::Notification;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct NotificationRepository;

impl NotificationRepository {
    pub async fn queue<'e, E>(
        executor: E,
        tenant_id: Uuid,
        notification_type: &str,
        channel: &str,
        message_reference: Option<&str>,
    ) -> Result<Notification, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (tenant_id, notification_type, channel, message_reference, status)
            VALUES ($1, $2, $3, $4, 'QUEUED')
            RETURNING
                id, tenant_id, notification_type, channel, message_reference,
                status, sent_at, created_at
            "#,
        )
        .bind(tenant_id)
        .bind(notification_type)
        .bind(channel)
        .bind(message_reference)
        .fetch_one(executor)
        .await?;

        Ok(notification)
    }

    pub async fn find_all(pool: &sqlx::PgPool) -> Result<Vec<Notification>, sqlx::Error> {
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, tenant_id, notification_type, channel, message_reference, status, sent_at, created_at
            FROM notifications
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(notifications)
    }

    pub async fn find_by_tenant(
        pool: &sqlx::PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, tenant_id, notification_type, channel, message_reference, status, sent_at, created_at
            FROM notifications
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(notifications)
    }

    pub async fn mark_sent(pool: &sqlx::PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE notifications SET status = 'SENT', sent_at = CURRENT_TIMESTAMP WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

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
}

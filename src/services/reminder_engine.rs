use crate::db::DbPool;
use crate::models::notification::Notification;
use crate::repository::{NotificationRepository, RentLedgerRepository, TenantRepository};
use chrono::Utc;
use serde::Serialize;

pub struct ReminderEngine;

#[derive(Debug, Clone, Serialize)]
pub struct ReminderRunSummary {
    pub overdue_marked: u64,
    pub due_reminders_queued: usize,
    pub overdue_reminders_queued: usize,
    pub notifications_processed: usize,
}

impl ReminderEngine {
    pub async fn run_reminder_cycle(pool: &DbPool) -> Result<ReminderRunSummary, String> {
        let overdue_marked = RentLedgerRepository::mark_overdue_items(pool)
            .await
            .map_err(|e| e.to_string())?;

        let all_ledgers = RentLedgerRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        let today = Utc::now().date_naive();
        let mut due_reminders_queued = 0;
        let mut overdue_reminders_queued = 0;

        for ledger in all_ledgers {
            if ledger.payment_status == "PAID" {
                continue;
            }

            let tenant = TenantRepository::find_by_id(pool, ledger.tenant_id)
                .await
                .map_err(|e| e.to_string())?;

            let Some(t) = tenant else { continue };

            // Check if reminder was already sent today
            if let Some(last_sent) = ledger.last_reminder_sent_at {
                if last_sent.date_naive() == today {
                    continue;
                }
            }

            let mut queued = false;

            // 1. Due Today Reminder (e.g. 7th)
            if ledger.due_date == today {
                let msg_ref = format!(
                    "DUE_REMINDER:{}:₹{}",
                    t.tenant_id.as_deref().unwrap_or("TNT"),
                    ledger.pending_amount
                );
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                NotificationRepository::queue(
                    &mut *tx,
                    ledger.tenant_id,
                    "RENT_DUE",
                    "WHATSAPP",
                    Some(&msg_ref),
                )
                .await
                .map_err(|e| e.to_string())?;
                tx.commit().await.map_err(|e| e.to_string())?;

                due_reminders_queued += 1;
                queued = true;
            }
            // 2. Overdue Daily Reminder (8th onwards)
            else if ledger.due_date < today && ledger.pending_amount > 0.0 {
                let msg_ref = format!(
                    "OVERDUE_REMINDER:{}:₹{}",
                    t.tenant_id.as_deref().unwrap_or("TNT"),
                    ledger.pending_amount
                );
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                NotificationRepository::queue(
                    &mut *tx,
                    ledger.tenant_id,
                    "RENT_REMINDER",
                    "WHATSAPP",
                    Some(&msg_ref),
                )
                .await
                .map_err(|e| e.to_string())?;
                tx.commit().await.map_err(|e| e.to_string())?;

                overdue_reminders_queued += 1;
                queued = true;
            }

            if queued {
                let _ = RentLedgerRepository::update_reminder_sent(pool, ledger.id).await;
            }
        }

        // Auto-process queued notifications
        let notifications = NotificationRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut processed = 0;
        for n in notifications {
            if n.status == "QUEUED" {
                let _ = NotificationRepository::mark_sent(pool, n.id).await;
                processed += 1;
            }
        }

        tracing::info!(
            "Reminder Engine Completed: {} marked overdue, {} due queued, {} overdue queued, {} sent",
            overdue_marked,
            due_reminders_queued,
            overdue_reminders_queued,
            processed
        );

        Ok(ReminderRunSummary {
            overdue_marked,
            due_reminders_queued,
            overdue_reminders_queued,
            notifications_processed: processed,
        })
    }

    pub async fn list_notifications(pool: &DbPool) -> Result<Vec<Notification>, String> {
        NotificationRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())
    }
}

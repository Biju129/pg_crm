use crate::db::DbPool;
use crate::models::tenant_document::{TenantDocument, UploadTenantDocumentDto};
use uuid::Uuid;

pub struct TenantDocumentRepository;

impl TenantDocumentRepository {
    pub async fn create(
        pool: &DbPool,
        dto: &UploadTenantDocumentDto,
    ) -> Result<TenantDocument, sqlx::Error> {
        let doc = sqlx::query_as::<_, TenantDocument>(
            r#"
            INSERT INTO tenant_documents (tenant_id, document_type, file_name, file_url_or_path)
            VALUES ($1, $2, $3, $4)
            RETURNING id, tenant_id, document_type, file_name, file_url_or_path, uploaded_at
            "#,
        )
        .bind(dto.tenant_id)
        .bind(&dto.document_type)
        .bind(&dto.file_name)
        .bind(&dto.file_url_or_path)
        .fetch_one(pool)
        .await?;

        Ok(doc)
    }

    pub async fn find_by_tenant(
        pool: &DbPool,
        tenant_id: Uuid,
    ) -> Result<Vec<TenantDocument>, sqlx::Error> {
        let docs = sqlx::query_as::<_, TenantDocument>(
            r#"
            SELECT id, tenant_id, document_type, file_name, file_url_or_path, uploaded_at
            FROM tenant_documents
            WHERE tenant_id = $1
            ORDER BY uploaded_at DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(docs)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tenant_documents WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

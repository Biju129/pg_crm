use crate::db::DbPool;
use crate::models::tenant_document::{TenantDocument, UploadTenantDocumentDto};
use crate::repository::TenantDocumentRepository;
use uuid::Uuid;

pub struct DocumentService;

impl DocumentService {
    pub async fn upload_document(
        pool: &DbPool,
        dto: UploadTenantDocumentDto,
    ) -> Result<TenantDocument, String> {
        if dto.file_name.trim().is_empty() {
            return Err("File name is required".to_string());
        }
        if dto.file_url_or_path.trim().is_empty() {
            return Err("File URL or path is required".to_string());
        }

        TenantDocumentRepository::create(pool, &dto)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_tenant_documents(
        pool: &DbPool,
        tenant_id: Uuid,
    ) -> Result<Vec<TenantDocument>, String> {
        TenantDocumentRepository::find_by_tenant(pool, tenant_id)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_document(pool: &DbPool, id: Uuid) -> Result<bool, String> {
        TenantDocumentRepository::delete(pool, id)
            .await
            .map_err(|e| e.to_string())
    }
}

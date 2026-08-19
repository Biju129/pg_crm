use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    IdProof,
    AddressProof,
    Other,
}

impl ToString for DocumentType {
    fn to_string(&self) -> String {
        match self {
            DocumentType::IdProof => "ID_PROOF".to_string(),
            DocumentType::AddressProof => "ADDRESS_PROOF".to_string(),
            DocumentType::Other => "OTHER".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantDocument {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub document_type: String,
    pub file_name: String,
    pub file_url_or_path: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadTenantDocumentDto {
    pub tenant_id: Uuid,
    pub document_type: String,
    pub file_name: String,
    pub file_url_or_path: String,
}

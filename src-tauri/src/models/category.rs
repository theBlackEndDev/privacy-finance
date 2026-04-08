use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    #[serde(rename = "type")]
    pub category_type: String,
    pub sort_order: i64,
    pub is_archived: bool,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryGroup {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryInput {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub category_type: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub category_type: Option<String>,
    pub sort_order: Option<i64>,
    pub is_archived: Option<bool>,
    pub group_id: Option<Option<String>>,
}

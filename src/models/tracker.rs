use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "tracker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub code: String,
    pub status: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
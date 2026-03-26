use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use serde::{Deserialize, Serialize};

use crate::models::settings;

pub const KEY_THEME: &str = "theme";
pub const KEY_ARTICLE_FULL_MODE_PROXY: &str = "article_full_mode_proxy";

pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub article_full_mode_proxy: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: THEME_DARK.to_string(),
            // Default on: Full Mode should use the in-app proxy unless the user turns it off.
            article_full_mode_proxy: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppSettings {
    pub theme: Option<String>,
    pub article_full_mode_proxy: Option<bool>,
}

async fn get_raw(db: &DatabaseConnection, key: &str) -> Result<Option<String>, DbErr> {
    Ok(settings::Entity::find()
        .filter(settings::Column::Key.eq(key))
        .one(db)
        .await?
        .map(|m| m.value))
}

async fn upsert(db: &DatabaseConnection, key: &str, value: &str) -> Result<(), DbErr> {
    if let Some(model) = settings::Entity::find()
        .filter(settings::Column::Key.eq(key))
        .one(db)
        .await?
    {
        let mut active: settings::ActiveModel = model.into();
        active.value = Set(value.to_string());
        active.update(db).await?;
    } else {
        settings::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

fn parse_bool(s: &str) -> bool {
    s == "true" || s == "1"
}

pub async fn get_app_settings(db: &DatabaseConnection) -> Result<AppSettings, DbErr> {
    let mut out = AppSettings::default();
    if let Some(v) = get_raw(db, KEY_THEME).await? {
        if v == THEME_LIGHT || v == THEME_DARK {
            out.theme = v;
        }
    }
    if let Some(v) = get_raw(db, KEY_ARTICLE_FULL_MODE_PROXY).await? {
        out.article_full_mode_proxy = parse_bool(&v);
    }
    Ok(out)
}

pub async fn update_app_settings(
    db: &DatabaseConnection,
    patch: UpdateAppSettings,
) -> Result<AppSettings, DbErr> {
    if let Some(ref t) = patch.theme {
        if *t == THEME_LIGHT || *t == THEME_DARK {
            upsert(db, KEY_THEME, t).await?;
        }
    }
    if let Some(b) = patch.article_full_mode_proxy {
        upsert(
            db,
            KEY_ARTICLE_FULL_MODE_PROXY,
            if b { "true" } else { "false" },
        )
        .await?;
    }
    get_app_settings(db).await
}

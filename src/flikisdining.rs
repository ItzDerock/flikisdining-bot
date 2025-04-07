// hide dead code warnings
#![allow(dead_code)]

use crate::env::SCHOOL_KEY;
use chrono::Datelike;
use chrono::{DateTime, Utc};
use http_cache_quickcache::QuickManager;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Default, Clone, Debug)]
pub struct FlikIsDiningNutritionInfo {
    pub calories: Option<f32>,
    pub raw_calories: Option<f32>,
    pub g_fat: Option<f32>,
    pub g_saturated_fat: Option<f32>,
    pub g_trans_fat: Option<f32>,
    pub mg_cholesterol: Option<f32>,
    pub g_carbs: Option<f32>,
    pub g_sugar: Option<f32>,
    pub g_added_sugar: Option<f32>,
    pub mg_sodium: Option<f32>,
    pub g_protein: Option<f32>,
    pub mg_iron: Option<f32>,
    pub mg_calcium: Option<f32>,
    pub mg_vitamin_c: Option<f32>,
    pub iu_vitamin_a: Option<f32>,
    pub re_vitamin_a: Option<f32>,
    pub mg_vitamin_d: Option<f32>,
    pub mg_potassium: Option<f32>,
    pub g_fiber: Option<f32>,
    pub mcg_vitamin_a: Option<f32>,
    pub mcg_vitamin_d: Option<f32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FlikIsDiningServingSizeInfo {
    pub serving_size_amount: String,
    pub serving_size_unit: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FlikIsDiningFood {
    pub id: f32,
    pub name: String,
    pub ingredients: Option<String>,
    // pub description: Option<String>,
    // pub synced_ingredients: Option<String>,
    pub rounded_nutrition_info: Option<FlikIsDiningNutritionInfo>,
    pub serving_size_info: Option<FlikIsDiningServingSizeInfo>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FlikIsDiningMenuItem {
    pub id: f32,
    pub position: f32,
    pub bold: bool,
    pub text: String,
    pub image: Option<String>,
    pub image_thumbnail: Option<String>,
    pub food: Option<FlikIsDiningFood>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FlikIsDiningDay {
    /// yyyy-mm-dd
    pub date: String,
    pub has_unpublished_menus: bool,
    pub menu_info: Option<serde_json::Value>,
    pub menu_items: Vec<FlikIsDiningMenuItem>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FlikIsDiningResponse {
    pub start_date: Option<String>,
    pub menu_type_id: Option<f32>,
    pub days: Vec<FlikIsDiningDay>,
    pub last_updated: Option<String>,
    pub id: Option<f32>,
    pub bold_all_entrees_enabled: Option<bool>,
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error(transparent)]
    RequestFailed(#[from] reqwest::Error),

    #[error(transparent)]
    RequestMiddlewareFailed(#[from] reqwest_middleware::Error),

    #[error("JSON Parsing failed: {0}")]
    JsonParseFailed(#[from] serde_json::Error),

    #[error("Response body read failed: {0}")]
    BodyReadFailed(reqwest::Error),

    #[error("No lunch found for date {0}")]
    NoLunchForDate(String),

    #[error("Received non-success status code: {0}")]
    HttpStatusError(reqwest::StatusCode),
}

// create the http client
static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: QuickManager::default(),
            options: HttpCacheOptions::default(),
        }))
        .build()
});

pub async fn fetch_week_lunch(date: DateTime<Utc>) -> Result<Vec<FlikIsDiningDay>, FetchError> {
    // create the URL
    let url = format!(
        "https://{}.api.flikisdining.com/menu/api/weeks/school/kentucky-country-day-school/menu-type/lunch/{}/{}/{}/?format=json",
        *SCHOOL_KEY, date.year(), date.month(), date.day()
    );

    println!("Fetching lunch from {}", url);

    // fetch the data
    let response = CLIENT.get(&url).send().await?;

    let status = response.status();
    if !status.is_success() {
        eprintln!(
            "Request failed with status: {}. Response text: {:?}",
            status,
            response.text().await.ok()
        );
        return Err(FetchError::HttpStatusError(status));
    }

    // read body as text
    let response_text = response.text().await.map_err(FetchError::BodyReadFailed)?;

    // Attempt to parse the text
    let response_data: FlikIsDiningResponse =
        serde_json::from_str(&response_text).map_err(|e| {
            eprintln!("Failed to parse JSON: {}", e);
            eprintln!("Response Text was:\n{}", response_text);
            FetchError::JsonParseFailed(e)
        })?;

    // for each day, filter so only food items are left
    let days = response_data
        .days
        .into_iter()
        .map(|day| {
            let menu_items = day
                .menu_items
                .into_iter()
                .filter(|item| item.food.is_some())
                .collect::<Vec<FlikIsDiningMenuItem>>();

            FlikIsDiningDay { menu_items, ..day }
        })
        .collect::<Vec<FlikIsDiningDay>>();

    // return the response
    Ok(days)
}

pub async fn fetch_lunch(date: DateTime<Utc>) -> Result<Vec<FlikIsDiningMenuItem>, FetchError> {
    // get the week
    let week = fetch_week_lunch(date).await?;

    let date_str = date.format("%Y-%m-%d").to_string();

    // find today's lunch
    let today = week.into_iter().find(|day| day.date == date_str);

    // if there was no lunch, return an error
    match today {
        Some(day) => Ok(day.menu_items),
        None => Err(FetchError::NoLunchForDate(date_str)),
    }
}

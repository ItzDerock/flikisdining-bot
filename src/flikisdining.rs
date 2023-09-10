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

#[derive(Deserialize, Default, Clone)]
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
}

#[derive(Deserialize, Clone)]
pub struct FlikIsDiningServingSizeInfo {
    pub serving_size_amount: String,
    pub serving_size_unit: String,
}

#[derive(Deserialize, Clone)]
pub struct FlikIsDiningFood {
    pub id: f32,
    pub name: String,
    pub ingredients: Option<String>,

    pub rounded_nutrition_info: Option<FlikIsDiningNutritionInfo>,
    pub serving_size_info: Option<FlikIsDiningServingSizeInfo>,
}

#[derive(Deserialize, Clone)]
pub struct FlikIsDiningMenuItem {
    pub position: f32,
    pub bold: bool,
    pub text: String,
    pub image: Option<String>,
    pub image_thumbnail: Option<String>,

    pub food: Option<FlikIsDiningFood>,
}

#[derive(Deserialize, Clone)]
pub struct FlikIsDiningDay {
    /// yyyy-mm-dd
    pub date: String,
    pub has_unpublished_menus: bool,
    // menu_info: Option<serde::Value>,
    pub menu_items: Vec<FlikIsDiningMenuItem>,
}

#[derive(Deserialize, Clone)]
pub struct FlikIsDiningResponse {
    pub start_date: Option<String>,
    pub menu_type_id: Option<f32>,

    pub days: Vec<FlikIsDiningDay>,
    pub last_updated: Option<String>,
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error(transparent)]
    RequestFailed(#[from] reqwest::Error),

    #[error(transparent)]
    RequestMiddlewareFailed(#[from] reqwest_middleware::Error),

    #[error("No lunch today")]
    NoLunchToday,
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
    ).to_owned();

    println!("Fetching lunch from {}", url);

    // fetch the data
    let response: FlikIsDiningResponse = CLIENT
        .get(&url)
        .send()
        .await?
        .json::<FlikIsDiningResponse>()
        .await?;

    // for each day, filter so only food items are left
    let days = response
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

    // find today's lunch
    let today = week
        .into_iter()
        .find(|day| day.date == date.format("%Y-%m-%d").to_string());

    // if there was no lunch, return an error
    if today.is_none() {
        return Err(FetchError::NoLunchToday);
    }

    // return the response
    Ok(today.unwrap().menu_items)
}

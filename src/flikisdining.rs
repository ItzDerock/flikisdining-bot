// hide dead code warnings
#![allow(dead_code)]

use chrono::Datelike;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::Error;
use std::env;

#[derive(Deserialize, Default)]
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

#[derive(Deserialize)]
pub struct FlikIsDiningServingSizeInfo {
    pub serving_size_amount: String,
    pub serving_size_unit: String,
}

#[derive(Deserialize)]
pub struct FlikIsDiningFood {
    pub id: f32,
    pub name: String,
    pub ingredients: Option<String>,

    pub rounded_nutrition_info: Option<FlikIsDiningNutritionInfo>,
    pub serving_size_info: Option<FlikIsDiningServingSizeInfo>,
}

#[derive(Deserialize)]
pub struct FlikIsDiningMenuItem {
    pub position: f32,
    pub bold: bool,
    pub text: String,
    pub image: Option<String>,
    pub image_thumbnail: Option<String>,

    pub food: Option<FlikIsDiningFood>,
}

#[derive(Deserialize)]
pub struct FlikIsDiningDay {
    /// yyyy-mm-dd
    pub date: String,
    pub has_unpublished_menus: bool,
    // menu_info: Option<serde::Value>,
    pub menu_items: Vec<FlikIsDiningMenuItem>,
}

#[derive(Deserialize)]
pub struct FlikIsDiningResponse {
    pub start_date: Option<String>,
    pub menu_type_id: Option<f32>,

    pub days: Vec<FlikIsDiningDay>,
    pub last_updated: Option<String>,
}

pub async fn fetch_lunch(date: DateTime<Utc>) -> Result<Vec<FlikIsDiningMenuItem>, Error> {
    // load the base API path
    static SCHOOL_KEY: Lazy<String> = Lazy::new(|| {
        env::var("API_SCHOOL_KEY")
            .ok()
            .expect("Expected API_SCHOOL_KEY in the environment")
    });

    // get the month, day, and year
    let month = date.month();
    let day = date.day();
    let year = date.year();

    // create the url
    let url = format!(
        "https://{}.api.flikisdining.com/menu/api/weeks/school/kentucky-country-day-school/menu-type/lunch/{}/{}/{}/?format=json",
        *SCHOOL_KEY, year, month, day
    ).to_owned();

    println!("Fetching lunch from {}", url);

    // fetch the data
    let response: FlikIsDiningResponse = reqwest::get(&url)
        .await?
        .json::<FlikIsDiningResponse>()
        .await?;

    // find today's lunch
    let today = response
        .days
        .into_iter()
        .find(|day| day.date == date.format("%Y-%m-%d").to_string());

    // if there was no lunch, return an error
    if today.is_none() {
        return Err(Error::Other("No lunch today"));
    }

    // filter out items without FlikIsDiningFood
    let today = today
        .unwrap()
        .menu_items
        .into_iter()
        .filter(|item| item.food.is_some())
        .collect::<Vec<FlikIsDiningMenuItem>>();

    // return the response
    Ok(today)
}
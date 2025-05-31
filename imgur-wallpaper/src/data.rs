use std::{error::Error, fmt::Display};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Wallpaper {
    pub url: String,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct TopDataItem {
    pub data: Wallpaper,
}

#[derive(Deserialize, Debug)]
pub struct TopData {
    pub children: Vec<TopDataItem>,
}

#[derive(Deserialize, Debug)]
pub struct RedditData {
    pub data: TopData,
}

#[derive(Debug)]
pub struct GetFileNameError {
    pub message: String,
}

impl Display for GetFileNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct DownloadImageError {
    pub message: String,
}

impl Display for DownloadImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for DownloadImageError {}

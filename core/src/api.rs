use reqwest::{blocking::Client, blocking::Response, header::CONTENT_TYPE, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap};
use std::io::{self, Read};

// Fileters for API
const MIN_FILTER: &str = "id,names,series";
const PLAYER_FILTER: &str = "playlist";
#[derive(Serialize, Deserialize)]
pub struct ServerResponseMessage {
    pub status: bool,
    pub data: Data,
    pub error: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Data {
    items: Vec<Title>,
    pagination: Pagination,
}
#[derive(Serialize, Deserialize)]
pub struct Title {
    pub names: [String; 2],
    pub id: u32,
    #[serde(default)]
    pub playlist: Vec<PlaylistItem>,
    pub series: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: f32, //don't ask me why its float, ask the dude which wrote the API...
    pub name: Option<String>,
    pub title: String,
    pub skips: Skips,
    pub sd: Option<String>,
    pub hd: Option<String>,
    pub fullhd: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Skips {
    pub ending: Vec<u32>,
    pub opening: Vec<u32>,
}
#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub perPage: u32,
    pub allPages: u32,
}
// Fetches list of items for provided page, ordered by update time
// Each page contains 9 elements
pub fn fetch_updates_list(
    page: u32,
    url: &String,
    client: &Client,
) -> Result<(Pagination, Vec<Title>), reqwest::Error> {
    let resp = client
        .post(url)
        .body(format!(
            "query=list&perPage=9&page={page}&filter={MIN_FILTER}"
        ))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()?;
    if resp.status() != StatusCode::OK {
        return Err(resp.error_for_status().err().unwrap());
    }
    return Ok(process_list_response(resp));
}
pub fn search_title(
    name: &String,
    page: u32,
    url: &String,
    client: &Client,
) -> Result<Vec<Title>, reqwest::Error> {
    let response = client
        .post(url)
        .body(format!(
            "query=search&search={name}&perPage=9&page={page}&filter={MIN_FILTER}"
        ))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()?;
    if response.status() != StatusCode::OK {
        return Err(response.error_for_status().err().unwrap());
    }
    return Ok(process_search_response(response));
}
pub fn get_title_playlist(
    id: u32,
    client: &Client,
    url: &String,
) -> Result<Vec<PlaylistItem>, reqwest::Error> {
    let response = client
        .post(url)
        .body(format!("query=release&id={id}&filter={PLAYER_FILTER}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()?;
    if response.status() != StatusCode::OK {
        return Err(response.error_for_status().err().unwrap());
    }
    let mut content: serde_json::Value = response.json()?;
    let playlist = content["data"]["playlist"].take();
    return Ok(serde_json::from_value(playlist).expect("api problem"));
}
// Parses "recent releases" response
fn process_list_response(response: Response) -> (Pagination, Vec<Title>) {
    let content: ServerResponseMessage = response.json().expect("error when parsing");
    if !content.status {
        println!(
            "{}",
            content.error.unwrap_or("critical api error".to_string())
        );
        _ = io::stdin().read(&mut [0u8]);
        panic!();
    }
    return (content.data.pagination, content.data.items);
}
// Parses search API response
fn process_search_response(response: Response) -> Vec<Title> {
    let mut weakContent: serde_json::Value = response.json().expect("Error parsing search api");
    return serde_json::from_value(weakContent["data"].take())
        .expect("problem parsing search api response");
}

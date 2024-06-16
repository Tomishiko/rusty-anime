use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::error;
use serde_json::Value;



const API_HOST:&str = "https://api.anilibria.tv/v3";

#[derive(Serialize, Deserialize)]
pub struct Title {
    pub names: Name,
    pub id: i32,
    pub player: Value,
}
#[derive(Serialize, Deserialize)]
pub struct Name {
    pub ru: String,
    pub en: String,
}
#[derive(Serialize, Deserialize)]
pub struct Response {
    pub list: Vec<Title>,
}

pub fn fetch_updates_list(page:u8) -> Result<Vec<Title>,reqwest::Error>{
    //println!("Fetching releases");
    let resp = reqwest::blocking::get(
        format!("{API_HOST}/v3/title/updates?filter=names,player,list,id&limit=9&page={page}"),
    )?;
    if resp.status()!= StatusCode::OK{
        //self.out_handle.write_fmt(format_args!("Failed to fetch, status code {}\n",resp.status()));
        return  Err(resp.error_for_status().err().unwrap());
    }
    //dbg!(&resp);
    return Ok(process_server_response_body(resp));
}
pub fn search_title(name: &String) -> Result<Vec<Title>,reqwest::Error> {
    
    let response = reqwest::blocking::get(format!(
        "{API_HOST}/v3/title/search?limit=9&order_by=id&search={name}"
    ))?;
    if response.status() != StatusCode::OK{
        return Err(response.error_for_status().err().unwrap())
    }
    return Ok(process_server_response_body(response));
}
fn process_server_response_body(response:reqwest::blocking::Response)->Vec<Title>{
    let mut jsonVal: Value = serde_json::from_str(response.text().unwrap().as_str()).expect("error while parsing response");
    return serde_json::from_value(jsonVal["list"].take()).expect("error while parsing response");
}
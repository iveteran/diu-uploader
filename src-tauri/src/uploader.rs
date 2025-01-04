// Refers:
//   https://medium.com/@itsuki.enjoy/post-file-using-multipart-form-data-in-rust-5171ae57aeed
//   https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/upload/src/lib.rs

use reqwest::{multipart, Client};
use serde::{ser::Serializer, Serialize};
use std::collections::HashMap;
use tauri::command;
use tokio::{
    fs::File,
    io::AsyncReadExt, // for read_to_end()
};

//#[derive(Debug, thiserror::Error)]
//pub enum Error {
//    #[error(transparent)]
//    Io(#[from] std::io::Error),
//    #[error(transparent)]
//    Request(#[from] reqwest::Error),
//    //#[error("{0}")]
//    //ContentLength(String),
//    #[error("request failed with status code {0}: {1}")]
//    HttpErrorCode(u16, String),
//}
//
//type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("request failed with status code {0}: {1}")]
pub struct HttpError(u16, String);

impl Serialize for HttpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[command]
pub async fn my_upload(
    url: &str,
    headers: HashMap<String, String>,
    form_params: HashMap<String, String>,     // key -> value
    form_kvs: HashMap<String, String>,        // key -> content
    form_files: HashMap<String, Vec<String>>, // key -> [file path]
    body: String,
) -> Result<String, HttpError> {
    let client = Client::new();
    let mut request = client.post(url);

    for (key, value) in headers {
        request = request.header(&key, value);
    }

    if !body.is_empty() {
        // content-type: text/plain
        request = request
            .header(reqwest::header::CONTENT_LENGTH, body.len())
            .header(reqwest::header::CONTENT_TYPE, "text/plain")
            .body(reqwest::Body::wrap(body));
    } else if !form_params.is_empty() {
        // content-type: application/x-www-form-urlencoded
        request = request.form(&form_params);
    } else {
        // content-type: multipart/form-data
        let mut form = multipart::Form::new();

        for (key, value) in form_kvs {
            form = form.text(key, value);
        }
        for (key, filepaths) in form_files {
            for filepath in filepaths.iter() {
                println!("> filepath: {}", filepath.clone());
                let mut file_contents = vec![];
                let mut file = File::open(filepath.clone()).await.unwrap();
                file.read_to_end(&mut file_contents).await.unwrap();
                let (_, filename) = filepath.rsplit_once('/').unwrap();
                let part = multipart::Part::bytes(file_contents).file_name(String::from(filename));
                form = form.part(key.clone(), part);
            }
        }
        request = request.multipart(form);
    }

    let response = request.send().await.unwrap();
    //println!("status: {}", response.status());

    if response.status().is_success() {
        //response.text().await.map_err(Into::into)
        Ok(response.text().await.unwrap_or_default())
    } else {
        //Err(Error::HttpErrorCode(
        //    response.status().as_u16(),
        //    response.text().await.unwrap_or_default(),
        //))
        Err(HttpError(
            response.status().as_u16(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn upload() {
        let uploadUrl = "http://localhost:8099/";
        let docDir = "/home/yuu/diu_upload_documents";

        let mut headers = HashMap::new();
        headers.insert("x-token".to_string(), "my_token".to_string());

        let params = HashMap::new();
        //let mut params = HashMap::new();
        //params.insert("lang".to_string(), "zh_CN".to_string());
        //params.insert("count".to_string(), "100".to_string());

        let mut files = HashMap::new();
        files
            .entry(String::from("files"))
            .or_insert(Vec::new())
            .push(String::format!("{}/hello.txt", docDir));
        files
            .entry(String::from("files"))
            .or_default()
            .push(String::from("{}/hello_world.txt", docDir));
        files
            .entry(String::from("files_2"))
            .or_default()
            .push(String::from("{}/databases/abc.txt", docDir));

        let mut form_kvs = HashMap::new();
        form_kvs.insert("my_key".to_string(), "my_value".to_string());
        form_kvs.insert("my_key_2".to_string(), "my_value_2".to_string());

        let body: String = String::new();
        //let body: String = String::from("foo bar");

        let res = uploader::my_upload(uploadUrl, headers, params, form_kvs, files, body)
            .await
            .unwrap();
        println!("response: {}", res);
    }
}

extern crate google_sheets4 as sheets4;
use sheets4::api::{Spreadsheet, ValueRange};
use sheets4::{hyper, hyper_rustls};
use sheets4::oauth2::{self, authenticator::Authenticator};
use chrono::prelude::*;
use crate::prelude::*;

pub struct GoogleSheetsApi {
    hub: sheets4::Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>, 
    spreadsheet_id: String,
}

impl GoogleSheetsApi {
    pub async fn new(spreadsheet_id: String) -> Self {
        let client = http_client();
        let auth = auth(client.clone()).await;
        let hub = sheets4::Sheets::new(client, auth);
        Self {
            hub,
            spreadsheet_id
        }
    }

    async fn get_spreadsheet(&self) -> Result<Spreadsheet, Error> {
        let result = self
            .hub
            .spreadsheets()
            .get(&self.spreadsheet_id)
            .doit()
            .await?
            .1;
        Ok(result)
    }

    pub async fn post_timelog(&self, task: String, in_office: String, hrs: String) -> Result<(), Error> {
        println!("{}", "Updating timelog".yellow());
        let today = Local::now();
        let day = today.day();
        let month = today.format("%b").to_string();
        let year = today.year();
        let sheet_name = format!("{}({})", month, year);
        if !self.contains_sheet(&sheet_name).await? {
           todo!() 
        }
        let range = format!("{}!B{}:D{}", sheet_name, day + 1, day + 1);
        let task = serde_json::Value::String(task);
        let in_office = serde_json::Value::String(in_office);
        let hrs = serde_json::Value::String(hrs);
        let data = vec![vec![in_office, task, hrs]];
        let req = ValueRange {
            major_dimension: Some("ROWS".to_string()),
            values: Some(data.clone()),
            range: Some(range.clone()),
        };

        let result = self
            .hub
            .spreadsheets()
            .values_update(req, &self.spreadsheet_id, &range)
            .value_input_option("USER_ENTERED")
            .doit()
            .await;
        match result {
            Err(err) => {
                println!("{}", "Failed to update timelog".red());
                println!("{}", err.to_string().red());
            }
            Ok(_) => {
                println!("{}", "Successfully updated Timelog".green());
            }
        }
        Ok(())
    }

    async fn contains_sheet(&self, sheet_name: &str) -> Result<bool, Error> {
        let spreadsheet = self.get_spreadsheet().await?;
        if let Some(sheets) = spreadsheet.sheets {
            let found_sheet = sheets.iter().find(|sheet| {
                sheet
                    .properties
                    .as_ref()
                    .map_or(false, |props| props.title == Some(sheet_name.to_string()))
            });
            if let Some(_sheet) = found_sheet {
                return Ok(true);
            } else {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }
}


fn http_client() -> hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    return hyper::Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build(),
    );
}

async fn auth(
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
) -> Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    let service_account = env::var("SERVICE_ACCOUNT_FILE").unwrap();
    let secret: oauth2::ServiceAccountKey = oauth2::read_service_account_key(service_account)
        .await
        .expect("secret not found");

    return oauth2::ServiceAccountAuthenticator::with_client(secret, client.clone())
        .build()
        .await
        .expect("could not create an authenticator");
}


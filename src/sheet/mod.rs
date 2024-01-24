extern crate google_sheets4 as sheets4;
use sheets4::api::Spreadsheet;
use sheets4::{hyper, hyper_rustls};
use sheets4::oauth2::{self, authenticator::Authenticator};
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

    pub async fn post_timelog(&self) -> Result<(), Error> {
        self.get_or_create_sheet("Jan(2024)".to_string()).await?;
        Ok(())
    }

    pub async fn get_or_create_sheet(&self, sheet_name: String) -> Result<(), Error> {
        let spreadsheet = self.get_spreadsheet().await?;
        if let Some(sheets) = spreadsheet.sheets {
            let found_sheet = sheets.iter().find(|sheet| {
                sheet
                    .properties
                    .as_ref()
                    .map_or(false, |props| props.title == Some(sheet_name.clone()))
            });
            if let Some(sheet) = found_sheet {
                println!("{:?}", sheet);
            } else {
                todo!()
            }
        } else {
            todo!()
        };
        Ok(())
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


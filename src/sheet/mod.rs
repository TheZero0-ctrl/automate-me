pub mod timelog;
use crate::prelude::*;

extern crate google_sheets4 as sheets4;
use sheets4::api::{
    Spreadsheet, ValueRange, BatchUpdateSpreadsheetRequest, SheetProperties,
    AddSheetRequest, Request, UpdateCellsRequest, RowData, BatchUpdateSpreadsheetResponse,
    DimensionProperties, UpdateDimensionPropertiesRequest, DimensionRange, UpdateValuesResponse
};
use sheets4::{hyper, hyper_rustls, FieldMask};
use sheets4::oauth2::{self, authenticator::Authenticator};
use chrono::{prelude::*, Duration};
use self::timelog::{GridRangeType, TimeLog};

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
            let response = self.create_sheet(&sheet_name).await;
            match response {
                Err(_) => {
                    println!("{}", "Failed to create sheet".red());
                }
                Ok(data) => {
                    println!("{}", "Created sheet".green());
                    let gid = data.replies.unwrap()[0].clone().add_sheet.unwrap().properties.unwrap().sheet_id;
                    let timelog = TimeLog::new(
                        gid,
                        task,
                        in_office,
                        hrs,
                        sheet_name.clone()
                    );
                    let response = self.update_newly_created_sheet(timelog).await;
                    match response {
                        Err(_) => {
                            println!("{}", "Failed to update new sheet".red());
                        }
                        Ok(_) => {
                            println!("{}", "Updated new sheet".green());
                        }
                    }
                }
            }
        } else {
            let timelog = TimeLog::new(
                None,
                task,
                in_office,
                hrs,
                sheet_name.clone()
            );
            let result = self.update_existing_sheet(timelog, day as i32).await;
            match result {
                Err(_) => {
                    println!("{}", "Failed to update timelog".red());
                }
                Ok(_) => {
                    println!("{}", "Successfully updated Timelog".green());
                }
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

    async fn update_newly_created_sheet(
        &self,
        timelog: TimeLog,
    ) -> Result<BatchUpdateSpreadsheetResponse, Error> {
        let (first_day_of_month, last_day_of_month) = first_and_last_day_of_month();
        let mut dates: Vec<RowData> = Vec::new();
        let mut current_day = first_day_of_month.clone();
        while current_day <= last_day_of_month {
            if current_day == Local::now().date_naive() {
                dates.push(timelog.get_current_row_data(current_day));
            } else if current_day.weekday() == Weekday::Sun || current_day.weekday() == Weekday::Sat {
                dates.push(timelog.get_weekend_row_data(current_day));
            } else {
                dates.push(timelog.get_normal_row_data(current_day));
            }
            current_day += Duration::days(1);
        }

        let fields: FieldMask = "user_entered_value, user_entered_format".parse().unwrap();
        let dimension_fields: FieldMask = "pixel_size".parse().unwrap();

        let update_cells_req = UpdateCellsRequest {
            range: Some(timelog.get_grid_range(GridRangeType::Header, 1)),
            rows: Some(vec![timelog.get_header_row_data()]),
            fields: Some(fields.clone()),
            ..Default::default()
        };

        let dimension_properties = DimensionProperties {
            pixel_size: Some(520),
            ..Default::default()
        };

        let req = BatchUpdateSpreadsheetRequest {
            requests: Some(vec![
                Request {
                    update_cells: Some(update_cells_req),
                    ..Default::default()
                },
                Request {
                    update_cells: Some(UpdateCellsRequest {
                        range: Some(
                            timelog.get_grid_range(
                                GridRangeType::Date,
                                last_day_of_month.day() as i32 + 1
                            )
                        ),
                        rows: Some(dates),
                        fields: Some(fields),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Request {
                    update_dimension_properties: Some(
                        UpdateDimensionPropertiesRequest {
                            properties: Some(dimension_properties),
                            fields: Some(dimension_fields),
                            range: Some(DimensionRange {
                                sheet_id: timelog.sheet_id,
                                dimension: Some("COLUMNS".to_string()),
                                start_index: Some(2),
                                end_index: Some(3),
                            }),
                            ..Default::default()
                        }
                    ),
                    ..Default::default()
                }
            ]),
            ..Default::default()
        };

        let result = self
            .hub
            .spreadsheets()
            .batch_update(req, &self.spreadsheet_id)
            .doit()
            .await?;
        Ok(result.1)
    }

    async fn update_existing_sheet(
        &self,
        timelog: TimeLog,
        day: i32,
    ) -> Result<UpdateValuesResponse, Error> {
        let range = format!("{}!B{}:D{}", timelog.sheet_name, day + 1, day + 1);
        let task = serde_json::Value::String(timelog.task.to_string());
        let in_office = serde_json::Value::String(timelog.in_office.to_string());
        let hrs = serde_json::Value::String(timelog.hrs.to_string());
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
            .await?;
        Ok(result.1)
    }

    async fn create_sheet(&self, sheet_name: &str) -> Result<BatchUpdateSpreadsheetResponse, Error> {
        println!("{}", "Creating sheet".yellow());
        let sheet_properties = SheetProperties {
            title: Some(sheet_name.to_string()),
            index: Some(0),
            ..Default::default()
        };

        let add_sheet_request = AddSheetRequest {
            properties: Some(sheet_properties),
        };

        let req = BatchUpdateSpreadsheetRequest {
            requests: Some(vec![
                Request {
                    add_sheet: Some(add_sheet_request),
                    ..Default::default()
                }
            ]),
            ..Default::default()
        };


        let result = self
            .hub
            .spreadsheets()
            .batch_update(req, &self.spreadsheet_id)
            .doit()
            .await?
            .1;
        Ok(result)
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

fn first_and_last_day_of_month() ->(NaiveDate, NaiveDate) {
    let today = Local::now();
    let first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let last = NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
        .unwrap_or(NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap())
        .pred_opt()
        .unwrap();
        
    (first, last)
}

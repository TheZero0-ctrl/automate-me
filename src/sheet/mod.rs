extern crate google_sheets4 as sheets4;
use sheets4::api::{
    Spreadsheet, ValueRange, BatchUpdateSpreadsheetRequest, SheetProperties,
    AddSheetRequest, Request, GridRange, UpdateCellsRequest, RowData, CellData,
    ExtendedValue, BatchUpdateSpreadsheetResponse, Color, CellFormat,
    DimensionProperties, UpdateDimensionPropertiesRequest, DimensionRange
};
use sheets4::{hyper, hyper_rustls, FieldMask};
use sheets4::oauth2::{self, authenticator::Authenticator};
use chrono::{prelude::*, Duration};
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
            let response = self.create_sheet(&sheet_name).await;
            match response {
                Err(_) => {
                    println!("{}", "Failed to create sheet".red());
                }
                Ok(data) => {
                    println!("{}", "Created sheet".green());
                    let gid = data.replies.unwrap()[0].clone().add_sheet.unwrap().properties.unwrap().sheet_id.unwrap();
                    let response = self.update_newly_created_sheet(gid, &task, &in_office, &hrs).await;
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
            return Ok(())
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

    async fn update_newly_created_sheet(
        &self,
        sheet_id: i32,
        task: &str,
        in_office: &str,
        hrs: &str,
    ) -> Result<BatchUpdateSpreadsheetResponse, Error> {
        let grid_range_for_headers = GridRange {
            sheet_id: Some(sheet_id),
            start_row_index: Some(0),
            end_row_index: Some(1),
            start_column_index: Some(0),
            end_column_index: Some(4),
        };

        let (first_day_of_month, last_day_of_month) = first_and_last_day_of_month();
        let grid_range_for_date = GridRange {
            sheet_id: Some(sheet_id),
            start_row_index: Some(1),
            end_row_index: Some(last_day_of_month.day() as i32 + 1),
            start_column_index: Some(0),
            end_column_index: Some(26),
        };
        let mut dates: Vec<RowData> = Vec::new();
        let mut current_day = first_day_of_month.clone();
        while current_day <= last_day_of_month {
            if current_day == Local::now().date_naive() {
                let row_data = RowData {
                    values: Some(vec![
                        CellData {
                            user_entered_value: Some(
                                ExtendedValue {
                                    string_value: Some(current_day.format("%m/%d/%Y").to_string()),
                                    ..Default::default()
                                }
                            ),
                            ..Default::default()
                        },
                        CellData {
                            user_entered_value: Some(
                                ExtendedValue {
                                    string_value: Some(in_office.to_string()),
                                    ..Default::default()
                                }
                            ),
                            ..Default::default()
                        },
                        CellData {
                            user_entered_value: Some(
                                ExtendedValue {
                                    string_value: Some(task.to_string()),
                                    ..Default::default()
                                }
                            ),
                            ..Default::default()
                        },
                        CellData {
                            user_entered_value: Some(
                                ExtendedValue {
                                    string_value: Some(hrs.to_string()),
                                    ..Default::default()
                                }
                            ),
                            ..Default::default()
                        }
                    ])
                };  
                dates.push(row_data);
            } else if current_day.weekday() == Weekday::Sun || current_day.weekday() == Weekday::Sat {
                let colored_cell = CellData {
                    user_entered_format: Some(
                        CellFormat {
                            background_color: Some(
                                Color {
                                    red: Some(1.0),
                                    green: Some(0.65),
                                    blue: Some(0.0),
                                    ..Default::default()
                                }
                            ),
                            ..Default::default()
                        }
                    ),
                    ..Default::default()
                };

                let colored_cells = vec![colored_cell; 25];
                let mut values = Vec::new();
                values.push(
                    CellData {
                        user_entered_value: Some(
                            ExtendedValue {
                                string_value: Some(current_day.format("%m/%d/%Y").to_string()),
                                ..Default::default()
                            }
                        ),
                        user_entered_format: Some(
                            CellFormat {
                                background_color: Some(
                                    Color {
                                        red: Some(1.0),
                                        green: Some(0.65),
                                        blue: Some(0.0),
                                        ..Default::default()
                                    }
                                ),
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    }
                );
                values.extend(colored_cells);
                
                let row_data = RowData {
                    values: Some(values),
                }; 
                dates.push(row_data);
            } else {
                let row_data = RowData {
                    values: Some(vec![
                        CellData {
                        user_entered_value: Some(
                        ExtendedValue {
                        string_value: Some(current_day.format("%m/%d/%Y").to_string()),
                        ..Default::default()
                        }
                        ),
                        ..Default::default()
                        }
                    ])
                };
                dates.push(row_data);
            }
            current_day += Duration::days(1);
        }
        let mut cells_data_for_headers = Vec::new();
        for value in vec!["Date", "In Office", "Task", "hrs"] {
            cells_data_for_headers.push(CellData {
                user_entered_value: Some(
                    ExtendedValue {
                        string_value: Some(value.to_string()),
                        ..Default::default()
                    } 
                ),
                ..Default::default()
            })            
        }
        let row_data_for_headers = RowData {
            values: Some(cells_data_for_headers),
        };

        let fields: FieldMask = "user_entered_value, user_entered_format".parse().unwrap();
        let dimension_fields: FieldMask = "pixel_size".parse().unwrap();

        let update_cells_req = UpdateCellsRequest {
            range: Some(grid_range_for_headers),
            rows: Some(vec![row_data_for_headers]),
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
                        range: Some(grid_range_for_date),
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
                                sheet_id: Some(sheet_id),
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

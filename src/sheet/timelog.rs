use chrono::NaiveDate;
use google_sheets4::api::{GridRange, RowData, CellData, ExtendedValue, CellFormat, Color};

pub struct TimeLog {
    pub sheet_id: Option<i32>,
    pub task: String,
    pub in_office: String,
    pub hrs: String,
    pub sheet_name: String
}

impl TimeLog {
    pub fn new(
        sheet_id: Option<i32>,
        task: String,
        in_office: String,
        hrs: String,
        sheet_name: String
    ) -> Self {
        Self {
            sheet_id,
            task,
            in_office,
            hrs,
            sheet_name
        }
    }
    pub fn get_grid_range(
        &self,
        grid_range_type: GridRangeType,
        end_row_index: i32,
    ) -> GridRange {
        match grid_range_type {
            GridRangeType::Header => {
                GridRange {
                    sheet_id: self.sheet_id,
                    start_row_index: Some(0),
                    end_row_index: Some(end_row_index),
                    start_column_index: Some(0),
                    end_column_index: Some(4),
                }
            }
            GridRangeType::Date => {
                GridRange {
                    sheet_id: self.sheet_id,
                    start_row_index: Some(1),
                    end_row_index: Some(end_row_index),
                    start_column_index: Some(0),
                    end_column_index: Some(26),

                }
            }
        }
    }

    pub fn get_current_row_data(&self, current_day: NaiveDate) -> RowData {
        RowData {
            values: Some(vec![
                self.get_cell_data(
                    CellDataType::ValueOnly,
                    Some(current_day.format("%m/%d/%Y").to_string())
                ),
                self.get_cell_data(
                    CellDataType::ValueOnly,
                    Some(self.in_office.to_string())
                ),
                self.get_cell_data(
                    CellDataType::ValueOnly,
                    Some(self.task.to_string())
                ),
                self.get_cell_data(
                    CellDataType::ValueOnly,
                    Some(self.hrs.to_string())
                ),
            ])
        }
    }

    pub fn get_weekend_row_data(&self, current_day: NaiveDate) -> RowData {
        let colored_cells = vec![self.get_cell_data(CellDataType::StyleOnly, None); 25];
        let mut values = Vec::new();
        values.push(
            self.get_cell_data(
                CellDataType::StyleAndValue,
                Some(current_day.format("%m/%d/%Y").to_string()),
            )
        );
        values.extend(colored_cells);

        RowData { values: Some(values) }
    }

    pub fn get_normal_row_data(&self, current_day: NaiveDate) -> RowData {
        RowData {
            values: Some(vec![
                self.get_cell_data(
                    CellDataType::ValueOnly,
                    Some(current_day.format("%m/%d/%Y").to_string())
                )
            ])
        }
    }

    pub fn get_header_row_data(&self) -> RowData {
        let mut cells_data_for_headers = Vec::new();
        for value in vec!["Date", "In Office", "Task", "hrs"] {
            cells_data_for_headers.push(
                self.get_cell_data(CellDataType::ValueOnly, Some(
                    value.to_string(),
                ))
            )            
        }
        RowData {
           values: Some(cells_data_for_headers),
        }
    }

    pub fn get_cell_data(
        &self,
        cell_data_type: CellDataType,
        value: Option<String>,
    ) -> CellData {
        match cell_data_type {
            CellDataType::StyleOnly => {
               CellData {
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
               }
            }
            CellDataType::StyleAndValue => {
                CellData {
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
                    user_entered_value: Some(
                        ExtendedValue {
                            string_value: value,
                            ..Default::default()
                        }
                    ),
                    ..Default::default()
                }
            }
            CellDataType::ValueOnly => {
                CellData {
                    user_entered_value: Some(
                        ExtendedValue {
                            string_value: value,
                            ..Default::default()
                        }
                    ),
                    ..Default::default()
                }
            }
        }
    }
}

// Different variants of grid range for different request
pub enum GridRangeType {
    Header,
    Date
}

pub enum CellDataType {
    StyleOnly,
    StyleAndValue,
    ValueOnly
}



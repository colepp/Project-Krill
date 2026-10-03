    impl<'a> Table<'a> {

        // TABLE CREATION CONSTRUCTORS AND HELPERS
        /// from_csv Creates a Table struct from a given file path
        /// On success a Table with all values will be returned
        pub fn from_csv(path: &'a Path) -> Result<Self, Box<dyn Error>> {

            

            let Some(table_name) = path.file_stem().and_then(|f| f.to_str()) else {
                return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "File does not exist at")))
            };


            // let x = path.extension().and_then(|extentsion| extentsion.to_str()).is_some_and(|extension| extension.contains("csv"));
            // println!("{}",x);

            if path.extension().and_then(|extentsion| extentsion.to_str()).is_some_and(|extension| extension.contains(".csv")) {
                return Err(Box::new(io::Error::new(io::ErrorKind::InvalidFilename, "Expected .csv")))
            }

            let csv_file = File::open(path)?;

            

            let table_details = TableDetails::from_csv(String::from(table_name), Some(path),csv_file.metadata()?.len());

            let reader = BufReader::new(csv_file);

            // covert the rows into an lines iterator then 
            let mut raw_rows = reader.lines().map(move |line| line.unwrap());

            let columns_name_row = raw_rows
            .next()
            .ok_or_else(|| KrillErrors::EmptyCSV)?;

            let column_names = Self::parse_row(columns_name_row)
            .ok_or_else(|| KrillErrors::ParsingError)?;

            let mut column_map = Self::map_columns(&column_names);

            let mut rows = vec![];
            for raw_row in raw_rows {

                let mut typed_rows = vec![];
                let untyped_row = Self::parse_row(raw_row).ok_or_else(|| KrillErrors::ParsingError)?;

                for(val_index,untyped_val) in untyped_row.iter().enumerate() {

                    let column_name = column_names.get(val_index).ok_or_else(|| KrillErrors::ColumnOutOfBounds(val_index, column_names.len()))?;
                    let column  = column_map.get_mut(column_name).ok_or_else(|| KrillErrors::ColumnNotFound(column_name.clone()))?;

                    let typed_value = Self::infer_type(untyped_val);
                    let value_type = typed_value.value_type();

                    if column.value_type == ValueType::Unset && value_type != ValueType::EMPTY {
                        column.value_type = value_type;
                        typed_rows.push(typed_value);
                    } else if column.value_type != value_type && value_type != ValueType::EMPTY {
                        return Err(Box::new(KrillErrors::ColumnTypeMismatched(column.value_type.clone(), value_type)));
                    } else {
                        typed_rows.push(typed_value);
                    }

                }
                rows.push(Row { values: typed_rows});   
                
            }


            Ok(Table { table_details,column_map, column_names, rows })
        }

        /// to_csv Converts a Table struct into a .csv
        /// the path parameter should be given if the Table does not have a save path in its detials
        /// or the Table struct has been constructed from new() rather than from_csv()
        pub fn to_csv(&mut self, path: &Path,overwrite: bool) -> Result<(),Box<dyn Error>> {

            let mut res = String::new();
            res = self.column_names.join(",");
            
            res = res.add("\n");

            for row in self.rows().iter() {
                for column in 0..self.column_names.len() {
                    if self.column_details(self.column_names[column].as_str())?.value_type == ValueType::Unset && column == self.column_names.len() - 1 {
                        continue;
                    }
                    res = res.add(format!("{},", row.get_value_from_row(column)?).as_str());
                }
                res = res.add("\n");
            }

            if let Some(parent_dir) = path.parent() {
                if !parent_dir.as_os_str().is_empty() {
                    fs::create_dir_all(parent_dir)?;
                }
            }

            let mut options = OpenOptions::new();
            options.write(true);
            if overwrite {
                options.create(true).truncate(true);
            } else {
                options.create_new(true);
            }
            let mut file = options.open(path)?;
            file.write_all(res.as_bytes())?;
            
            // decide when you work on this if this should replace a already existing table
            // throw an error or append a copy var i.e. file_copy(1).csv
            Ok(())
        }

        /// creates empty Table Struct with all empty rows cols and details
        pub fn new(self, table_name: &str) -> Self {
            let table_details = TableDetails::new(String::from(table_name));
            Table {
                table_details,
                column_map: HashMap::new(),
                column_names: vec![],
                rows: vec![],
            }

        }

        fn parse_row(row: String) -> Option<Vec<String>> {
            let mut res: Vec<String> = vec![];
            let mut is_quote_string = false;
            let mut value = String::new();

            if row.len() == 0 {
                return None;
            }

            for character in row.chars() {
                match character {
                    '"' => {is_quote_string = !is_quote_string;}
                    ',' => {
                        if !is_quote_string{
                            res.push(value);
                            value = String::new();
                        }
                    }
                    _ => {
                        value.push(character);
                    }
                }
            }
            res.push(value);
            Some(res)
        }

        fn map_columns(column_names: &[String]) -> HashMap<String,ColumnDetails> {
            let mut mapping: HashMap<String,ColumnDetails> = HashMap::new();
            

            for (i, column_name) in column_names.iter().enumerate() {
                mapping.insert(column_name.clone(),ColumnDetails {column_number: i, value_type: ValueType::Unset} );
            }
            mapping
        }

        fn infer_type(raw_value: &str) -> Value {

            if raw_value.is_empty() {
                return Value::Empty;
            }

            if let Ok(parsed_val) = raw_value.parse::<i64>() {
                return Value::from(parsed_val)
            } 

            if let Ok(parsed_val) = raw_value.parse::<f64>() {
                return Value::from(parsed_val);
            }
            
            // set this up later for numeric bools like 0 for true 1 for false 
            if let Ok(parsed_val) = raw_value.parse::<bool>() {
                return Value::from(parsed_val);
            }

            Value::from(raw_value.to_string())
        
        }

    }

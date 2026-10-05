    #[derive(Clone,Debug)]
    pub struct Table<'a> {
        table_details: TableDetails<'a>,
        pub column_map: HashMap<String,ColumnDetails>,
        column_names: Vec<String>,
        rows: Vec<Row>,
    }

    pub enum FilterType<T> {
        NonEmpty,
        Empty,
        Conditional(T),
    }

    pub enum FindRowResultTypes<'a> {
        FindRowResult(&'a Row,usize),
        FindRowResultMutable(&'a mut Row,usize),
    }

    impl fmt::Display for Table<'_> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            if self.column_names.is_empty() {
                return write!(formatter, "Empty table");
            }

        // Start each width at the corresponding column-name width.
        let mut column_widths: Vec<usize> = self
            .column_names
            .iter()
            .map(|name| name.chars().count())
            .collect();

        // Expand each width to fit the largest value in that column.
        for row in &self.rows {
            for (column_index, value) in row.values.iter().enumerate() {
                if let Some(width) = column_widths.get_mut(column_index) {
                    *width = (*width).max(value.to_string().chars().count());
                }
            }
        }

        // write_separator(formatter, &column_widths)?;

        // Header row
        write!(formatter, "|")?;

        for (name, width) in self.column_names.iter().zip(&column_widths) {
            write!(formatter, " {:<width$} |", name, width = width)?;
        }

        writeln!(formatter)?;
        write!(formatter, "+")?;

        for width in &column_widths {
            write!(formatter, "{}+", "-".repeat(*width + 2))?;
        }

        writeln!(formatter);

        // Data rows
        for row in &self.rows {
            write!(formatter, "|")?;

            for (column_index, width) in column_widths.iter().enumerate() {
                let value = row
                    .values
                    .get(column_index)
                    .map(Value::to_string)
                    .unwrap_or_default();

                write!(formatter, " {:<width$} |", value, width = width)?;
            }

            writeln!(formatter)?;
        }

        for width in column_widths {
            write!(formatter, "{}+", "-".repeat(width + 2))?;
        }

        write!(formatter,"")
        }

        
    }


    impl<'a> Table<'a> {
        // write data operations
        /// `fillna_colum` replaces all values of value type `ValueType::Empty` with the 
        /// the method takes three values `column_name` the name of the target column `replacement_function` a closure that must always return type 
        /// `Result<Value,Box<dyn Error>>`, the closure must also ingest in order a reference to the table being worked on and the target row as a `usize`
        /// 
    
        pub fn fillna_column<T>(&mut self,column_name: &str,replacement_function: T, inplace: bool) -> Result<Option<Self>,Box<dyn Error>>
        where T: Fn(&Table,usize) -> Result<Value,Box<dyn Error>> {

            let column_details = self.column_details(column_name)?;
            let column_number = column_details.column_number;


            if inplace {
                for row_number in 0..self.rows.len() {
                    let value = self.rows.get(row_number)
                    .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number, self.rows.len())))?
                    .get_value_from_row(column_number)?;
                    println!("{}",value);
                    if value.is_empty() {
                        let new_value = replacement_function(self,row_number)?;
                        self.replace_value_on_row(row_number, column_number, new_value, true)?;
                        }
                }
                return Ok(None);
            }

            let mut copy_table = self.clone();
            for row_number in 0..copy_table.rows.len() {

            
                

                if copy_table.rows.get(row_number)
                .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number, copy_table.rows.len())))?
                .get_value_from_row(column_number)?
                .value_type() == ValueType::EMPTY {
                    let new_value = replacement_function(self,row_number)?;
                    copy_table.replace_value_on_row(row_number, column_number, new_value, true)?;
                }
            }   
            Ok(Some(copy_table))
        }

        
       
        // row data access and modification


        /// Replaces a value at the given row and column indices.
        /// Returns None when inplace is true; otherwise returns a modified copy.
        pub fn replace_value_on_row(&mut self, row_number: usize, column_number: usize, value: Value, inplace: bool) -> Result<Option<Self>, Box<dyn Error>> {
            self.with_inplace(inplace, |table| {
                let row = table.get_row_mut(row_number)?;
                let column_count = row.values.len();
                let target = row.values.get_mut(column_number)
                    .ok_or_else(|| KrillErrors::ColumnOutOfBounds(column_number, column_count))?;
                *target = value;
                Ok(())
            })
        }

        /// Returns a non mutable reference to the rows table
        pub fn rows(&self) -> &[Row] {
            &self.rows
        }

        /// Returns a mutable reference to the rows table
        pub fn rows_mut(&mut self) -> &mut [Row] {
            &mut self.rows
        }

        /// Returns a non mutable reference to a row
        pub fn get_row(&self,row_number: usize) -> Result<&Row,Box<dyn Error>> {
            Ok(self.rows
            .get(row_number)
            .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number,self.rows.len())))?)
        }

        /// Returns a mutable reference to a row
        pub fn get_row_mut(&mut self, row_number: usize) -> Result<&mut Row,KrillErrors> {
            let len: usize = self.rows.len();
            self.rows.get_mut(row_number).ok_or_else(|| KrillErrors::RowOutOfBounds(row_number,len ))
        }

        /// Appends a row in column order, padding omitted trailing values with Value::Empty.
        /// Empty values are allowed in every column. Nonempty values establish unset
        /// column types and must otherwise match the existing type.
        /// Invalid rows return an error without changing rows or column types.
        /// Returns None when inplace is true; otherwise returns a modified copy.
        pub fn add_row(&mut self, mut row_values: Vec<Value>, inplace: bool) -> Result<Option<Self>, Box<dyn Error>> {
            self.with_inplace(inplace, |table| {
                if row_values.len() > table.column_names.len() {
                    return Err(Box::new(KrillErrors::TooManyArguments(row_values.len(),table.column_names.len())))
                }

                // Validate the entire row before changing any inferred column types.
                for (value, column_name) in row_values.iter().zip(&table.column_names) {
                    let column_details = table.column_details(column_name)?;
                    if !value.is_empty()
                        && column_details.value_type != ValueType::Unset
                        && column_details.value_type != value.value_type()
                    {
                        return Err(Box::new(KrillErrors::ColumnTypeMismatched(
                                column_details.value_type, value.value_type(),
                        )));
                    }
                }

                for (value, column_name) in row_values.iter().zip(&table.column_names) {
                    let column_details = table.column_map.get_mut(column_name).unwrap();
                    if column_details.value_type == ValueType::Unset && !value.is_empty() {
                        column_details.value_type = value.value_type();
                    }
                }

                while row_values.len() < table.column_names.len() {
                    row_values.push(Value::Empty)
                }

                let new_row = Row {
                    values: row_values,
                };
                table.rows.push(new_row);



                Ok(())
            })
        }

        /// Removes a row while preserving the order of the remaining rows.
        /// Returns None when inplace is true; otherwise returns a modified copy.
        pub fn remove_row(&mut self, row_number: usize, inplace: bool) -> Result<Option<Self>, Box<dyn Error>> {
            self.with_inplace(inplace, |table| {
                if row_number >= table.rows.len() {
                    return Err(Box::new(KrillErrors::RowOutOfBounds(row_number, table.rows.len())));
                }
                table.rows.remove(row_number);
                Ok(())
            })
        }

        // Match fillna: mutate self and return None, or mutate a clone and return it.
        fn with_inplace<F>(&mut self, inplace: bool, operation: F) -> Result<Option<Self>, Box<dyn Error>>
        where F: FnOnce(&mut Self) -> Result<(), Box<dyn Error>> {
            if inplace {
                operation(self)?;
                Ok(None)
            } else {
                let mut copy = self.clone();
                operation(&mut copy)?;
                Ok(Some(copy))
            }
        }

        /// Adds an empty column unless the name already exists.
        /// Returns None when inplace is true; otherwise returns a modified copy.
        pub fn add_column_empty(&mut self, column_name: &str, inplace: bool) -> Result<Option<Self>, Box<dyn Error>> {
            self.add_column_default(Value::Empty, column_name, inplace)
        }

        /// Adds a column filled with default_value unless the name already exists.
        /// Returns None when inplace is true; otherwise returns a modified copy.
        pub fn add_column_default(&mut self, default_value: Value, column_name: &str, inplace: bool) -> Result<Option<Self>, Box<dyn Error>> {
            self.with_inplace(inplace, |table| {
                if !table.column_names.iter().any(|name| name == column_name) {
                    let value_type = if default_value.is_empty() {
                        ValueType::Unset
                    } else {
                        default_value.value_type()
                    };
                    table.column_map.insert(column_name.to_string(), ColumnDetails {
                        column_number: table.column_names.len(), value_type,
                    });
                    table.column_names.push(column_name.to_string());
                    for row in &mut table.rows {
                        row.values.push(default_value.clone());
                    }
                }
                Ok(())
            })
        }

        /// Removes a column and its values, preserving the order of remaining columns.
        /// Returns None when inplace is true; otherwise returns a modified copy.
        pub fn rem_col(&mut self, column_name: &str, inplace: bool) -> Result<Option<Self>, Box<dyn Error>> {
            self.with_inplace(inplace, |table| {
                let column_number = table.column_index(column_name)?;
                // Reject incomplete rows before changing the table.
                for row in &table.rows {
                    row.get_value_from_row(column_number)?;
                }
                for row in &mut table.rows {
                    row.remove_value_from_row(column_number)?;
                }
                table.column_map.remove(column_name);
                table.column_names.retain(|name| name != column_name);
                for (column_number, column_name) in table.column_names.iter().enumerate() {
                    let details = table.column_map.get_mut(column_name)
                        .ok_or_else(|| KrillErrors::ColumnNotFound(column_name.clone()))?;
                    details.column_number = column_number;
                }
                Ok(())
            })
        }

        fn row_empty(row: &Row) -> bool{
            for value in row.values.iter() {
                if !value.is_empty() {
                    return false
                }
            }
            true
        }


        pub fn filter_row<T>(&mut self,filter_type: FilterType<T>,inplace: bool ) 
        -> Result<Option<Self>,Box<dyn Error>>
        where T: Fn(&Row) -> bool {
            self.with_inplace(inplace,|table| {
                let mut row_number = 0;
                let mut table_len = table.rows.len();

                 while row_number < table_len {
                    let row = table.get_row(row_number)?;
                    if match filter_type {
                        FilterType::Empty => Self::row_empty(row),
                        FilterType::NonEmpty => !Self::row_empty(row),
                        FilterType::Conditional(ref condition) => condition(row)
                    } {
                        table.remove_row(row_number,true)?;
                        table_len = table.rows.len();
                        row_number = if row_number == 0 {
                            0
                        } else {
                            row_number - 1
                        }
                    } else {
                        row_number += 1;
                    }
                }

                Ok(())
            })
        }


        pub fn find_rows<T>(&'_ self,filter_type: FilterType<T>) -> Result<Vec<FindRowResultTypes<'_>>,Box<dyn Error>>
        where T: Fn(&Row) -> bool {
            let mut res = vec![];
            for (row_number,row) in self.rows().iter().enumerate() {
                if match filter_type {
                        FilterType::Empty => Self::row_empty(row),
                        FilterType::NonEmpty => !Self::row_empty(row),
                        FilterType::Conditional(ref condition) => condition(row)
                    } {
                        res.push(
                            FindRowResultTypes::FindRowResult(row,row_number)
                        );
                    }
            }
            Ok(res)
        }

        pub fn find_rows_mut<T>(&'_ mut self,filter_type: FilterType<T>) -> Result<Vec<FindRowResultTypes<'_>>,Box<dyn Error>>
        where T: Fn(&Row) -> bool {
            let mut res = vec![];
            for (row_number,row) in self.rows_mut().iter_mut().enumerate() {
                if match filter_type {
                        FilterType::Empty => Self::row_empty(row),
                        FilterType::NonEmpty => !Self::row_empty(row),
                        FilterType::Conditional(ref condition) => condition(row)
                    } {
                        res.push(
                            FindRowResultTypes::FindRowResultMutable(row,row_number)
                        );
                    }
            }
            Ok(res)
        }

        // column data access and modification
        /// returns an non mutable Column iterator over a given column name
        pub fn column_iter(&'_ self, column_name: &str) -> Result<Column<'_>,Box<dyn Error>> {
            
            let column_details = self.column_details(column_name)?;
            Ok(Column {
                rows: &self.rows,
                 column_number: column_details.column_number,
                  row_len: self.rows.len(),
                   current_row: 0 }
                )
        }

        
        /// returns a deep copy of a column given the column name
        pub fn column_deep_copy(&self,column_name: &str) -> Result<Vec<Value>, Box<dyn Error>> {
            Ok(self.column_iter(column_name)?.map(|value| value.clone()).collect())
        }

        pub fn column(&self,column_name: &str) -> Result<Vec<&Value>, Box<dyn Error>> {
            let column: Vec<&Value> = self
            .column_iter(column_name)?.collect();
            Ok(column)
        }

        /// will return the column details of a given column name
        pub fn column_details(&self,column_name: &str) -> Result<&ColumnDetails, Box<dyn Error>>{
            let column_details = self.column_map.get(column_name).ok_or_else(|| Box::new(KrillErrors::ColumnNotFound(column_name.to_string())))?;
            Ok(column_details)

        }




        /// returns the type of a given column
        pub fn column_type(&self,column_name: &str) -> Result<ValueType,KrillErrors> {
            let column = self.column_map.get(column_name).ok_or_else(|| KrillErrors::ColumnNotFound(column_name.to_string()))?;
            Ok(column.value_type.clone())
        }

        pub fn column_index(&self,column_name: &str) -> Result<usize,KrillErrors> {
            let column = self.column_map.get(column_name).ok_or_else(|| KrillErrors::ColumnNotFound(column_name.to_string()))?;
            Ok(column.column_number)
        }

    

        // Descriptors and presentations
        /// returns dimensions of the table
        pub fn dimensions(&self) -> (usize,usize) {
            (self.column_names.len(),self.rows.len())
        }

        /// returns a string result of dimesions
        pub fn describe(&self) {
            let (x,y)= self.dimensions();
            let path_string: &str = self.table_details.
            file_path.map_or_else(|| "Table has no Save Destination.",|path| path.to_str().unwrap());
            

            println!("TABLE DETAILS");
            println!("TABLE NAME: {}",self.table_details.table_name);
            println!("PATH: {}",path_string);
            println!("DIMENSIONS: ({},{})",x,y);
            

            const WRITE_SEPARATOR: &str = "------------------------";
            for (column_name,column) in self.column_map.iter() {
                println!("NAME: {}, DATA-TYPE {}",column_name.to_uppercase(),column.value_type);
                println!("{}",WRITE_SEPARATOR);
                // println!("NUMBER OF NON-NULL VALUES: {}",self.count(column_name, CountType::NonEmpty));
                println!("MAX: ");
                println!("MIN: ");
                // then do avg mode etc
                println!("{}\n\n",WRITE_SEPARATOR);
            }
        }

        

        



    }


    impl<'a> Index<usize> for Table<'a> {
        type Output = Row;
    
        fn index(&self, index: usize) -> &Self::Output {
            self.get_row(index).unwrap_or_else(|err| {
            panic!("Failed to index Table: {err:?}")})
        }
    }

    impl<'a> IndexMut<usize> for Table<'a> {
        
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            self.get_row_mut(index).unwrap_or_else(|err| {
            panic!("Failed to index Table: {err:?}")})
        }
            
    }

    impl<'a> Index<&str> for Table<'a> {
        type Output = ColumnDetails;
        
        fn index(&self, index: &str) -> &Self::Output {
            &self.column_details(index).unwrap_or_else(|err| panic!("{}",err))
        }
    }

    #[derive(Clone,Debug)]
    pub struct Table<'a> {
        table_details: TableDetails<'a>,
        pub column_map: HashMap<String,ColumnDetails>,
        column_names: Vec<String>,
        rows: Vec<Row>,
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
    
        pub fn fillna_column<T>(&mut self,column_name: &str,replacement_function: T, inplace: bool) -> Result<Option<Table<'_>>,Box<dyn Error>>
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
                        Self::replace_value_on_row(&mut self.rows,row_number, column_number, new_value)?;
                        }
                }
                return Ok(None);
            }

            let mut copy_table = self.clone();
            for row_number in 0..copy_table.column_names.len() {

            
                

                if copy_table.rows.get(row_number)
                .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number, copy_table.rows.len())))?
                .get_value_from_row(column_number)?
                .value_type() == ValueType::EMPTY {
                    let new_value = replacement_function(self,row_number)?;
                    Self::replace_value_on_row(&mut copy_table.rows,row_number, column_number, new_value)?;
                }
            }   
            Ok(Some(copy_table))
        }

        
       
        // row data access and modification


        /// Replaces a value in a given column on a target row, takes a mutable reference to the rows table,
        ///  the row number, the column number, and the replacement value 
        pub fn replace_value_on_row(rows: &mut [Row],row_number: usize,column_number: usize, value: Value) -> Result<(),Box<dyn Error>> {
            if let Some(row) = rows.get_mut(row_number) {
                row.values[column_number] = value;
                return Ok(());
            }
            Err(Box::new(KrillErrors::RowOutOfBounds(row_number, rows.len())))
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


        /// adds column of given name assuming it does not already exist
        pub fn add_column_empty(&mut self, column_name: &str,inplace:bool) -> Result<(),Box<dyn Error>> {
            if !self.column_names.contains(&column_name.to_string()) {
                self.column_names.push(column_name.to_string());
                self.column_map.insert(column_name.to_string(), ColumnDetails { column_number: self.column_names.len() - 1, value_type: ValueType::Unset });

                for row in self.rows_mut() {
                    row.values.push(Value::Empty);
                }
            }

            Ok(())
        }

        pub fn add_column_default(&mut self,default_value: Value, column_name: &str,inplace:bool) -> Result<(),Box<dyn Error>> {
            if !self.column_names.contains(&column_name.to_string()) {
                self.column_names.push(column_name.to_string());
                self.column_map.insert(column_name.to_string(), ColumnDetails { column_number: self.column_names.len() - 1, value_type: default_value.value_type() });

                for row in self.rows_mut() {
                    row.values.push(default_value.clone());
                }
            }

            Ok(())
        }

        /// removes column of given name if it exists
        pub fn rem_col(&mut self, column_name: &str,inplace:bool) -> Result<(),Box<dyn Error>>  {
            for row in self.rows_mut().iter_mut() {
                row.remove_value_from_row(self.column_index(column_name)?)?;
            }

            Ok(())
        }

        // column data access and modification
        /// returns an non mutable Column iterator over a given column name
        pub fn column_iter(&self, column_name: &str) -> Result<Column,Box<dyn Error>> {
            
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

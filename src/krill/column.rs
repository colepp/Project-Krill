    
     /// Represents a Read-Only Column for iterating through
    #[derive(Debug,Clone)]
    pub struct Column<'a>{
        rows: &'a Vec<Row>,
        column_number: usize, 
        row_len: usize,
        current_row: usize,
    }

    impl<'a> Iterator for Column<'a> {
        type Item = &'a Value;
        fn next(&mut self) -> Option<Self::Item> {

            if self.current_row >= self.row_len {
                return None
            }
            let row = self.get_row(self.current_row);
            self.current_row += 1;
            let Some(row) = row else {
                panic!("Row should exist but none was found");
            };

            row.get_value_from_row(self.column_number).ok()


        }
    }

    impl<'a> Column<'a> {
        pub fn get_row(&self, row_number: usize) -> Option<&'a Row> {
            self.rows.get(row_number)
        }
    }


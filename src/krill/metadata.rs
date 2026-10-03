    #[derive(Clone,Debug)]
    pub struct TableDetails<'a> {
        pub file_path: Option<&'a Path>,
        pub table_name: String,
        pub file_size: u64,
    }


    
    #[derive(Debug, Clone, PartialEq)]
    pub struct ColumnDetails {
        pub column_number: usize,
        pub value_type: ValueType,
    }

    impl<'a> TableDetails<'a> {

        /// creates table details for a Table struct where the only input is the tables name and the file_path is None
        pub fn new(table_name: String) -> Self {
            TableDetails {
                file_path: None,
                table_name,
                file_size: 0
            }
        }

        pub fn from_csv(table_name: String, file_path: Option<&'a Path>,file_size: u64) -> Self {
            TableDetails {
                file_path,
                table_name,
                file_size,
            }
        }
    }

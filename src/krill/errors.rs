    #[derive(Error,Debug)]
    pub enum KrillErrors {
        #[error("Tried to read CSV file: {0} , but file is invalid type")]
        InvalidCSV(String),
        #[error("Error attempting to parse ")]
        ParsingError,
        #[error("Empty CSV File")]
        EmptyCSV,
        #[error("Row Index : {0} Given, But Only {1}'s Exist")]
        RowOutOfBounds(usize,usize),
        #[error("Column INdex: {0} Given, But Only {1}'s Exist")]
        ColumnOutOfBounds(usize,usize),
        #[error("Column With Name: {0} Not Found")]
        ColumnNotFound(String),
        #[error("Column Expects Type {0}, But Type {1} Was Given")]
        ColumnTypeMismatched(ValueType,ValueType),
        #[error("Attempted Numeric Operation On Non-Numeric Type")]
        OperationOnNonNumericType,


    }


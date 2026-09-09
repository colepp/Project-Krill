

pub mod krill {
    use core::{fmt, panic};
    use std::collections::{HashMap};
    use std::error::Error;
    use std::ops::{Index, IndexMut};
    use std::{fs::*, println, vec, write};
    use std::io::{BufRead, BufReader};
    use thiserror::Error;

    use crate::krill::CountType::{IsEmpty, NonEmpty};
    use crate::krill::ValueType::{Float};

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
        #[error("Attempted {0} on type {1}")]
        OperationOnNonNumericType(String,ValueType)

    }

    #[derive(Clone)]
    pub struct Table {
        column_map: HashMap<String,ColumnDetails>,
        column_names: Vec<String>,
        rows: Vec<Row>,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct ColumnDetails {
        column_number: usize,
        value_type: ValueType,
    }

    #[derive(Debug,Clone,PartialEq)]
    pub struct Row{
        pub values: Vec<Value>,
        // potential to add, len / len minus empty
    }


    impl Row {
        pub fn get_value_from_row(&self, column_number: usize) -> Result<&Value,Box<dyn Error>> {
            let value = self.values.get(column_number).ok_or_else(|| Box::new(KrillErrors::ColumnOutOfBounds(column_number, self.values.len())))?;
            Ok(value)
        } 
    }

     #[derive(Debug,Clone)]


     /// Represents a Read-Only Column for iterating through
    pub struct Column{
        // column_details:  &'a ColumnDetails,
        row_len: usize,
        current_row: usize,

    }

    impl Iterator for Column {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.row_len {
            return None;
        }

        self.current_row += 1;
        Some(self.current_row)
        }
    }

    #[derive(Debug,Clone,PartialEq)]
    pub enum Value {
        String(std::string::String),
        Integer(i64),
        Float(f64),
        Boolean(bool),
        Empty,
    }

    impl From<i64> for Value {
        fn from(value: i64) -> Self {
            Value::Integer(value)
        }
    }

    impl From<f64> for Value {
        fn from(value: f64) -> Self {
            Value::Float(value)
        }
    }

    impl From<bool> for Value {
        fn from(value:bool) -> Self {
            Value::Boolean(value)
        }
    }

    impl From<String> for Value {
        fn from(value: String) -> Self {
            Value::String(value)
        }
    }

    impl From<&str> for Value {
        fn from(value: &str) -> Self {
            Value::String(value.to_string())
        }
    }

    impl Value {
        pub fn value_type(&self) -> ValueType {
            match self {
                Value::String(_) => ValueType::String,
                Value::Integer(_) => ValueType::Integer,
                Value::Float(_) => ValueType::Float,
                Value::Boolean(_) => ValueType::Boolean,
                Value::Empty => ValueType::EMPTY,
            }
        }

        pub fn convert(self,replacement_value: Value) -> Value {
            replacement_value
        }

        pub fn is_numeric(&self) -> bool {
            self.value_type() == ValueType::Integer || self.value_type() == Float
        }

        pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    }

    #[derive(Debug,Clone,Copy,PartialEq)]
    pub enum ValueType {
        String,
        Integer,
        Float,
        Boolean,
        Unset,
        EMPTY,
    }

    pub enum CountType<'a>{
        NonEmpty,
        IsEmpty,
        Conditional(Box<dyn Fn(&Value) -> bool + 'a>),
    }


    impl fmt::Display for ValueType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                ValueType::String => write!(f,"String"),
                ValueType::Integer => write!(f,"Integer"),
                ValueType::Float => write!(f,"Float"),
                ValueType::Boolean => write!(f,"Boolean"),
                _ => write!(f,"UNDEFINED")
            }
        }
    }

    

    impl Table {
        pub fn from_csv(path: &str) -> Result<Self, Box<dyn Error>> {

            // open and load files into buffer
            if !path.contains(".csv") {
                return Err(Box::new(KrillErrors::InvalidCSV(path.to_string())));
            }

            let csv_file = File::open(path)?;
            let reader = BufReader::new(csv_file);

            // covert the rows into an lines iterator then 
            let mut raw_rows = reader.lines().map(move |line| line.unwrap());

            let raw_header_row = raw_rows
            .next()
            .ok_or_else(|| KrillErrors::EmptyCSV)?;

            let column_names = Self::parse_row(raw_header_row)
            .ok_or_else(|| KrillErrors::ParsingError)?;

            let mut column_map = Self::map_columns(&column_names);

            let mut rows = vec![];
            for raw_row in raw_rows {

                let mut typed_rows = vec![];
                let untyped_row = Self::parse_row(raw_row).ok_or_else(|| KrillErrors::ParsingError)?;

                for(val_index,untyped_val) in untyped_row.iter().enumerate() {

                    let column_name = headers.get(val_index).ok_or_else(|| KrillErrors::ColumnOutOfBounds(val_index, headers.len()))?;
                    let column  = heading_map.get_mut(column_name).ok_or_else(|| KrillErrors::ColumnNotFound(column_name.clone()))?;

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


            Ok(Table { column_map, column_names, rows })
        }

        pub fn sum(&self, column_name: &str) -> Result<f64,Box<dyn Error>> {
            let column = self.column(column_name)?;
            let mut res: f64 = 0.0;


            for row in self.rows.iter() {
                let value = row.get_value_from_row(column.column_number)?;
                if !value.is_numeric() && value.value_type() != ValueType::EMPTY {
                    return Err(Box::new(KrillErrors::OperationOnNonNumericType("SUM".to_string(), val.value_type().clone())));
                }
                match value {
                    Value::Float(f) => res += *f,
                    Value::Integer(i) => res += *i as f64,
                    _ => ()
                };
            }

           Ok(res) 
        }

        pub fn mean(&self, column_name: &str) -> Result<f64,Box<dyn Error>>{
            Ok(self.sum(column_name)?/ self.count(column_name, CountType::NonEmpty)? as f64)
        }

        pub fn count(&self,column_name: &str, counting_type: CountType) -> Result<usize,Box<dyn Error>> {
            
            let column = self.column(column_name)?;
            let mut count: usize = 0;
            for row in self.rows.iter() {
                let value = row.get_value_from_row(column.column_number)?;
                match counting_type {
                    IsEmpty => if value.value_type() == ValueType::EMPTY {count = count + 1;},
                    NonEmpty => if value.value_type() != ValueType::EMPTY {count = count + 1;},
                    CountType::Conditional(ref func) => if func(value) {count = count + 1;},
                }
            }
            Ok(count)
        }

        pub fn fillna_column<'a, T>(&mut self,column_name: &str,replacement_function: Fn(&Table,usize) -> Result<Value,Box<dyn Error>> + 'a, inplace: bool) -> Result<Option<Table>,Box<dyn Error>>
        where T: Into<Value> + Clone {

            
            

            // let typed_value = replacement_value.into();
            
            if inplace {

                let column = self.column(column_name)?.clone();
                let column_number = column.column_number;

                for row_number in 0..self.column_names.len() {
                if self.rows.get(row_number)
                .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number, self.rows.len())))?
                .get_value_from_row(column_number)?
                .value_type() == ValueType::EMPTY {
                    Self::replace_value_on_row(&mut self.rows,row_number, column_number, typed_value.clone())?;
                }
                return Ok(None);
                }
            }

            let mut copy_table = self.clone();
            for row_number in 0..copy_table.column_names.len() {

            
                let column = copy_table.column(column_name)?.clone();
                let column_number = column.column_number;

                if copy_table.rows.get(row_number)
                .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number, copy_table.rows.len())))?
                .get_value_from_row(column_number)?
                .value_type() == ValueType::EMPTY {
                    Self::replace_value_on_row(&mut copy_table.rows,row_number, column_number, typed_value.clone())?;
                }
            }   
            Ok(Some(copy_table))
        }


        

        pub fn replace_value_on_row(rows: &mut [Row],row_number: usize,column_number: usize, value: Value) -> Result<(),Box<dyn Error>> {
            if let Some(row) = rows.get_mut(row_number) {
                row.values[column_number] = value;
                return Ok(());
            }
            Err(Box::new(KrillErrors::RowOutOfBounds(row_number, rows.len())))
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
                return Value::Integer(parsed_val);
            } 

            if let Ok(parsed_val) = raw_value.parse::<f64>() {
                return Value::Float(parsed_val);
            }
            
            // set this up later for numeric bools like 0 for true 1 for false 
            if let Ok(parsed_val) = raw_value.parse::<bool>() {
                return Value::Boolean(parsed_val);
            }

            Value::String(raw_value.to_string())
        
        }
        
        pub fn rows(&self) -> &[Row] {
            &self.rows
        }

        pub fn rows_mut(&mut self) -> &mut [Row] {
            &mut self.rows
        }

        pub fn get_row(&self,row_number: usize) -> Result<&Row,Box<dyn Error>> {
            Ok(self.rows
            .get(row_number)
            .ok_or_else(|| Box::new(KrillErrors::RowOutOfBounds(row_number,self.rows.len())))?)
        }

        pub fn get_row_mut(&mut self, row_number: usize) -> Result<&mut Row,KrillErrors> {
            let len: usize = self.rows.len();
            self.rows.get_mut(row_number).ok_or_else(|| KrillErrors::RowOutOfBounds(row_number,len ))
        }

        pub fn column(&self, name: &str) -> Result<&ColumnDetails,KrillErrors> {
            
            let column_details = self.column_map.get(name).ok_or_else(|| KrillErrors::ColumnNotFound(name.to_string()))?;
            Ok(column_details)
        }

        pub fn dimensions(&self) -> (usize,usize) {
            (self.column_names.len(),self.rows.len())
        }

        pub fn describe(&self) {
            for (column_name,column) in self.column_map.iter() {
                println!("{}: {}",column_name.to_uppercase(),column.value_type);
            }
        }

        pub fn column_type(&self,column_name: &str) -> Result<ValueType,KrillErrors> {
            let column = self.column_map.get(column_name).ok_or_else(|| KrillErrors::ColumnNotFound(column_name.to_string()))?;
            Ok(column.value_type.clone())
        }


    }


    impl Index<usize> for Table {
        type Output = Row;
    
        fn index(&self, index: usize) -> &Self::Output {
            self.get_row(index).unwrap_or_else(|err| {
            panic!("Failed to index Table: {err:?}")})
        }
    }

    impl IndexMut<usize> for Table {
        
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            self.get_row_mut(index).unwrap_or_else(|err| {
            panic!("Failed to index Table: {err:?}")})
        }
            
    }

}

#[cfg(test)]
pub mod krill_testing {

    use core::panic;
use std::assert_eq;
    use std::println;
    use std::vec;


    use crate::krill::Row;
    use crate::krill::Table;
    use crate::krill::Value;
    use super::*;


    #[test]
    fn col_read() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let ages = table.column("age").unwrap();
        table.describe();
    }

    #[test]
    fn count_non_empty_test() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let count = table.count("batting_average", krill::CountType::NonEmpty).unwrap();
        // println!("{}",count);
        assert_eq!(count,5);
    }

    #[test]
    fn count_empty_test() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let count = table.count("batting_average", krill::CountType::IsEmpty).unwrap();
        println!("{}",count);
        // assert_eq!(count,5);
    }

    #[test]
    fn count_conditional_test() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let count = table.count("batting_average", krill::CountType::Conditional(Box::new(|avg| avg.as_float().is_some_and(|v| v > 0.285 )))).unwrap();
        println!("{}",count);
    }

    #[test]
    fn fill_na_test() {
        let mut table = krill::Table::from_csv("./test_data/baseball_stats.csv").unwrap();
        // let col = table.column("batting_average").unwrap();
        let res = table.fillna_column("batting_average", 0.00, true).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn mean_test() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let count = table.mean("batting_average").unwrap();
        println!("{}",count);
        // assert_eq!(count,5);

    }


    #[test]
    fn col_read_fail() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let bad_column = table.column("non-existent-column");
        assert!(bad_column.is_err());
        
        let err = bad_column.unwrap_err();
        assert_eq!(err.to_string(),"Column With Name: non-existent-column Not Found")
    }



    #[test]
    fn csv_row_read() {
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();

        let player_1 = table.get_row(0).unwrap();

        let player_1_res = Row{
            values: vec![Value::String(String::from("Mike")),
             Value::Integer(24),
              Value::String(String::from("Cubs")),
               Value::String(String::from("SS")),
                Value::Float(0.284)]
        };

        println!("{:?}",player_1);
        // assert_eq!(player_1,&player_1_res);
    }

    #[test]
    fn read_row_with_missing_value(){
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let player_1 = table.get_row(0).unwrap().clone().values;
        assert_eq!(player_1,
            vec![Value::String(String::from("Alex")),
             Value::Integer(26),
              Value::String(String::from("Giants")),
               Value::String(String::from("3B")),
                Value::Empty]
        );
        // println!("{:?}",player_1);

    }

    #[test]
    fn dimensions_check(){
        let table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        let (n_col,n_row) = table.dimensions();
        assert_eq!((n_col,n_row),(5,6));
        
        // println!("{},{}",n_col,n_row);
    }

    #[test]
    fn fill_na_with_func_enum() {
        let batting_average = |table: &krill::Table,row: usize| {
            let at_bats = table.get_row(row).unwrap_or_else(|err| panic!("placeholder"))
            .get_value_from_row(2).unwrap().as_integer(); // lets say hits is here
            assert!(at_bats.is_some());
            let at_bats = at_bats.unwrap();

            let hits = table.get_row(row).unwrap_or_else(|err| panic!("placeholder"))
            .get_value_from_row(3).unwrap().as_integer(); // lets say hits is here
            assert!(hits.is_some());
            let hits = hits.unwrap();


            hits as f64 / at_bats as f64

        };

        let mut table = krill::Table::from_csv("baseball_stats.csv").unwrap();
        table.fillna_column("batting_average", Box::new(batting_average), false);

    }

}
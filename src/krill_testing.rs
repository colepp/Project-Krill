
    use std::{assert_eq, error::Error};
    use std::path::Path;
    use std::println;
    

    use ordered_float::NotNan;

    use crate::krill::{Table, Value};
    use super::*;



    // #[test]
    // fn fill_na_test() {
    //     let path = Path::new("./test_data/test input/baseball_stats.csv");
    //     let mut table = krill::Table::from_csv(path).unwrap();
    //     // let col = table.column("batting_average").unwrap();


    //     // let res = table.fillna_column("avg", fill_avg, true).unwrap();
    //     // assert!(res.is_none());
    //     println!("{table}");
    // }

    #[test]
    fn equality_checks_for_value_on_non_value_types(){
        let test_value_int = Value::Integer(5);
        assert!(test_value_int == 5);

        let test_value_float = Value::from(5.5);
        assert!(test_value_float == 5.5);

        let test_value_string = Value::String("Apple".to_string());
        assert!(test_value_string == "Apple".to_string());

    }

    #[test]
    fn ordering_checks_for_value_on_non_value_types() {
        let test_value_int = Value::Integer(5);
        assert!(test_value_int < 6);
        assert!(test_value_int > 4);
        assert!(test_value_int >= 5);

        let test_value_float = Value::from(5.5);
        assert!(test_value_float < 6.5);
        assert!(test_value_float > 4.0);
        assert!(test_value_float >= 5.2);
    }

    #[test]
    fn sum_test() {
        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let table = krill::Table::from_csv(path).unwrap();

        let int_res = table.sum("integer_values").unwrap();
        assert_eq!(int_res,Value::Integer(140));
        println!("Result: {}\nValueType: {}",int_res,int_res.value_type());

        let float_res = table.sum("float_values").unwrap();
        assert_eq!(float_res,Value::Float(NotNan::new(35.0).unwrap()));
        println!("Result: {}\nValueType: {}",float_res,float_res.value_type());
    }

    #[test]
    fn count_test(){

        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let table = krill::Table::from_csv(path).unwrap();

        let int_empty_res = table.count("integer_values",krill::CountType::IsEmpty).unwrap();
        assert_eq!(int_empty_res,3);
        println!("Result: {} empty values in integer_values",int_empty_res);
        
        let int_nonempty_res = table.count("integer_values",krill::CountType::NonEmpty).unwrap();
        assert_eq!(int_nonempty_res,9);
        println!("Result: {} nonempty values in integer_values",int_nonempty_res);

        let int_conditional_res = table.count("integer_values",krill::CountType::Conditional(Box::new(|value| *value > 10))).unwrap();
        assert_eq!(int_conditional_res,5);
        println!("Result: {} values in integer_values greater than 10",int_conditional_res);

        let float_empty_res = table.count("float_values",krill::CountType::IsEmpty).unwrap();
        assert_eq!(float_empty_res,3);
        println!("Result: {} empty values in float_values",int_empty_res);

    }

    #[test]
    fn mean_test() {

        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let table = krill::Table::from_csv(path).unwrap();

        let int_res = table.mean("integer_values").unwrap();
        assert_eq!(int_res,Value::from(15.555555555555555));
        println!("Result: {}\nValueType: {}",int_res,int_res.value_type());

        let float_res = table.mean("float_values").unwrap();
        assert_eq!(float_res,Value::Float(NotNan::new(3.888888888888889).unwrap()));
        println!("Result: {}\nValueType: {}",float_res,float_res.value_type());

    }

    #[test]
    pub fn mode_test() {
        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let table = krill::Table::from_csv(path).unwrap();

        let int_mode = table.mode("integer_values").unwrap();
        assert_eq!(int_mode,Value::Integer(20));
        println!("Result: {}\nValueType: {}",int_mode,int_mode.value_type());

        let float_mode = table.mode("float_values").unwrap();
        assert_eq!(float_mode,Value::from(5.0));
        println!("Result: {}\nValueType: {}",float_mode,float_mode.value_type());

    }



    #[test]
    pub fn save_to_csv_test() {
        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let mut table = krill::Table::from_csv(path).unwrap();

        table.describe();

        let new_path = Path::new("./test_data/test_outputs/test_save_output.csv");
        table.to_csv(new_path,true);

    }

    #[test]
    pub fn replace_all_test() {
        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let mut table = krill::Table::from_csv(path).unwrap();

        let replace_function = |table: &Table, row_number: usize| {
            Ok(Value::Integer(0))
        };

        table.describe();

        let res = table.fillna_column("integer_values",replace_function, true);
        
        table.describe();

        let new_path = Path::new("./test_data/test_outputs/test_fillna_output.csv");
        table.to_csv(new_path, true);

    }

    #[test]
    pub fn add_col_empty_test() {
        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let mut table = krill::Table::from_csv(path).unwrap();

        
        table.describe();

        let res = table.add_column_empty("new_empty_column" ,true);
        
        table.describe();

        let new_path = Path::new("./test_data/test_outputs/test_add_column_output.csv");
        table.to_csv(new_path, true);
    }


    #[test]
    pub fn add_col_default_test() {
        let path = Path::new("./test_data/test input/column_operations_test.csv");
        let mut table = krill::Table::from_csv(path).unwrap();

        let default_value = Value::String(String::from("DEFAULT"));
        
        // let _ = table.describe();

        let res = table.add_column_default(default_value ,"new_default_string_column_column",true);
        
        let _ = table.describe();

        let new_path = Path::new("./test_data/test_outputs/test_add_column_default_output.csv");
        let _ = table.to_csv(new_path, false);
    }

    


    
   


#[path = "krill_testing/failure_cases.rs"]
mod failure_cases;

#[path = "krill_testing/table_mutations.rs"]
mod table_mutations;

#[path = "krill_testing/row_filters.rs"]
mod row_filters;

#[path = "krill_testing/find_rows.rs"]
mod find_rows;

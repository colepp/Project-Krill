    #[derive(Debug,Clone,PartialEq)]
    pub struct Row{
        pub values: Vec<Value>,
        // potential to add, len / len minus empty
    }

    impl fmt::Display for Row {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            let values = self
                .values
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            write!(formatter, "{values}")
        }
    }


    impl Row {



        pub fn get_value_from_row(&self, column_number: usize) -> Result<&Value,Box<dyn Error>> {
            let value = self.values.get(column_number).ok_or_else(|| Box::new(KrillErrors::ColumnOutOfBounds(column_number, self.values.len())))?;
            Ok(value)
        }

        pub fn remove_value_from_row(&mut self, column_number: usize) -> Result<(),Box<dyn Error>> {
            
        }
    }


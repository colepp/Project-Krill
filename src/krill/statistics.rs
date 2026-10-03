    pub enum CountType<'a>{
        NonEmpty,
        IsEmpty,
        Conditional(Box<dyn Fn(&Value) -> bool + 'a>),
    }

    impl<'a> Table<'a> {
        // read-only data operations

        pub fn sum(&self, column_name: &str) -> Result<Value,Box<dyn Error>> {
            let mut res = Value::from(0);
            for value in self.column_iter(column_name)? {
                res = res + value.clone();
            }
            Ok(res)
        }

        pub fn mean(&self, column_name: &str) -> Result<Value,Box<dyn Error>>{
            Ok(self.sum(column_name)? / (self.count(column_name, CountType::NonEmpty)? as f64))
        }

        pub fn count(&self,column_name: &str, counting_type: CountType) -> Result<usize,Box<dyn Error>> {
            let mut count:usize = 0;

            for value in self.column_iter(column_name)? {
                match counting_type {
                    CountType::IsEmpty => {if value.is_empty() {count += 1}},
                    CountType::NonEmpty => {if !value.is_empty() {count += 1}},
                    CountType::Conditional(ref condition) => {if !value.is_empty() && condition(&                                                           value) {count += 1}} 
                }
            }

            Ok(count)

        }

        pub fn median(&self, column_name: &str) -> Result<Value,Box<dyn Error>> {
            let mut column = self.column_deep_copy(column_name)?;
            column.sort();
            let med = column.len() / 2;
            if column.len() % 2 == 0 {
                let l = column.get(med).ok_or_else(|| Box::new(KrillErrors::ColumnOutOfBounds(med,column.len())))?.clone();
                let r = column.get(med - 1).ok_or_else(|| Box::new(KrillErrors::ColumnOutOfBounds(med + 1,column.len())))?.clone();
                Ok((l + r) / 2.0)
            } else {
                Ok(column.get(med).ok_or_else(|| Box::new(KrillErrors::ColumnOutOfBounds(med,column.len())))?.clone())
            }
        
        }

        pub fn mode(&self, column_name: &str) -> Result<Value,Box<dyn Error>> {
            let mut freq_map: HashMap<Value,usize> = HashMap::new();
            
            for value in self.column_iter(column_name)?.map(|reference| reference.clone())  {
                if !value.is_empty(){
                    let count = freq_map.entry(value).or_insert(0);
                    *count += 1;
                }
            }

            let mut mode = Value::Empty;
            let mut mode_freq = 0;
            for (k,v) in freq_map.iter()  {
                if *v > mode_freq{
                    mode_freq = *v;
                    mode = k.clone();
                }
            }
            Ok(mode)
        }

    }

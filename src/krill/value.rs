    #[derive(Debug,Clone,PartialEq, PartialOrd,Ord,Eq,Hash)]
    pub enum Value {
        String(String),
        Integer(i64),
        Float(NotNan<f64>),
        Boolean(bool),
        Empty,
    }

    

    impl PartialEq<i64> for Value {
        fn eq(&self, other: &i64) -> bool {
            let Value::Integer(value) = self else {
                panic!("Integer Equality Comparison done on type {}, only Values of ValueType::Integer can be used for integer comparison",self.value_type())
            };
            value == other
        }
    }

    impl PartialOrd<i64> for Value {
        fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
            let Value::Integer(value) = self else {
                panic!("Integer Order Comparison done on type {}, only Values of ValueType::Integer can be used for integer comparison",self.value_type())
            };
            value.partial_cmp(other)
        }
    }

    impl PartialEq<f64> for Value {
        fn eq(&self, other: &f64) -> bool {
            let Value::Float(value) = self else {
                panic!("Float Equality Comparison done on type {}, only Values of ValueType::Float can be used for integer comparison",self.value_type())
            };
            value == other
        }
    }

    impl PartialOrd<f64> for Value {
        fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
            let Value::Float(value) = self else {
                panic!("Float Order Comparison done on type {}, only Values of ValueType::Float can be used for integer comparison",self.value_type())
            };
            let wrapped_other = NotNan::new(other.clone()).unwrap();
            value.partial_cmp(&wrapped_other)
        }
    }

    impl PartialEq<String> for Value {
        fn eq(&self, other: &String) -> bool {
            let Value::String(value) = self else {
                panic!("String Order Comparison done on type {}, only Values of ValueType::String can be used for integer comparison",self.value_type())
            };
            *value == *other
        }
    }


    impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(value) => write!(formatter, "{value}"),
            Value::Integer(value) => write!(formatter, "{value}"),
            Value::Float(value) => write!(formatter, "{value}"),
            Value::Boolean(value) => write!(formatter, "{value}"),
            Value::Empty => write!(formatter, ""),
        }
    }
}

    impl From<i64> for Value {
        fn from(value: i64) -> Self {
            Value::Integer(value)
        }
    }

    impl From<f64> for Value {
        fn from(value: f64) -> Self {
            Value::Float(NotNan::new(value).unwrap())
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


    impl Add<Value> for Value {
        type Output = Value;
    
        fn add(self, rhs: Value) -> Self::Output {
            if (!self.is_numeric() || !rhs.is_numeric()) && !rhs.is_empty() && !self.is_empty() {panic!("One or more values in addition operation is not an numeric type")}
            match self{
                Value::Integer(lhs) => {
                    match rhs {
                        Value::Integer(rhs) => Value::from(lhs + rhs),
                        Value::Float(rhs) => Value::from(lhs as f64 + *rhs),
                        Value::Empty => self,
                        _ => panic!("Righ hand side operator is either non numeric or some unkown ValueType")
                    }
                },
                Value::Float(lhs) => {
                    match rhs {
                        Value::Integer(rhs) => Value::from(*lhs + rhs as f64),
                        Value::Float(rhs) => Value::from(*lhs + *rhs),
                        Value::Empty => self,
                        _ => panic!("Right hand side operator is either non numeric or some unkown ValueType")
                    }
                },
                Value::Empty => {
                    match rhs {
                        Value::Integer(rhs) => Value::from(rhs),
                        Value::Float(rhs) => Value::from(*rhs),
                        Value::Empty => Value::from(0.0),
                        _ => panic!("Righ hand side operator is either non numeric or some unkown ValueType")
                    }
                },
                _ => panic!("Left hand side operator is either non numeric or some unkown ValueType")
            }

        }
    }

    impl Add<i64> for Value {
        type Output = Value;
    
        fn add(self, rhs: i64) -> Self::Output {
            if !self.is_numeric() && !self.is_empty() {panic!("One or more values in addition operation is not an numeric type")}
            match self{
                Value::Integer(lhs) => Value::from(lhs + rhs),
                Value::Float(lhs) =>Value::from(*lhs + rhs as f64),
                Value::Empty => Value::from(rhs),
                _ => panic!("Left hand side operator is either non numeric or some unkown ValueType")
            }

        }
    }

    impl Add<f64> for Value {
        type Output = Value;
    
        fn add(self, rhs: f64) -> Self::Output {
            if !self.is_numeric() && !self.is_empty() {panic!("left hand side operators in addition operation is not an numeric type")}
            match self{
                Value::Integer(lhs) => Value::from(lhs as f64 + rhs),
                Value::Float(lhs) =>Value::from(*lhs + rhs),
                Value::Empty => Value::from(rhs),
                _ => panic!("Left hand side operator is either non numeric or some unkown ValueType")
            }
        }
    }


    // all div operations return ValueFloats
    impl Div for Value {
        type Output = Value;

        fn div(self, rhs: Value) -> Self::Output {
            if (!self.is_numeric() || !rhs.is_numeric()) && rhs.is_empty() && !self.is_empty() {panic!("One or more values in addition operation is not an numeric type")}
            match self{
                Value::Integer(numerator) => {
                    match rhs {
                        Value::Integer(denominator) => {
                            if denominator == 0 {panic!("Error divide by 0 error")}
                            Value::from(numerator as f64 / denominator as f64)},
                        Value::Float(denominator) =>{
                            if denominator == 0.0 {panic!("Error divide by 0 error")}
                            Value::from(numerator as f64 / *denominator)},
                        _ => panic!("Right hand side operator is either non numeric or some unkown ValueType")
                    }
                },
                Value::Float(numerator) => {
                    match rhs {
                        Value::Integer(denominator) => Value::from(*numerator / denominator as f64),
                        Value::Float(denominator) => Value::from(*numerator / *denominator),
                        _ => panic!("Right hand side operator is either non numeric or some unkown ValueType")
                    }
                },
                Value::Empty => {
                    match rhs {
                        Value::Integer(_) => Value::from(0.0),
                        Value::Float(_) => Value::from(0.0),
                        _ => panic!("Right hand side operator is either non numeric or some unkown ValueType")
                    }
                },
                _ => panic!("Left hand side operator is either non numeric or some unkown ValueType")
            }

        }
    }
    
    impl Div<f64> for Value {
        type Output = Value;
    
        fn div(self, rhs: f64) -> Self::Output {
            if rhs == 0.0 {panic!("Error divide by zero on right hand side")}
            match self {
                Value::Integer(lhs) => Value::from(lhs as f64 / rhs),
                Value::Float(lhs) => Value::from(*lhs / rhs),
                Value::Empty => Value::from(0.0),
                _ => panic!("lhs value in Division operation is not an numeric type\n is type: {}",self.value_type())
            }
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

        pub fn is_empty(&self) -> bool {
            self.value_type() == ValueType::EMPTY
        }

        pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            _ => None,
        }
    }


    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(n) => Some(*(n.clone())),
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
    
    impl ValueType {
        pub fn is_numeric(&self) -> bool{
            return *self == ValueType::Integer || *self == ValueType::Float
        }
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

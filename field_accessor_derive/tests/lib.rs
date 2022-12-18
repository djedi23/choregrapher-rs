extern crate field_accessor_derive;

#[cfg(test)]
mod tests {
  use field_accessor_derive::FieldAccessor;
  use serde::Serialize;
  use std::ops::{Mul, Sub};

  trait FieldAccessor {
    fn field_list(self) -> Vec<String>;
    /// Return a string representation of the fielf
    ///
    fn field(self) -> Vec<String>;
  }

  #[derive(FieldAccessor, Serialize)]
  pub enum Fact<T>
  where
    T: Sub + Mul,
  {
    //    #[serde(rename = "n")]
    N(T, T),
    #[serde(rename = "fact")]
    F(T),
  }

  #[derive(FieldAccessor, Serialize)]
  pub struct Output<T>
  where
    T: Sub + Mul,
  {
    n: T,
    #[serde(rename = "out")]
    o: T,
  }

  #[test]
  fn fieldaccessor_field_1() {
    let fact = Fact::N(5, 4);
    assert_eq!(fact.field(), vec!["n"]);
  }
  #[test]
  fn fieldaccessor_field_2() {
    let fact = Fact::F(5);
    assert_eq!(fact.field(), vec!["fact"]);
  }

  #[test]
  fn fieldaccessor_field_list_1() {
    let fact = Fact::N(5, 4);
    assert_eq!(fact.field_list(), vec!["n".to_string(), "fact".to_string()]);
  }

  #[test]
  fn fieldaccessor_field_struct_1() {
    let out = Output { n: 5, o: 8 };
    assert_eq!(out.field(), vec!["n".to_string(), "out".to_string()]);
  }
  #[test]
  fn fieldaccessor_field_list_struct_1() {
    let out = Output { n: 5, o: 8 };
    assert_eq!(out.field_list(), vec!["n".to_string(), "out".to_string()]);
  }
}

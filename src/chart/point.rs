extern crate chrono;
use chrono::{DateTime, Utc};

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct Point {
  pub value: f64,
  pub timestamp: DateTime<Utc>,
}

impl Point {
  pub fn new(value: f64, timestamp: DateTime<Utc>) -> Point {
    Point {value: value, timestamp: timestamp}
  }
}

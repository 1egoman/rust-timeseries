extern crate chrono;
use chrono::{DateTime, Utc};

use chart::point::Point;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct PointIndex {
  pub timestamp: Option<DateTime<Utc>>,
  pub less: usize,
  pub more: usize,
  pub parent: usize,
  pub data: Option<Vec<Point>>,
}

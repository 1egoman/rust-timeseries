extern crate chrono;
use chrono::{DateTime, Utc, TimeZone};

use chart::point::Point;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct PointIndex {
  pub timestamp: Option<DateTime<Utc>>,
  pub less: usize,
  pub more: usize,
  pub parent: usize,
  pub data: Option<Vec<Point>>,
}

impl PointIndex {

  // Return the default value used in projections - ie, when an node in the index tree is removed
  // due to a filter, what should the value be? Eventually, it may be a good idea to make this a
  // shared read-only static reference instead of giving each projection its own default value.
  pub fn new_default_value() -> PointIndex {
    PointIndex{
      timestamp: Some(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
      less: 0,
      more: 0,
      parent: 0,
      data: Some(vec![]),
    }
  }
}

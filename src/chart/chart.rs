extern crate chrono;
use chrono::{DateTime, Utc};

use chart::point::Point;
use chart::point_index::PointIndex;

fn linear_interpolation(starting_value: f64, ending_value: f64, percentage: f64) -> f64 {
  starting_value + ((ending_value - starting_value) * percentage)
}

pub struct Chart {
  pub points: Vec<Point>,
  pub index: Vec<PointIndex>,

  pub max_index_node_capacity: usize,
}

impl Chart {
  pub fn get_value_vec(&self, timestamp: DateTime<Utc>) -> Option<f64> {
    // Find the point before the passed-in timestamp
    let point_iterator = 0..self.points.len();
    for index in point_iterator.rev() {
      if self.points[index].timestamp < timestamp {
        let point_before = &self.points[index];

        // If at the most recent point, then no interpolation can be done. Return the final point.
        if index == self.points.len()-1 {
          return Some(point_before.value);
        }

        let point_after = &self.points[index+1];
        return Some(self.interpolate_between_points(timestamp, &point_before, &point_after));
      }
    }

    return None
  }

  pub fn interpolate_between_points(&self, timestamp: DateTime<Utc>, point_before: &Point, point_after: &Point) -> f64 {
    // Figure out the percentage between the points point before and the point after that
    // `timestamp` represents.
    let time_to_timestamp_ms = (
      timestamp.timestamp_millis() - point_before.timestamp.timestamp_millis()
    ) as f64;
    let time_between_points_ms = (
      point_after.timestamp.timestamp_millis() - point_before.timestamp.timestamp_millis()
    ) as f64;
    let percentage_between_points = time_to_timestamp_ms / time_between_points_ms;

    // Don't interpolate if not required
    if percentage_between_points == 0.0 {
      return point_before.value;
    }
    if percentage_between_points == 1.0 {
      return point_after.value;
    }

    // Interpolate to find the actual value
    let result = linear_interpolation(
      point_before.value,
      point_after.value,
      percentage_between_points,
    );
    return result;
  }
}


extern crate chrono;
use chrono::{DateTime, Utc, NaiveDateTime};

use chart::chart::Chart;
use chart::point_index::PointIndex;
use chart::projection::Projection;
use chart::projection::ProjectionDisposable;

impl Chart {
  pub fn rebalance_index_node(&mut self, node_index: usize) {
    /* println!("\n\nRebalancing {}", node_index); */

    let mut rebalanced_node = false;
    let mut less: Option<PointIndex> = None;
    let mut more: Option<PointIndex> = None;

    // Borrow a mutable reference to the node at `node_index`.
    {
      let node = &mut self.index[node_index];

      match node.data {
        Some(ref data) => {
          // If this node isn't balanced...
          if data.len() > self.max_index_node_capacity {
            rebalanced_node = true;
            let data_length = data.len();

            // Calculate average timestamp of all data items in the node. Set that average timestamp
            // equal to `node.timestamp` (as it's the natural breaking point into two smaller nodes)
            let mut average_timestamp: i64 = 0;
            for item in data {
              average_timestamp += item.timestamp.timestamp();
            }
            average_timestamp /= data_length as i64;

            let timestamp_split_point = DateTime::from_utc(
              NaiveDateTime::from_timestamp(average_timestamp, 0),
              Utc,
            );
            node.timestamp = Some(timestamp_split_point);

            /* print!("Split timestamp: {:?}\n", timestamp_split_point); */

            // Create a two new nodes. All less than the average timestamp goes in one node, all more
            // than it goes into the other. The timestamp in these nodes is None.

            let mut index_of_timestamp_split: usize = data_length-1;
            for index in 0..data_length {
              if data[index].timestamp > timestamp_split_point {
                index_of_timestamp_split = index;
                break;
              }
            }

            // Note: if the timestamp is equal to the split timestamp, it'll end up on the `less`
            // side.

            /* print!("Split index: {:?}\n", index_of_timestamp_split); */

            less = Some(PointIndex {
              timestamp: None,
              less: 0,
              more: 0,
              parent: node_index,
              data: Some(data[..index_of_timestamp_split].to_vec()),
            });

            more = Some(PointIndex {
              timestamp: None,
              less: 0,
              more: 0,
              parent: node_index,
              data: Some(data[index_of_timestamp_split..].to_vec()),
            });
          }
        }
        None => ()
      }
    }

    if rebalanced_node {
      self.index[node_index].data = None;

      // Set `node.less` and `node.more` equal to the indexes of the two new nodes.
      let unwrapped_less = less.unwrap();
      let unwrapped_more = more.unwrap();

      self.index.push(unwrapped_less);
      let less_index = self.index.len()-1;
      self.index[node_index].less = less_index;

      self.index.push(unwrapped_more);
      let more_index = self.index.len()-1;
      self.index[node_index].more = more_index;

      // Call this function on each new node.
      self.rebalance_index_node(less_index);
      self.rebalance_index_node(more_index);
    }
  }
  pub fn build_index(&mut self) {
    // Start by making a single index for all items
    self.index.push(
      PointIndex {
        timestamp: Some(Utc::now()),
        less: 0,
        more: 0,
        parent: 0,
        data: Some(self.points.clone()),
      }
    );

    // Then, break up that node into smaller nodes
    self.rebalance_index_node(0);
  }

  // Given a timestamp and a projection, project the chart and return the value at that point using
  // the on the projected chart.
  pub fn get_value_projection(
    &self,
    timestamp: DateTime<Utc>,
    projection: Option<(&Projection, &mut ProjectionDisposable)>,
  ) -> Option<f64> {
    debug!(
      "CALLING chart.get_value_projection({:?}, {})",
      timestamp,
      if projection.is_some() { "<projection>" } else { "None" }
    );
    if let Some(node_index) = self.lookup_in_index(timestamp) {
      debug!("Timestamp {} is in node index {}", timestamp, node_index);
      let (node, projection) = self.project_index_node(node_index, projection);
      if let Some(ref node_data) = node.data {
        if node_data.len() == 0 {
          debug!("Index node {} is empty, returning None", node_index);
          return None;
        } else if timestamp < node_data[0].timestamp {
          debug!("Need index node LESS than {} in order to interpolate", node_index);
          // Need to fetch node index less than current node index in order to get item smaller than
          // current value.
          if let Some(node_less_index) = self.get_node_less_than(node_index) {
            let (node_less, projection) = self.project_index_node(node_less_index, projection);
            if let Some(ref node_less_data) = node_less.data {
              if node_less_data.len() > 0 {
                let smaller_value = &node_less_data[node_less_data.len()-1];
                let larger_value = &node_data[0];
                let interpolated_value = self.interpolate_between_points(
                  timestamp,
                  smaller_value,
                  larger_value,
                );
                return Some(interpolated_value);
              }
            }
          }
        } else if timestamp > node_data[node_data.len()-1].timestamp {
          debug!("Need index node MORE than {} in order to interpolate", node_index);
          // Need to fetch node index more than current node index in order to get item larger than
          // current value.
          if let Some(node_more_index) = self.get_node_more_than(node_index) {
            let (node_more, projection) = self.project_index_node(node_more_index, projection);
            if let Some(ref node_more_data) = node_more.data {
              if node_more_data.len() > 0 {
                let smaller_value = &node_data[node_data.len()-1];
                let larger_value = &node_more_data[0];
                let interpolated_value = self.interpolate_between_points(
                  timestamp,
                  smaller_value,
                  larger_value,
                );
                return Some(interpolated_value);
              }
            }
          }
        } else {
          debug!("No other index nodes needed other than {}", node_index);
          for point_index in 1..node_data.len()-1 {
            if timestamp < node_data[point_index].timestamp {
              let smaller_value = &node_data[point_index];
              let larger_value = &node_data[point_index+1];
              return Some(self.interpolate_between_points(timestamp, smaller_value, larger_value));
            }
          }

          // For checking the last item, the logic is slightly different.
          if timestamp <= node_data[node_data.len()-1].timestamp {
            let smaller_value = &node_data[node_data.len()-2];
            let larger_value = &node_data[node_data.len()-1];
            return Some(self.interpolate_between_points(timestamp, smaller_value, larger_value));
          }
        }
      }
    }

    None
  }

  // Given a timestamp, return the value found at that location on the chart.
  pub fn get_value(&self, timestamp: DateTime<Utc>) -> Option<f64> {
    self.get_value_projection(timestamp, None /* no projection */)
  }

  // Find the location in the index that would contain a node with `timestamp`.
  pub fn lookup_in_index(&self, timestamp: DateTime<Utc>) -> Option<usize> {
    debug!("CALLING chart.lookup_in_index({:?})", timestamp);
    if self.index.len() == 0 {
      panic!("Index is not built, this is a requirement to lookup in the index!");
    }

    let mut node_index = 0;
    loop {
      match self.index[node_index].timestamp {
        Some(node_timestamp) => {
          // At a regular node. Figure out which leg to traverse down.
          debug!("Looking up regular node {}", node_index);
          if node_timestamp > timestamp {
            debug!("Moving in the LESS direction: {} => {}", node_index, self.index[node_index].less);
            node_index = self.index[node_index].less;
          } else {
            debug!("Moving in the MORE direction: {} => {}", node_index, self.index[node_index].more);
            node_index = self.index[node_index].more;
          }
          continue;
        }
        None => {
          // At a leaf.
          debug!("Reached leaf {}", node_index);
          break
        }
      }
    }

    debug!("DONE chart.lookup_in_index({:?})", timestamp);
    Some(node_index)
  }

  fn get_node_less_than(&self, node_index: usize) -> Option<usize> {
    let parent_node_index = self.index[node_index].parent;
    let parent_parent_node_index = self.index[parent_node_index].parent;

    if self.index[parent_node_index].less == node_index {
      //
      //          &
      //    Less / \ More
      //        /   \
      //       O     #
      //      / \   / \
      //         $ @   \
      //     end-^ ^-start
      //
      // We are at the # (node_index), traversing down to the @.
      // The node with values less than @ is $. Traversing to it requires knowlege of &
      // (parent_node_index).
      //
      return Some(self.index[self.index[parent_parent_node_index].less].more);
    }

    if self.index[parent_node_index].more == node_index {
      return Some(self.index[self.index[node_index].parent].less);
    }

    None
  }
  fn get_node_more_than(&self, node_index: usize) -> Option<usize> {
    let parent_node_index = self.index[node_index].parent;
    let parent_parent_node_index = self.index[parent_node_index].parent;

    if self.index[parent_node_index].more == node_index {
      //
      //          &
      //    Less / \ More
      //        /   \
      //       #     O
      //      / \   / \
      //         @ $   \
      //   start-^ ^-end
      //
      // We are at the # (node_index), traversing down to the @.
      // The node with values more than @ is $. Traversing to it requires knowlege of &
      // (parent_node_index).
      //
      return Some(self.index[self.index[parent_parent_node_index].more].less);
    }

    if self.index[parent_node_index].less == node_index {
      return Some(self.index[parent_node_index].more);
    }

    None
  }

  pub fn print_indexes(&self) {
    println!("== START INDEXES ==");
    let mut ct = 0;
    for index in &self.index {
      if index.timestamp.is_none() {
        println!("{}\tLEAF\t{:?} => parent:{}", ct, index.timestamp, index.parent);
        if let Some(ref data) = index.data {
          for item in data {
            println!("    - {:?} {:?}", item.timestamp, item.value);
          }
        } else {
          println!("    - None");
        }
      } else {
        println!("{}\tNODE\t{:?} => less:{} more:{} parent:{}", ct, index.timestamp, index.less, index.more, index.parent);
      }
      ct += 1;
    }
    println!("== END INDEXES ==");
  }
}


#[cfg(test)]
mod tests {
  use chrono::{Utc, TimeZone};
  use chart::chart::Chart;
  use chart::point::Point;

  #[test]
  fn it_gets_items_via_index() {
    let mut chart = Chart {
      max_index_node_capacity: 3,
      points: vec![
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 10, 0)),
        Point::new(6.0, Utc.ymd(2018, 1, 1).and_hms(9, 11, 0)),
        Point::new(7.0, Utc.ymd(2018, 1, 1).and_hms(9, 12, 0)),
        Point::new(8.0, Utc.ymd(2018, 1, 1).and_hms(9, 13, 0)),
        Point::new(9.0, Utc.ymd(2018, 1, 1).and_hms(9, 14, 0)),
        Point::new(1.0, Utc.ymd(2018, 1, 1).and_hms(9, 15, 0)),
        Point::new(2.0, Utc.ymd(2018, 1, 1).and_hms(9, 16, 0)),
        Point::new(3.0, Utc.ymd(2018, 1, 1).and_hms(9, 17, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 18, 0)),
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 19, 0)),
        Point::new(6.0, Utc.ymd(2018, 1, 1).and_hms(9, 20, 0)),
        Point::new(7.0, Utc.ymd(2018, 1, 1).and_hms(9, 21, 0)),
        Point::new(8.0, Utc.ymd(2018, 1, 1).and_hms(9, 22, 0)),
        Point::new(9.0, Utc.ymd(2018, 1, 1).and_hms(9, 23, 0)),
        Point::new(1.0, Utc.ymd(2018, 1, 1).and_hms(9, 24, 0)),
        Point::new(2.0, Utc.ymd(2018, 1, 1).and_hms(9, 25, 0)),
        Point::new(3.0, Utc.ymd(2018, 1, 1).and_hms(9, 26, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 27, 0)),
        Point::new(5.1, Utc.ymd(2018, 1, 1).and_hms(9, 28, 0)),
      ],
      index: vec![],
    };
    chart.build_index();

    // Exact datapoint somewhere in the middle
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 14, 0)), Some(9.0));

    // Interpolated somewhere in the middle (ie, not exactly on a datapoint)
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 11, 30)), Some(6.5));

    // First datapoint of whole dataset
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 10, 0)), Some(5.0));

    // Last datapoint of whole dataset
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 28, 0)), Some(5.1));

    /*
    11      LEAF    None => 9
        - 2018-01-01T09:20:00Z 6.0
        - 2018-01-01T09:21:00Z 7.0
        - 2018-01-01T09:22:00Z 8.0
    12      LEAF    None => 9
        - 2018-01-01T09:23:00Z 9.0
        - 2018-01-01T09:24:00Z 1.0
    13      LEAF    None => 10
        - 2018-01-01T09:25:00Z 2.0
        - 2018-01-01T09:26:00Z 3.0
    */

    // The below tests reference the above index layout:

    // In between two nodes
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 22, 15)), Some(8.25));

    // Right after the start of second index node
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 23, 15)), Some(7.0));

    // Right between the second and third index node
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 24, 15)), Some(1.25));

    // Get value in middle of an index node
    assert_eq!(chart.get_value(Utc.ymd(2018, 1, 1).and_hms(9, 20, 30)), Some(6.5));
  }

  #[test]
  fn it_gets_items_within_projection() {
    let mut chart = Chart {
      max_index_node_capacity: 3,
      points: vec![
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 10, 0)),
        Point::new(6.0, Utc.ymd(2018, 1, 1).and_hms(9, 11, 0)),
        Point::new(7.0, Utc.ymd(2018, 1, 1).and_hms(9, 12, 0)),
        Point::new(8.0, Utc.ymd(2018, 1, 1).and_hms(9, 13, 0)),
        Point::new(9.0, Utc.ymd(2018, 1, 1).and_hms(9, 14, 0)),
        Point::new(1.0, Utc.ymd(2018, 1, 1).and_hms(9, 15, 0)),
        Point::new(2.0, Utc.ymd(2018, 1, 1).and_hms(9, 16, 0)),
        Point::new(3.0, Utc.ymd(2018, 1, 1).and_hms(9, 17, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 18, 0)),
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 19, 0)),
        Point::new(6.0, Utc.ymd(2018, 1, 1).and_hms(9, 20, 0)),
        Point::new(7.0, Utc.ymd(2018, 1, 1).and_hms(9, 21, 0)),
        Point::new(8.0, Utc.ymd(2018, 1, 1).and_hms(9, 22, 0)),
        Point::new(9.0, Utc.ymd(2018, 1, 1).and_hms(9, 23, 0)),
        Point::new(1.0, Utc.ymd(2018, 1, 1).and_hms(9, 24, 0)),
        Point::new(2.0, Utc.ymd(2018, 1, 1).and_hms(9, 25, 0)),
        Point::new(3.0, Utc.ymd(2018, 1, 1).and_hms(9, 26, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 27, 0)),
        Point::new(5.1, Utc.ymd(2018, 1, 1).and_hms(9, 28, 0)),
      ],
      index: vec![],
    };
    chart.build_index();

    // Create a projection without operations
    let projection = chart.new_projection(
      Utc.ymd(2018, 1, 1).and_hms(9, 13, 0), /* start */
      Utc.ymd(2018, 1, 1).and_hms(9, 18, 0), /* end */
      vec![],
    );

    // Check datapoint within projection is correct
    let value = chart.get_value_projection(Utc.ymd(2018, 1, 1).and_hms(9, 14, 0), Some(&projection));
    assert_eq!(value, Some(9.0));

    // Check datapoint outside projection is None
    let value = chart.get_value_projection(Utc.ymd(2018, 1, 1).and_hms(9, 0, 0), Some(&projection));
    assert_eq!(value, None);
  }
}

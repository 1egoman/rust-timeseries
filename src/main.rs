extern crate chrono;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime};

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
struct Point {
  value: f64,
  timestamp: DateTime<Utc>,
}

impl Point {
  fn new(value: f64, timestamp: DateTime<Utc>) -> Point {
    Point {value: value, timestamp: timestamp}
  }
}

fn linear_interpolation(starting_value: f64, ending_value: f64, percentage: f64) -> f64 {
  starting_value + ((ending_value - starting_value) * percentage)
}


#[derive(Debug)]
#[derive(PartialEq)]
struct PointIndex {
  timestamp: Option<DateTime<Utc>>,
  less: usize,
  more: usize,
  data: Option<Vec<Point>>,
}

struct Chart {
  points: Vec<Point>,
  index: Vec<PointIndex>,

  max_index_node_capacity: usize,
}

impl Chart {
  fn get_value_vec(&self, timestamp: DateTime<Utc>) -> Option<f64> {
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
          return Some(point_before.value);
        }
        if percentage_between_points == 1.0 {
          return Some(point_after.value);
        }

        // Interpolate to find the actual value
        let result = linear_interpolation(
            point_before.value,
            point_after.value,
            percentage_between_points,
        );
        return Some(result);
      }
    }

    return None
  }

  fn rebalance_index_node(&mut self, node_index: usize) {
    println!("\n\nRebalancing {}", node_index);

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

            print!("Split timestamp: {:?}\n", timestamp_split_point);

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

            print!("Split index: {:?}\n", index_of_timestamp_split);

            less = Some(PointIndex {
              timestamp: None,
              less: 0,
              more: 0,
              data: Some(data[..index_of_timestamp_split].to_vec()),
            });

            more = Some(PointIndex {
              timestamp: None,
              less: 0,
              more: 0,
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
  fn build_index(&mut self) {
    // Start by making a single index for all items
    self.index.push(
      PointIndex {
        timestamp: Some(Utc::now()),
        less: 0,
        more: 0,
        data: Some(self.points.clone()),
      }
    );

    // Then, break up that node into smaller nodes
    self.rebalance_index_node(0);
  }

  // Find the location in the index that would contain a node with `timestamp`.
  fn lookup_in_index(&self, timestamp: DateTime<Utc>) -> Option<&PointIndex> {
    let mut node_index = 0;
    loop {
      match self.index[node_index].timestamp {
        Some(node_timestamp) => {
          // At a regular node. Figure out which leg to traverse down.
          if node_timestamp > timestamp {
            node_index = self.index[node_index].more;
          } else {
            node_index = self.index[node_index].less;
          }
          continue;
        }
        None => {
          // At a leaf.
          break
        }
      }
    }

    return Some(&self.index[node_index])
  }

  /* // Given the timestamp of a node, find the node that comes immediately before it. */
  /* fn get_value_before(&self, timestamp: DateTime<Utc>) -> Option<(Point, Point)> { */
  /*   println!("=== START === {}", timestamp); */
  /*   let mut node_index = 0; */
  /*   let mut parent_node_index = 0; */
  /*   let mut less_node_index = 0; */
  /*   let mut more_node_index = 0; */
  /*  */
  /*   // Find the node and the node that contains values before the current node. */
  /*   loop { */
  /*     match self.index[node_index].timestamp { */
  /*       Some(node_timestamp) => { */
  /*         // At a regular node. Figure out which leg to traverse down. */
  /*         if node_timestamp > timestamp { */
  /*           // */
  /*           //          & */
  /*           //    Less / \ More */
  /*           //        /   \ */
  /*           //       O     # */
  /*           //      / \   / \ */
  /*           //         $ @   \ */
  /*           //     end-^ ^-start */
  /*           // */
  /*           // We are at the # (node_index), traversing down to the @. */
  /*           // The node with values less than @ is $. Traversing to it requires knowlege of & */
  /*           // (parent_node_index). */
  /*           // */
  /*           less_node_index = self.index[self.index[parent_node_index].less].more; */
  /*           more_node_index = self.index[node_index].more; */
  /*  */
  /*           parent_node_index = node_index; */
  /*           node_index = self.index[node_index].less; */
  /*         } else { */
  /*           // If on the more side, the node with items less than the current node is the parent's */
  /*           // less node. */
  /*           // */
  /*           //             # */
  /*           //       Less / \ More */
  /*           //           $   @ */
  /*           // */
  /*           //  We are at the # (node_index), traversing down to the @. The node with values less */
  /*           //  than @ is $. */
  /*           // */
  /*           less_node_index = self.index[node_index].less; */
  /*           more_node_index = self.index[self.index[parent_node_index].more].less; */
  /*  */
  /*           parent_node_index = node_index; */
  /*           node_index = self.index[node_index].more; */
  /*         } */
  /*         continue; */
  /*       } */
  /*       None => { */
  /*         // At a leaf. */
  /*         break */
  /*       } */
  /*     } */
  /*   } */
  /*  */
  /*   /* let node_is_empty = self.index[node_index].data.len() == 0; */ */
  /*   /*  */ */
  /*   /*  */ */
  /*   /* let before_point: Point; */ */
  /*   /* let after_point: Point; */ */
  /*  */
  /*   if */
  /*  */
  /*   match self.index[node_index].data { */
  /*     Some(ref node_data) => { */
  /*       let ref node_data_last_datapoint = node_data[node_data.len()-1]; */
  /*  */
  /*       if timestamp > node_data_last_datapoint.timestamp { */
  /*         if let Some(ref more_node_data) = self.index[more_node_index].data { */
  /*           println!("RESULT: {:?} {:?}", */
  /*             node_data_last_datapoint, */
  /*             more_node_data[0], */
  /*           ); */
  /*         } */
  /*       } */
  /*     } */
  /*     None => () */
  /*   } */
  /*  */
  /*   /* if (timestamp > self.index[node_index].data[0].timestamp */ */
  /*   /* ) { */ */
  /*   /* } */ */
  /*  */
  /*   println!("NODE WITH ITEMS BEFORE {:?}", less_node_index); */
  /*   println!("THIS NODE {:?}", node_index); */
  /*   println!("NODE WITH ITEMS AFTER {:?}", more_node_index); */
  /*  */
  /*   return None */
  /* } */

  fn get_node_less_than(&self, parent_index: usize, node_index: usize) -> Option<usize> {
    if self.index[parent_index].less == node_index {
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
      return Some(self.index[self.index[parent_index].less].more);
    }

    if self.index[parent_index].more == node_index {
      return Some(self.index[node_index].less);
    }

    None
  }

  /* fn get_value_index(&self, timestamp: DateTime<Utc>) -> Option<f64> { */
  /*   match self.get_value_before(timestamp) { */
  /*     Some((ref point_before, ref point_after)) => { */
  /*       // Figure out the percentage between the points point before and the point after that */
  /*       // `timestamp` represents. */
  /*       let time_to_timestamp_ms = ( */
  /*         timestamp.timestamp_millis() - point_before.timestamp.timestamp_millis() */
  /*       ) as f64; */
  /*       let time_between_points_ms = ( */
  /*         point_after.timestamp.timestamp_millis() - point_before.timestamp.timestamp_millis() */
  /*       ) as f64; */
  /*       let percentage_between_points = time_to_timestamp_ms / time_between_points_ms; */
  /*  */
  /*       // Don't interpolate if not required */
  /*       if percentage_between_points == 0.0 { */
  /*         return Some(point_before.value); */
  /*       } */
  /*       if percentage_between_points == 1.0 { */
  /*         return Some(point_after.value); */
  /*       } */
  /*  */
  /*       // Interpolate to find the actual value */
  /*       let result = linear_interpolation( */
  /*         point_before.value, */
  /*         point_after.value, */
  /*         percentage_between_points, */
  /*       ); */
  /*       return Some(result); */
  /*     }, */
  /*     _ => None, */
  /*   } */
  /* } */
}

fn main() {
    let mut chart = Chart {
      max_index_node_capacity: 3,
      points: vec![
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 10, 0)),
        Point::new(6.0, Utc.ymd(2018, 1, 1).and_hms(9, 11, 0)),
        Point::new(7.0, Utc.ymd(2018, 1, 1).and_hms(9, 12, 0)),
        Point::new(6.0, Utc.ymd(2018, 1, 1).and_hms(9, 13, 0)),
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 14, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 15, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 16, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 17, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 18, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 19, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 20, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 21, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 22, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 23, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 24, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 25, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 26, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 27, 0)),
        Point::new(4.0, Utc.ymd(2018, 1, 1).and_hms(9, 28, 0)),
      ],
      index: vec![],
    };

    chart.build_index();

    /* println!("Point at 9:11:00: {}", chart.get_value_vec(Utc.ymd(2018, 1, 1).and_hms(9, 11, 00)).unwrap()); */
    /* println!("Point at 9:11:30: {}", chart.get_value_vec(Utc.ymd(2018, 1, 1).and_hms(9, 11, 30)).unwrap()); */

    /* println!("Get from index for 9:11:30: {:?}", chart.lookup_in_index(Utc.ymd(2018, 1, 1).and_hms(9, 11, 30))); */
    /* println!("Get value at 9:12:15: {:?}", chart.get_value_index(Utc.ymd(2018, 1, 1).and_hms(9, 12, 15))); */

    {
      println!("");
      println!("");
      println!("INDEXES:");
      let mut ct = 0;
      for index in chart.index {
        if index.timestamp.is_none() {
          println!("{}\tLEAF\t{:?}", ct, index.timestamp);
          for item in index.data.unwrap() {
            println!("    - {:?} {:?}", item.timestamp, item.value);
          }
        } else {
          println!("{}\tNODE\t{:?} => less:{} more:{}", ct, index.timestamp, index.less, index.more);
        }
        ct += 1;
      }
    }
}

extern crate chrono;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime};

use chart::chart::Chart;
use chart::point_index::PointIndex;

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

  // Get the value at a specified timestamp using the index
  pub fn get_value_index(&self, timestamp: DateTime<Utc>) -> Option<f64> {
    if let Some(node_index) = self.lookup_in_index(Utc.ymd(2018, 1, 1).and_hms(9, 13, 30)) {
      println!("Get from index for 9:13:30: {:?}", node_index);

      if let Some(ref node_data) = self.index[node_index].data {
        if timestamp < node_data[0].timestamp {
          println!("Need to get LESS side!");

          if let Some(node_less) = self.get_node_less_than(node_index) {
            if let Some(ref node_less_data) = self.index[node_less].data {
              let smaller_value = &node_less_data[node_less_data.len()-1];
              let larger_value = &node_data[0];
              return Some(self.interpolate_between_points(timestamp, smaller_value, larger_value));
            } else {
              None
            }
          } else {
            None
          }
        } else if timestamp > node_data[node_data.len()-1].timestamp {
          println!("Need to get MORE side!");

          let node_more_wrapped = self.get_node_more_than(node_index);
          println!("NODE GREATER THAN CURRENT NODE: {:?}", node_more_wrapped);

          if let Some(node_more) = self.get_node_more_than(node_index) {
            if let Some(ref node_more_data) = self.index[node_more].data {
              let smaller_value = &node_data[node_data.len()-1];
              let larger_value = &node_more_data[0];
              return Some(self.interpolate_between_points(timestamp, smaller_value, larger_value));
            } else {
              None
            }
          } else {
            None
          }
        } else {
          println!("In bounds");

          for point_index in 1..node_data.len()-1 {
            println!("Is {:?} < {:?}", timestamp, node_data[point_index].timestamp);
            if timestamp < node_data[point_index].timestamp {
              let smaller_value = &node_data[point_index];
              let larger_value = &node_data[point_index+1];
              return Some(self.interpolate_between_points(timestamp, smaller_value, larger_value));
            }
          }

          // For checking the last item, the logic is slightly different.
          if timestamp < node_data[node_data.len()-1].timestamp {
            let smaller_value = &node_data[node_data.len()-2];
            let larger_value = &node_data[node_data.len()-1];
            return Some(self.interpolate_between_points(timestamp, smaller_value, larger_value));
          }

          None
        }
      } else {
        None
      }
    } else {
      None
    }
  }

  // Find the location in the index that would contain a node with `timestamp`.
  pub fn lookup_in_index(&self, timestamp: DateTime<Utc>) -> Option<usize> {
    let mut node_index = 0;
    loop {
      match self.index[node_index].timestamp {
        Some(node_timestamp) => {
          // At a regular node. Figure out which leg to traverse down.
          if node_timestamp > timestamp {
            node_index = self.index[node_index].less;
          } else {
            node_index = self.index[node_index].more;
          }
          continue;
        }
        None => {
          // At a leaf.
          break
        }
      }
    }

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
        println!("{}\tLEAF\t{:?} => {}", ct, index.timestamp, index.parent);
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


extern crate chrono;
use chrono::{DateTime, Utc};

use chart::chart::Chart;
use chart::point_index::PointIndex;

pub trait ChartProjectionOperation {
  // Apply is called for each point index node to determine how it should be modified in the
  // projection.
  // 
  // Three cases:
  // Return a Some(index_node), which means keep the point index
  // Return a Some(something_else), which means replace the point index with something_else
  // Return None, which means don't include the point index
  fn apply(&self, index_node: &PointIndex) -> Option<&PointIndex>;
}

pub struct ChartProjection<'a> {
  pub chart: &'a Chart,
  pub operations: Vec<Box<ChartProjectionOperation> /* box ensures that each has the same size */>,

  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
  pub default_value: PointIndex,
}

impl Chart {
  pub fn new_projection(
    &self,
    start_time: DateTime<Utc>, end_time: DateTime<Utc>,
    operations: Vec<Box<ChartProjectionOperation>>,
  ) -> ChartProjection {
    ChartProjection {
      chart: &self,
      operations: operations,

      // Projection parameters
      start_time: start_time,
      end_time: end_time,
      default_value: PointIndex::new_default_value(),
    }
  }

  pub fn project_index_node<'a>(
    &'a self,
    node_index: usize,
    projection: Option<&'a ChartProjection>,
  ) -> &'a PointIndex {
    debug!(
      "CALLING chart.project_index_node({}, {})",
      node_index,
      if projection.is_some() { "<projection>" } else { "None" }
    );

    match projection {
      // No projection, perform a mapping and that's it
      None => &self.index[node_index],

      // Project the index node.
      Some(ref projection) => {
        let mut accumulator = &self.index[node_index];

        if let Some(ref data) = accumulator.data {
          if data.len() == 0 {
            debug!("No data in node to project, so just returning node.");
            // No data, so it's not required to copy the node since no items exist to filter anyway
            accumulator
          } else {
            // If outside of the range the projection applies to, then disregard.
            if data[0].timestamp < projection.start_time || data[data.len()-1].timestamp > projection.end_time {
              debug!(
                "Node index {} out of projection range (timestamp:{:?} start_time:{} end_time{}), returning default",
                node_index, accumulator.timestamp, projection.start_time, projection.end_time,
              );
              return &projection.default_value;
            }

            // Apply projection operations
            for operation in &projection.operations {
              match operation.apply(accumulator) {
                Some(result) => {
                  accumulator = result;
                }
                None => {
                  return &projection.default_value;
                }
              }
            }

            accumulator
          }
        } else {
          accumulator
        }
      }
    }
  }
}

/*
  ChartProjection {
    operations: vec![
      ChartProjectionOperationFilterTimestamp.new(start, end),
      ChartProjectionOperationFilter.new(|point| point.value > 5),
      ChartProjectionOperationParelelReduce.new(|acc, point| acc + point.value, 0),
    ],
  }
*/


/* pub struct ChartProjectionOperationFilter { */
/*   predicate: fn(point: &Point) -> bool, */
/* } */
/* impl ChartProjectionOperationFilter { */
/*   fn new(predicate: fn(point: &Point) -> bool) -> ChartProjectionOperationFilter { */
/*     ChartProjectionOperationFilter { predicate: predicate } */
/*   } */
/* } */
/* impl ChartProjectionOperation for ChartProjectionOperationFilter { */
/*   fn apply(&self, index_node: PointIndex) -> Option<PointIndex> { */
/*     match index_node.data { */
/*       Some(ref data) => { */
/*         let filter_results = data.iter().filter(self.predicate).collect(); */
/*  */
/*         if filter_results.len() == data.len() { */
/*           /* no items were removed, keep the existing node */ */
/*           Some(index_node) */
/*         } else if filter_results.len() == 0 { */
/*           /* remove the node, since no items in it matched the filter expression */ */
/*           None */
/*         } else { */
/*           /* return a modified copy of the node with filter_results as data */ */
/*           let nd = index_node.clone(); */
/*           nd.data = filter_results; */
/*           Some(nd) */
/*         } */
/*       } */
/*  */
/*       /* keep the node by default, since it's a non-leaf node */ */
/*       _ => Some(index_node) */
/*     } */
/*   } */
/* } */

// ChartProjectionOperationFilter.new(|point| p.value > 5)


extern crate chrono;
use chrono::{DateTime, Utc};

use chart::chart::Chart;
use chart::point::Point;
use chart::point_index::PointIndex;

/* pub trait ChartProjectionOperation<'a> { */
/*   // Apply is called for each point index node to determine how it should be modified in the */
/*   // projection. */
/*   //  */
/*   // Three cases: */
/*   // Return a Some(index_node), which means keep the point index */
/*   // Return a Some(something_else), which means replace the point index with something_else */
/*   // Return None, which means don't include the point index */
/*   fn apply(&self, index_node: &'a PointIndex) -> Option<&PointIndex>; */
/* } */


pub struct ProjectionDisposable {
    nodes: Vec<PointIndex>,
}

impl ProjectionDisposable {
    fn new() -> ProjectionDisposable {
        ProjectionDisposable {
            nodes: vec![],
        }
    }
    
    fn add(&mut self, point: PointIndex) -> usize {
        self.nodes.push(point);
        self.nodes.len()-1
    }
    
    fn get(&self, index: usize) -> &PointIndex {
        &self.nodes[index]
    }
}


pub struct ProjectionOperationResult {
  pub action: &'static str,
  pub index: Option<usize>,
}

impl ProjectionOperationResult {
  fn keep() -> ProjectionOperationResult {
    ProjectionOperationResult { action: "KEEP", index: None }
  }
  fn replace(index: usize) -> ProjectionOperationResult {
    ProjectionOperationResult { action: "REPLACE", index: Some(index) }
  }
  fn delete() -> ProjectionOperationResult {
    ProjectionOperationResult { action: "DELETE", index: None }
  }
}


pub struct ProjectionOperationNode<'a> {
  node: Option<&'a PointIndex>,
  index: usize,
}
impl<'a> ProjectionOperationNode<'a> {
  fn new(point_index: &'a PointIndex) -> ProjectionOperationNode {
    ProjectionOperationNode {
      node: Some(point_index),
      index: 0,
    }
  }

  fn update_index(&mut self, index: usize) {
    self.node = None;
    self.index = index;
  }

  fn value(&'a self, disposable: &'a ProjectionDisposable) -> &PointIndex {
    match self.node {
      Some(ref node) => node,
      None => disposable.get(self.index),
    }
  }
}




pub struct ProjectionOperation {
  predicate: fn(&Point) -> Point,
}
impl ProjectionOperation {
  fn apply(
    &self,
    node: &PointIndex,
    projection: &Projection,
    disposable: &mut ProjectionDisposable,
  ) -> ProjectionOperationResult {
    let mut results: Vec<Point> = vec![];
    let mut output_identical_to_input = true;

    if let Some(ref data) = node.data {
      // Map each point in `data` into `results`.
      for input_point in data {
        let output_point = (self.predicate)(&input_point);
        if input_point != &output_point {
          output_identical_to_input = false;
        }
        results.push(output_point);
      }

      if output_identical_to_input {
        // Return the original index node, since the mapping operation did nothing to any items in
        // it.
        ProjectionOperationResult::keep()
      } else {
        // Replace the index node with the updates from the mapping operation.
        let pt = node.clone();
        ProjectionOperationResult::replace(disposable.add(pt))
      }
    } else {
      // No data in the node? It's not a leaf, and map can't do anything with it.
      ProjectionOperationResult::keep()
    }
  }
}


pub struct Projection {
  pub operations: Vec<Box<ProjectionOperation> /* box ensures that each has the same size */>,

  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
  pub default_value: PointIndex,
}

impl Projection {
  fn new(
    start_time: DateTime<Utc>, end_time: DateTime<Utc>,
    operations: Vec<Box<ProjectionOperation>>,
  ) -> Projection {
    Projection {
      operations: operations,

      start_time: start_time,
      end_time: end_time,
      default_value: PointIndex::new_default_value(),
    }
  }
}

impl Chart {
  pub fn project_index_node<'a>(
    &'a self,
    node_index: usize,
    mut projection_disposable: Option<(&'a Projection, &'a mut ProjectionDisposable)>,
  ) -> (&'a PointIndex, Option<(&'a Projection, &'a mut ProjectionDisposable)>) {
    debug!(
      "CALLING chart.project_index_node({}, {})",
      node_index,
      if projection_disposable.is_some() { "<projection>" } else { "None" }
    );

    let mut accumulator = &self.index[node_index];

    match projection_disposable {
      // No projection, perform a mapping and that's it
      None => (),

      // Project the index node.
      Some((ref projection, ref mut disposable)) => {
        if let Some(ref data) = self.index[node_index].data {

          if data.len() == 0 {
            debug!("No data in node to project, so just returning node.");
            // No data, so it's not required to copy the node since no items exist to filter anyway
            return (accumulator, None)
          } else {
            // If outside of the range the projection applies to, then disregard.
            if data[0].timestamp < projection.start_time || data[data.len()-1].timestamp > projection.end_time {
              /* debug!( */
              /*   "Node index {} out of projection range (timestamp:{:?} start_time:{} end_time{}), returning default", */
              /*   node_index, accumulator.timestamp, projection.start_time, projection.end_time, */
              /* ); */
              return (&projection.default_value, None);
            }

            // Apply projection operations
            for operation in &projection.operations {
              match operation.apply(accumulator, projection, disposable) {
                ProjectionOperationResult { action: "KEEP", index: _ } => (),
                ProjectionOperationResult { action: "REPLACE", index: Some(index)} => {
                  /* FIXME FIXME FIXME */
                  accumulator = accumulator;
                },
                ProjectionOperationResult { action: "DELETE", index: _ } => {
                  return (&projection.default_value, None);
                },
                _ => (),
              }
            }
          }
        } else {
          return (&self.index[node_index], None)
        }
      }
    }

    // By default, return the accumulator and the projection/disposable combo
    (accumulator, projection_disposable)
  }
}

#[macro_use]
extern crate log;
extern crate simple_logger;

extern crate chrono;
use chrono::{Utc, TimeZone};

mod chart;
use chart::chart::Chart;
use chart::point::Point;

use std::time::Instant;

fn main() {
    simple_logger::init().unwrap();

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
        Point::new(5.0, Utc.ymd(2018, 1, 1).and_hms(9, 28, 0)),
      ],
      index: vec![],
    };



    chart.build_index();

    let projection = chart.new_projection(
      Utc.ymd(2018, 1, 1).and_hms(9, 13, 0), /* start */
      Utc.ymd(2018, 1, 1).and_hms(9, 18, 0), /* end */
      vec![],
    );


    let timestamp = Utc.ymd(2018, 1, 1).and_hms(9, 12, 30);

    println!("{:?}", chart.get_value_projection(timestamp, Some(&projection)));



    chart.print_indexes();


    /* // The vector */
    /* { */
    /*   let now = Instant::now(); */
    /*  */
    /*   for _ in 0..1000 { */
    /*     chart.get_value_vec(timestamp); */
    /*   } */
    /*  */
    /*   let elapsed = now.elapsed(); */
    /*   let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0); */
    /*   println!("Vector Seconds: {}", sec / 1000.0); */
    /* } */
    /*  */
    /* // The index */
    /* { */
    /*   let now = Instant::now(); */
    /*  */
    /*   for _ in 0..1000 { */
    /*     chart.get_value(timestamp); */
    /*   } */
    /*  */
    /*   let elapsed = now.elapsed(); */
    /*   let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0); */
    /*   println!("Index Seconds:  {}", sec / 1000.0); */
    /* } */
}

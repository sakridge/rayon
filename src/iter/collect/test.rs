#![cfg(test)]
#![allow(unused_assignments)]

// These tests are primarily targeting "abusive" producers that will
// try to drive the "collect consumer" incorrectly. These should
// result in panics.

use super::Collect;
use iter::plumbing::*;
use solana_rayon_core::join;

/// Promises to produce 2 items, but then produces 3.  Does not do any
/// splits at all.
#[test]
#[should_panic(expected = "too many values")]
fn produce_too_many_items() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 2);
    let consumer = collect.as_consumer();
    let mut folder = consumer.into_folder();
    folder = folder.consume(22);
    folder = folder.consume(23);
    folder.consume(24);
}

/// Produces fewer items than promised. Does not do any
/// splits at all.
#[test]
#[should_panic(expected = "expected 5 total writes, but got 2")]
fn produce_fewer_items() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 5);
    {
        let consumer = collect.as_consumer();
        let mut folder = consumer.into_folder();
        folder = folder.consume(22);
        folder = folder.consume(23);
        folder.complete();
    }
    collect.complete();
}

// Complete is not called by the consumer.Hence,the collection vector is not fully initialized.
#[test]
#[should_panic(expected = "expected 4 total writes, but got 2")]
fn left_produces_items_with_no_complete() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        let mut left_folder = left_consumer.into_folder();
        let mut right_folder = right_consumer.into_folder();
        left_folder = left_folder.consume(0).consume(1);
        right_folder = right_folder.consume(2).consume(3);
        right_folder.complete();
    }
    collect.complete();
}

// Complete is not called by the right consumer. Hence,the
// collection vector is not fully initialized.
#[test]
#[should_panic(expected = "expected 4 total writes, but got 2")]
fn right_produces_items_with_no_complete() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        let mut left_folder = left_consumer.into_folder();
        let mut right_folder = right_consumer.into_folder();
        left_folder = left_folder.consume(0).consume(1);
        right_folder = right_folder.consume(2).consume(3);
        left_folder.complete();
    }
    collect.complete();
}

// Complete is not called by the consumer. Hence,the collection vector is not fully initialized.
#[test]
#[should_panic(expected = "expected 2 total writes, but got 0")]
fn produces_items_with_no_complete() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 2);
    {
        let consumer = collect.as_consumer();
        let mut folder = consumer.into_folder();
        folder = folder.consume(22);
        folder = folder.consume(23);
    }
    collect.complete();
}

// The left consumer produces too many items while the right
// consumer produces correct number.
#[test]
#[should_panic(expected = "too many values")]
fn left_produces_too_many_items() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        let mut left_folder = left_consumer.into_folder();
        let mut right_folder = right_consumer.into_folder();
        left_folder = left_folder.consume(0).consume(1).consume(2);
        right_folder = right_folder.consume(2).consume(3);
        right_folder.complete();
    }
    collect.complete();
}

// The right consumer produces too many items while the left
// consumer produces correct number.
#[test]
#[should_panic(expected = "too many values")]
fn right_produces_too_many_items() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        let mut left_folder = left_consumer.into_folder();
        let mut right_folder = right_consumer.into_folder();
        left_folder = left_folder.consume(0).consume(1);
        right_folder = right_folder.consume(2).consume(3).consume(4);
        left_folder.complete();
    }
    collect.complete();
}

// The left consumer produces fewer items while the right
// consumer produces correct number.
#[test]
#[should_panic(expected = "expected 4 total writes, but got 3")]
fn left_produces_fewer_items() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        let mut left_folder = left_consumer.into_folder();
        let mut right_folder = right_consumer.into_folder();
        left_folder = left_folder.consume(0);
        right_folder = right_folder.consume(2).consume(3);
        left_folder.complete();
        right_folder.complete();
    }
    collect.complete();
}

// The right consumer produces fewer items while the left
// consumer produces correct number.
#[test]
#[should_panic(expected = "expected 4 total writes, but got 3")]
fn right_produces_fewer_items() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        let mut left_folder = left_consumer.into_folder();
        let mut right_folder = right_consumer.into_folder();
        left_folder = left_folder.consume(0).consume(1);
        right_folder = right_folder.consume(2);
        left_folder.complete();
        right_folder.complete();
    }
    collect.complete();
}

// The left consumer panics and the right stops short, like `panic_fuse()`.
// We should get the left panic without ever reaching `Collect::complete()`.
#[test]
#[should_panic(expected = "left consumer panic")]
fn left_panics() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        join(
            || {
                let mut left_folder = left_consumer.into_folder();
                left_folder = left_folder.consume(0);
                panic!("left consumer panic");
            },
            || {
                let mut right_folder = right_consumer.into_folder();
                right_folder = right_folder.consume(2);
                right_folder.complete() // early return
            },
        );
    }
    collect.complete();
}

// The right consumer panics and the left stops short, like `panic_fuse()`.
// We should get the right panic without ever reaching `Collect::complete()`.
#[test]
#[should_panic(expected = "right consumer panic")]
fn right_panics() {
    let mut v = vec![];
    let mut collect = Collect::new(&mut v, 4);
    {
        let consumer = collect.as_consumer();
        let (left_consumer, right_consumer, _) = consumer.split_at(2);
        join(
            || {
                let mut left_folder = left_consumer.into_folder();
                left_folder = left_folder.consume(0);
                left_folder.complete() // early return
            },
            || {
                let mut right_folder = right_consumer.into_folder();
                right_folder = right_folder.consume(2);
                panic!("right consumer panic");
            },
        );
    }
    collect.complete();
}

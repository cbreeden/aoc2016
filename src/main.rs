#![feature(test)]
extern crate test;
extern crate rayon;
extern crate fnv;

mod solutions;

fn main() {
    solutions::day1::run();
}
#![feature(test)]
extern crate test;

extern crate rayon;

mod solutions;

fn main() {
    solutions::day1::run();
}
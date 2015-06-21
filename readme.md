# A*

This is an implementation of the A* search algorithm in Rust.
It is meant to be very general, and it should be possible to use this A* library
for any A* search problem.

[Api Docs](http://tyoverby.com/astar/astar/)

### Requirements

In your search problem, each "node" must implement `Hash` and `PartialEq`

The distance and cost associated with the problem must implement `PartialOrd`, `Zero`, and `Clone`.

### Usage

In order to use the A* algorithm, you must frame your problem in a specific way.
This is done by implementing the SearchProblem trait.

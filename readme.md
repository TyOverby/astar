# A*

This is an implementation of the A* search algorithm in Rust.
It is meant to be very general, and it should be possible to use this A* library
for any A* search problem.

### Requirements

In your search problem, each node must have `Hash` and `PartialEq` defined on
them.  The distance and cost associated with the problem must implement
`PartialOrd`, `Zero`, and `Clone`.

### Usage

In order to use the A* algorithm, you must frame your problem in a specific way.
This is done by implementing the SearchProblem trait.  The full trait is given
here for clarity:

```rust
pub trait SearchProblem<N, C, I: Iterator<(N, C)>> {
    fn start(&self) -> N;
    fn is_end(&self, &N) -> bool;
    fn heuristic(&self, &N) -> C;
    fn neighbors(&self, at: &N) -> I;
    fn estimate_length(&self) -> Option<uint> { None }
}
```

Once you have an object that implements `SearchProblem`, you can perform a
search like this:

```rust
    let path = astar(my_search_problem);
```


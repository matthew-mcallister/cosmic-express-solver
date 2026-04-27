# Cosmic Express Solver

Solves levels for the puzzle game Cosmic Express. Uses depth-first
search with backtracking and reachability queries to prune unsolvable
states. It's pretty fast.

I wrote this in a short amount of time to solve one level in the first
constellation, so it's far from complete.

## Example

Input:
```
...........
..........x
..o.B.O....
........b..
..b..B.....
........b..
..o.B.O....
..........e
...........
```

Output:
```
.9012345678
.8.6543298x
.7/7/./1076
.6.8901./.5
.5/98/23..4
.4.07654/.3
.3/1/./...2
.212345...1
..09876....
```

## Legend

| Character | Meaning           |
| --------- | --------          |
| .         | empty             |
| /         | obstacle          |
| e         | entrance          |
| x         | exit              |
| b         | blue passenger    |
| B         | blue house        |
| o         | orange passenger  |
| O         | orange house      |
| 0-9       | Rails. Numbered 1-100 but only the final digit is printed. |

# Specification

## Entrypoint

Every turn the `step` function is called.

You get a `Game` object as an argument containing the following fields:

- `width`: `int` - width of the game map
- `height`: `int` - height of the game map
- `snake`: `Snake` - your snake object
- `opponents`: `Vec<Snake>` - list of enemy snake objects
- `apples`: `Vec<Point>` - currently present apples on the map

The `step` function returns a `Direction` enum which the snake will be moving towards during the turn.

#### Example

```
fn step(game: Game) -> Direction {
    return game.snake.direction.rot_clockwise();
}
```

## Direction (enum)

- `UP`
- `DOWN`
- `LEFT`
- `RIGHT`

### Methods

- `opposite() -> Direction` - return the opposite direction (UP -> DOWN)
- `rot_clockwise() -> Direction` - rotate the current direction one step clockwise (UP -> RIGHT)
- `rot_anti_clockwise() -> Direction` - rotate the current direction counter clowksie (UP -> LEFT)
- `to_string() -> String` - return string representation of the Direction

## Point (struct)

- `x`: `int`
- `y`: `int`

## Snake (struct)

- `points`: `Vec<Point>` - list of points of the snake (first is head, last is tail)
- `direction`: `Direction` - the direction the snake is currently facing

### Methods

- `head() -> Direction` - returns head of the snake (first element in `points`)
- `tail() -> Direction` - returns tail of the snake (last element in `points`)
- `size() -> int` - the size of the snake (same as `points.size()`)

# Scripting

## Overview

The Legion engine supports several scripting languages:

- [Mun](https://mun-lang.org/)
- [Rhai](https://rhai.rs/)
- [Rune](https://rune-rs.github.io/)

The examples in this chapter will use the Rune programming language for script examples.

## ScriptComponent

You enable scripting on an entity by attaching a [`ScriptComponent`](https://api.legionengine.com/lgn_scripting/runtime/struct.ScriptComponent.html) to it.

This component must refer to an external [`Script`](https://api.legionengine.com/lgn_scripting/runtime/struct.Script.html), containing the compiled source code.

Each script must have an entry-point function.

## Inputs

A `ScriptComponent` also has an associated list of input values, in the form of strings.

These can be used to pass in literal arguments, such as strings or numerical values.

However, in addition to literals, the scripts can use several inputs that provide special behavior.

### The "{entity}" input

Use an input with the value `"{entity}"` to have access to the entity to which the ScriptComponent is attached.

```rust
pub fn drift(entity) {
    if let Some(transform) = entity.transform {
        transform.translation.x -= 0.05;
    }
}
```

The passed-in argument (`entity` in this example), has several accessible fields, one per component type. Each field returns an Option of the corresponding component type. In this example, the `transform` field returns an `Option<&mut Transform>`.

### The "{entities}" input

Use an input with the value `"{entities}"` to get a lookup map that can retrieve entities by name.

```rust
pub fn print_paddles(entities) {
    if let Some(entity) = entities["Paddle Left"] {
        if let Some(transform) = entity.transform {
            println!("paddle left: {}", transform.translation.y);
        }
    }
    if let Some(entity) = entities["Paddle Right"] {
        if let Some(transform) = entity.transform {
            println!("paddle right: {}", transform.translation.y);
        }
    }
}
```

### The "{events}" input

Some input events are cached and accessible through an input name `"{events}"`.

```rust
pub fn print_mouse_cursor(events) {
    println!("cursor: {}", events.mouse_motion);
```

### The "{result}" input

The main script function, i.e. the entry-point, returns a result. The result will be of unit-type (`()`) by default.

The result from the previous invocation is always cached, and can be accessed using an input with the value `"{result}"`.

> The initial value of the cached result is the unit value.

This can be very useful as a quick way to implement some persistent state for the script.

```rust
pub fn update(last_result) {
    let speed = if last_result is unit {
        // initial value
        5.0
    } else {
        last_result
    };

    speed -= 0.01;
    speed
}
```

# Equator
Simple equation viewer in Rust.

## Instructions

Type with your keyboard or press the buttons in the app to insert the mathematical constructs into the equation. The right hand side is automatically updated with the evaluation of the expression.

Variables can be assigned to by pressing the STORE key and pressing a button in the app or a variable on your keyboard. The variable's value will be set to the right hand side of the equation. Constants such as pi (π), e and the golden ratio (φ) cannot be assigned to.

## Todo:
- [ ] Add more functions -- log

- [ ] MAJOR - Add clickable equation - selects a position in the expression
- [ ] MAJOR - Change number system to base 10. This allows for more accurate calculations, as base 10 is used more in everyday life
- [ ] MAJOR - Add 'infinite precision' numbers - surds, fractions, irrational constants and coefficients of these
- [ ] MAJOR - Add multiple types of numbers -- complex, matrices, etc.

### Mouse-clickable equations

This will need functions called `push_handled_translation_index` and `handle_translations` in the render.rs file with the following signatures.

```rust
fn push_handled_translation_index();
fn handle_translations(x: f64, y: f64);
```

`push_handled_translation_index` will push the current index of the selection boxes stack, and whether the cursor's box has been set yet. `handle_translations` will pop the index and apply a translation to all of the selection boxes after that. It will also translate the cursor's box if it was not set at the corresponding push and it is when the function is called.

`push_handled_translation_index` will be called just before recursing in path_expr. `handle_translations` will be called just after this function exits, and the translation has been calculated.

The vector of selection boxes will contain references to the expression and cursor position it refers to. It will be used when the user clicks the main `DrawingArea` to figure out where to place the cursor.

As this is done during the rendering stage, there will be no mismatch in the shown expression and the selection box vector.

## To fix:

![bug](http://i.imgur.com/SQbD2wu.png)

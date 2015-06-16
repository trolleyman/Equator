# Equator
Simple equation viewer in Rust.

## Todo:
- [ ] Get SHIFT + CTRL buttons to activate on Shift + Ctrl keypresses.
- [ ] Add fractions
- [ ] Add more functions -- ln, log, hyperbolics, |x|, factorials
- [ ] Add constants -- e, pi

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

### More accurate number system

```rust
pub enum Number {
	Float(f64), // Normal
	Scientific(i64, i8), // Num = i64 * 10^i8. i64 max will be 999,999,999,999,999,999. (i64 max is 9223372036854775807).
}
```

## To fix:
This bug

![bug](http://i.imgur.com/SQbD2wu.png)

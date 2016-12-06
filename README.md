# Equator
Simple equation viewer/editor written in Rust.

![](/example1.png)

## Instructions

Type with your keyboard or press the buttons in the app to insert the mathematical constructs into the equation. The right hand side is automatically updated with the evaluation of the expression. You can navigate the equation by clicking where you want the cursor, or alternatively you can use the arrow keys.

Variables can be assigned to by pressing the STORE key and pressing a button in the app or a variable on your keyboard. The variable's value will be set to the right hand side of the equation. Constants such as pi (π), e and the golden ratio (φ) cannot be assigned to.

### Keys

Key | Description
--- | ------------------
Up, Down, Left, Right | Navigate throughout the equation
Delete/Backspace | Remove parts of the equation
F1 | Toggle debug printing of expression lexing (1st stage)
F2 | Toggle debug printing of conversion of tokens to commands (2nd stage)
F3 | Toggle debug printing of calculation (3rd stage)
F4 | Toggle debug view

## Todo:
Status | Task
------ | -------------
Todo | Implement trig functions, and ln.
Todo | Add grapher that can display the equation entered.
Todo | Add equation 'history' that can be selected.
Todo | Add 'infinite precision' numbers - surds, fractions, irrational constants and coefficients of these
Todo | Add multiple types of numbers -- complex, matrices, etc.

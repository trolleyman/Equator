
// TODO: Change ParenOpen => LeftParen && ParenClose => RightParen

fn expr_to_infix(infix: &[Command]) -> Result<Vec<Command>, ParseError> {
let postfix = Vec::new();
let stack = Vec::new();

for tok in infix {
	match tok {
		Var(_) | Int(_) => postfix.push(tok),
		Func(_) => stack.push(tok),
		Comma => {
			loop {
				let pop = match stack.pop() {
					Some(ParenOpen) => stack.push(ParenOpen),
					Some(pop) => postfix.push(pop),
					None => return Err(UnmatchedParen),
				}
			}
		},
		_ if tok.is_operator() {
			loop {
				let peek = stack.get(stack.len() - 1);
				if peek.is_operator() {
					if (tok.is_left_associative() && tok.prescedence() <= peek.prescedence())
						|| (tok.is_right_associative() && tok.prescedence() > peek.prescedence()) {
						stack.pop();
						postfix.push(peek);
					} else {
						break;
					}
				} else {
					break;
				}
			}
			postfix.push(tok);
		},
		ParenOpen => stack.push(ParenOpen),
		ParenClose => {
			// Push all tokens from stack to output until left parenthesis
			loop {
				let pop = match stack.pop() {
					None => return Err(UnmatchedParenthesis),
					Some(ParenOpen) => break,
					Some(v) => postfix.push(pop),
				};
			}
		},
		_ => return Err(IllegalCharacter)
	}
}
while stack.len() > 0 {
	let pop = stack.pop();
	if pop == ParenOpen || pop == ParenClose {
		return Err(MismatchedParentheses);
	}
	postfix.push(pop);
}

Ok(postfix)
}


// 
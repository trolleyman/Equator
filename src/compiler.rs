
use com::*;

struct Compiler<'a> {
	cache: Option<Result<Vec<Command>, Vec<VError>>>,
}

impl<'a> Compiler<'a> {
	pub fn new() -> Compiler {
		Compiler { cache: None }
	}
	pub fn compile(&self, ex: VExpr) -> Result<&'a [Command], &'a [VError]> {
		
	}
}


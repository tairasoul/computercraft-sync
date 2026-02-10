use darklua_core::{Resources, nodes::{Arguments, BinaryExpression, BinaryOperator, Expression, FunctionCall, Identifier, Prefix, StringExpression}, process::{DefaultVisitor, NodeProcessor, NodeVisitor}, rules::{Context, FlawlessRule, Rule, RuleConfiguration, RuleProperties}};

#[derive(Debug)]
pub struct PrefixRequireRule {
	prefix: String,
	exceptions: Vec<String>,
}

struct Prefixer {
	prefix: String,
	exceptions: Vec<String>
}

impl PrefixRequireRule {
	pub fn new(prefix: String, exceptions: Vec<String>) -> Self {
		PrefixRequireRule { prefix, exceptions }
	}
}

impl Prefixer {
	fn replace_with(&self, expression: &Expression) -> Option<Expression> {
		match expression {
			Expression::Call(fc) => {
				let name = fc.get_prefix();
				if let Prefix::Identifier(id) = name {
					if id.get_name() == "require" {
						let args = fc.get_arguments();
						let exprs = args.clone().to_expressions();
						let mut new_args = Arguments::default();
						for i in 0..args.len() {
							if i == 0 {
								let expr = exprs.get(i).unwrap().clone();
								if let Expression::String(strexpr) = expr {
									let str = strexpr.get_string_value().unwrap();
									let combined = "\"".to_owned() + &self.prefix + str + "\"";
									let new_expr = Expression::String(StringExpression::new(&combined).unwrap());
									new_args.push(new_expr);
								}
								else {
									let binary = BinaryExpression::new(
										BinaryOperator::Plus,
										Expression::String(StringExpression::new(&("\"".to_string() + &self.prefix + "\"")).unwrap()),
										exprs.get(i).unwrap().clone()
									);
									new_args.push(binary);
								}
							}
							else {
								new_args.push(exprs.get(i).unwrap().clone())
							}
						}
						let func = FunctionCall::new(fc.get_prefix().clone(), new_args, None);
						let ec = Expression::Call(Box::new(func));
						Some(ec)
					}
					else {
						None
					}
				}
				else {
					None
				}
			}
			_ => None
		}
	}
}

impl NodeProcessor for Prefixer {
	fn process_expression(&mut self, expr: &mut Expression) {
		if let Some(replace) = self.replace_with(expr) {
			*expr = replace;
		}
	}
}

impl RuleConfiguration for PrefixRequireRule {
	fn configure(&mut self, properties: darklua_core::rules::RuleProperties) -> Result<(), darklua_core::rules::RuleConfigurationError> {
		Ok(())
	}

	fn get_name(&self) -> &'static str {
		"prefix-require"
	}

	fn serialize_to_properties(&self) -> darklua_core::rules::RuleProperties {
		RuleProperties::new()
	}
}

impl FlawlessRule for PrefixRequireRule {
	fn flawless_process(&self, block: &mut darklua_core::nodes::Block, _: &Context) {
		let mut processor = Prefixer { prefix: self.prefix.clone(), exceptions: self.exceptions.clone() };
		DefaultVisitor::visit_block(block, &mut processor);
	}
}
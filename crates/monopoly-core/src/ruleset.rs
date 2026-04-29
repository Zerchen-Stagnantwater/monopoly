pub trait RuleSet {
    fn name(&self) -> &str;
}

pub struct StandardRules;

impl RuleSet for StandardRules {
    fn name(&self) -> &str {
        "Standard Monopoly"
    }
}

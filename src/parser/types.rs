#[derive(Debug, PartialEq, PartialOrd)]
pub enum CMakeValue {
    ArgumentSpecifier(String),
    QuotedString(String),
    StringLiteral(String),
    Comment(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeCommand {
    pub name: String,
    pub args: Vec<CMakeValue>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CMakeStatement {
    CMakeCommandStatement(CMakeCommand),
    CMakeCommentStatement(String),
    CMakeNewline,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeDocument {
    pub statements: Vec<CMakeStatement>,
}

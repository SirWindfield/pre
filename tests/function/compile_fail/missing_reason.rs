use pre::{check_pre, pre};

#[pre(condition("must be bar"))]
fn foo() {}

#[check_pre]
fn main() {
    #[assert_pre(condition("must be bar"))]
    foo()
}
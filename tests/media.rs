#![cfg(test)]

#[macro_use]
mod macros;

test!(
    basic_toplevel,
    "@media foo {\n  a {\n    color: red;\n  }\n}\n"
);
error!(
    no_params,
    "@media {\n  a {\n    color: red;\n  }\n}\n", "Error: Expected identifier."
);
test!(
    basic_nested,
    "a {\n  @media foo {\n  color: red;\n  }\n}\n",
    "@media foo {\n  a {\n    color: red;\n  }\n}\n"
);
test!(empty_body, "@media (min-width: 2px) {}", "");
test!(
    newlines_are_not_emitted_for_child_styles,
    "a {
        @media screen {
            b {
                color: red;
            }
            c {
                color: green;
            }
        }
    }",
    "@media screen {\n  a b {\n    color: red;\n  }\n  a c {\n    color: green;\n  }\n}\n"
);

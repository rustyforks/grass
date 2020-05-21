#![cfg(test)]

#[macro_use]
mod macros;

test!(
    one_or_two,
    "a {\n  color: 1 or 2;\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    two_or_one,
    "a {\n  color: 2 or 1;\n}\n",
    "a {\n  color: 2;\n}\n"
);
test!(
    true_or_true,
    "a {\n  color: true or true;\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    true_or_false,
    "a {\n  color: true or false;\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    false_or_true,
    "a {\n  color: false or true;\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    false_or_false,
    "a {\n  color: false or false;\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    null_or_one,
    "a {\n  color: null or 1;\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    one_or_null,
    "a {\n  color: 1 or null;\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    one_or_two_or_three,
    "a {\n  color: 1 or 2 or 3;\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    part_of_binop,
    "a {\n  color: 1 - or;\n}\n",
    "a {\n  color: 1-or;\n}\n"
);
test!(
    #[ignore = "casing is not preserved for keyword operators"]
    part_of_binop_casing,
    "a {\n  color: 1 - OR;\n}\n",
    "a {\n  color: 1-OR;\n}\n"
);

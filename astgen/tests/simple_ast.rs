use astgen::*;

#[test]
fn can_generate_ast() {
    generate_ast!(
        TestAst,
        [ S => s: String ]
    );
    let _node = TestAst::S(S { s: "".into() });
}

#[test]
fn can_change_ast_name() {
    generate_ast!(
        OtherAst,
        [ S => s: String ]
    );
    let _node = OtherAst::S(S { s: "".into() });
}
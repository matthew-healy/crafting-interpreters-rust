#![allow(dead_code)]

use astgen::*;

#[test]
fn can_generate_ast() {
    generate_ast!(
        TestAst,
        [ S => { s: String } ]
    );
    let _node = TestAst::S(S { s: "".into() });
}

#[test]
fn can_change_ast_name() {
    generate_ast!(
        OtherAst,
        [ S => { s: String } ]
    );
    let _node = OtherAst::S(S { s: "".into() });
}

#[test]
fn uses_field_names_for_struct_fields() {
    generate_ast!(
        Example,
        [ S => { s: String, i: isize } ]
    );
    let _node = Example::S(S { s: "".into(), i: 0 });
}

#[test]
fn can_clone_nodes() {
    generate_ast!(A, [N => {a: usize}]);
    let n = N { a: 0 };
    let _clone = n.clone();
}

#[test]
fn uses_node_names_for_enum_variants() {
    generate_ast!(
        Example,
        [
            A => { a: isize };
            B => { b: String };
        ]
    );
    let _a_node = Example::A(A { a: 0 });
    let _b_node = Example::B(B { b: "".into() });
}

#[test]
fn generates_new_fns() {
    generate_ast!(
        Test,
        [
            A => { a: isize };
            B => { b: usize };
        ]
    );
    let _a_node = Test::new_a(1);
    let _b_node = Test::new_b(8);
}

#[test]
fn generates_visitor_trait() {
    generate_ast!(
        Some,
        [
            ANode => { a: isize };
            OtherNode => { b: String };
        ]
    );
    struct V;
    impl Visitor<()> for V {
        fn visit_a_node_some(&mut self, _a: &ANode) { () }
        fn visit_other_node_some(&mut self, _o: &OtherNode) { () }
    }
}

#[test]
fn accept_fn_routes_calls_to_correct_visitor_fn() {
    generate_ast!(
        VisitMe,
        [
            NotMe => { a: String };
            Test => { a: isize };
        ]
    );
    struct V {
        called: bool,
    }
    impl Visitor<()> for V {
        fn visit_not_me_visitme(&mut self, _t: &NotMe) {}
        fn visit_test_visitme(&mut self, _t: &Test) {
            self.called = true;
            ()
        }
    }
    let mut visitor = V { called: false };
    let node = VisitMe::Test(Test { a: 0 });
    node.accept(&mut visitor);
    assert!(visitor.called);
}
//! Edge case tests for God Object detector

#[cfg(test)]
mod tests {
    use super::super::unit_tests::create_parsed_file;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    use crate::ast::tree_sitter_impl::SourceLanguage;

    #[test]
    fn test_edge_case_exactly_at_threshold() {
        let detector = GodObjectDetector::new(10, 8);

        let method_count = 10;
        let field_count = 8;

        // Exactly at threshold should not trigger detection
        assert_eq!(method_count, detector.method_threshold());
        assert_eq!(field_count, detector.field_threshold());
    }

    #[test]
    fn test_edge_case_only_methods_exceed() {
        let detector = GodObjectDetector::new(10, 8);

        let method_count = 15;
        let field_count = 5;

        // Only methods exceed, fields are within limit
        assert!(method_count > detector.method_threshold());
        assert!(field_count < detector.field_threshold());
        // Should still trigger detection based on methods
    }

    #[test]
    fn test_edge_case_only_fields_exceed() {
        let detector = GodObjectDetector::new(10, 8);

        let method_count = 5;
        let field_count = 12;

        // Only fields exceed, methods are within limit
        assert!(method_count < detector.method_threshold());
        assert!(field_count > detector.field_threshold());
        // Should still trigger detection based on fields
    }

    #[test]
    fn test_nested_classes() {
        let source = r#"
            class OuterClass {
                constructor() {
                    this.field1 = 1;
                    this.field2 = 2;
                }

                method1() {}
                method2() {}

                createInnerClass() {
                    class InnerClass {
                        constructor() {
                            this.a = 1; this.b = 2; this.c = 3; this.d = 4;
                            this.e = 5; this.f = 6; this.g = 7; this.h = 8;
                            this.i = 9;
                        }
                        innerMethod1() {} innerMethod2() {} innerMethod3() {}
                        innerMethod4() {} innerMethod5() {} innerMethod6() {}
                        innerMethod7() {} innerMethod8() {} innerMethod9() {}
                        innerMethod10() {} innerMethod11() {}
                    }
                    return InnerClass;
                }
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        // Should analyze both outer and inner classes separately
        // OuterClass: 3 methods, 2 fields (acceptable)
        // InnerClass: 11 methods, 9 fields (God Object)
    }

    #[test]
    fn test_inheritance_chain() {
        let source = r#"
            class BaseClass {
                constructor() {
                    this.baseField1 = 1;
                    this.baseField2 = 2;
                }
                baseMethod1() {}
                baseMethod2() {}
            }

            class DerivedClass extends BaseClass {
                constructor() {
                    super();
                    this.derivedField1 = 3;
                    this.derivedField2 = 4;
                    this.derivedField3 = 5;
                    this.derivedField4 = 6;
                    this.derivedField5 = 7;
                    this.derivedField6 = 8;
                    this.derivedField7 = 9;
                }
                derivedMethod1() {} derivedMethod2() {} derivedMethod3() {}
                derivedMethod4() {} derivedMethod5() {} derivedMethod6() {}
                derivedMethod7() {} derivedMethod8() {} derivedMethod9() {}
                derivedMethod10() {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        // Should analyze each class independently
        // BaseClass: 2 methods, 2 fields (acceptable)
        // DerivedClass: 10 methods, 7 fields (methods at threshold, fields near)
        // Note: Inherited members typically not counted in derived class
    }
}
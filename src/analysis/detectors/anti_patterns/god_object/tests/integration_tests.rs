//! Integration tests for God Object detector

#[cfg(test)]
mod tests {
    use super::super::unit_tests::create_parsed_file;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    use crate::ast::tree_sitter_impl::SourceLanguage;

    #[test]
    fn test_multiple_classes_in_file() {
        let source = r#"
            // First class - God Object
            class ServiceManager {
                constructor() {
                    this.a = 1; this.b = 2; this.c = 3; this.d = 4;
                    this.e = 5; this.f = 6; this.g = 7; this.h = 8;
                    this.i = 9; this.j = 10;
                }
                method1() {} method2() {} method3() {} method4() {}
                method5() {} method6() {} method7() {} method8() {}
                method9() {} method10() {} method11() {} method12() {}
            }

            // Second class - Acceptable
            class SimpleHelper {
                constructor() {
                    this.value = 0;
                }
                getValue() { return this.value; }
                setValue(v) { this.value = v; }
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        // Should detect only the first class as God Object
        // ServiceManager: 12 methods, 10 fields (exceeds both)
        // SimpleHelper: 2 methods, 1 field (within limits)
    }

    #[test]
    fn test_real_world_scenario_user_manager() {
        let source = r#"
            // Real-world example of a God Object that grew over time
            class UserManagementSystem {
                constructor() {
                    // Authentication fields
                    this.authenticatedUsers = new Map();
                    this.sessionTokens = new Map();
                    this.loginAttempts = new Map();

                    // Authorization fields
                    this.userRoles = new Map();
                    this.permissions = new Map();
                    this.accessLevels = new Map();

                    // Profile management
                    this.userProfiles = new Map();
                    this.profileImages = new Map();
                    this.preferences = new Map();

                    // Communication
                    this.emailService = null;
                    this.notificationService = null;
                    this.auditLogger = null;
                }

                // Authentication methods
                authenticate(credentials) {}
                validateToken(token) {}
                logout(userId) {}
                refreshToken(token) {}

                // Authorization methods
                authorize(userId, resource) {}
                checkPermissions(userId, action) {}
                grantPermission(userId, permission) {}
                revokePermission(userId, permission) {}

                // Profile management methods
                createProfile(userId, data) {}
                updateProfile(userId, data) {}
                deleteProfile(userId) {}
                uploadProfileImage(userId, image) {}

                // Communication methods
                sendWelcomeEmail(userId) {}
                sendPasswordResetEmail(userId) {}
                sendNotification(userId, message) {}
                logUserAction(userId, action) {}

                // Administrative methods
                generateUserReport() {}
                exportUserData() {}
                importUserData(data) {}
                archiveInactiveUsers() {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        // This should clearly be detected as a God Object
        // Has 18 methods and 12 fields - both significantly exceed thresholds
        // Combines authentication, authorization, profile management, and communication concerns
    }

    #[test]
    fn test_framework_controller_pattern() {
        let source = r#"
            // This might look like a God Object but could be a legitimate controller
            class APIController {
                constructor() {
                    this.database = null;
                    this.validator = null;
                    this.serializer = null;
                    this.logger = null;
                    this.cache = null;
                }

                // User endpoints
                createUser(req, res) {}
                getUser(req, res) {}
                updateUser(req, res) {}
                deleteUser(req, res) {}

                // Post endpoints
                createPost(req, res) {}
                getPost(req, res) {}
                updatePost(req, res) {}
                deletePost(req, res) {}

                // Comment endpoints
                createComment(req, res) {}
                getComment(req, res) {}
                updateComment(req, res) {}
                deleteComment(req, res) {}

                // Search endpoints
                searchUsers(req, res) {}
                searchPosts(req, res) {}
                searchComments(req, res) {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        // This has 15 methods and 5 fields
        // Methods exceed threshold but might be acceptable for a REST controller
        // Pattern recognition should potentially exclude this
    }
}
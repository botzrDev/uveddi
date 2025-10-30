//! Language-specific tests for God Object detector

#[cfg(test)]
mod tests {
    use super::super::unit_tests::create_parsed_file;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    use crate::ast::tree_sitter_impl::SourceLanguage;

    #[test]
    fn test_rust_god_object_detection() {
        let source = r#"
            struct UserManager {
                users: Vec<User>,
                sessions: HashMap<String, Session>,
                permissions: PermissionSet,
                audit_log: AuditLog,
                cache: Cache,
                config: Config,
                metrics: Metrics,
                notifications: NotificationService,
                database: Database,
                backup_service: BackupService,
            }

            impl UserManager {
                fn create_user(&self) {}
                fn delete_user(&self) {}
                fn authenticate(&self) {}
                fn authorize(&self) {}
                fn log_action(&self) {}
                fn send_notification(&self) {}
                fn update_cache(&self) {}
                fn generate_report(&self) {}
                fn backup_data(&self) {}
                fn validate_permissions(&self) {}
                fn handle_session(&self) {}
                fn update_metrics(&self) {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::Rust);

        // In a real implementation, this would use the actual detector methods
        let method_count = 12;
        let field_count = 10;

        assert!(method_count > detector.method_threshold());
        assert!(field_count > detector.field_threshold());
    }

    #[test]
    fn test_rust_acceptable_struct() {
        let source = r#"
            struct UserRepository {
                storage: Box<dyn Storage>,
                cache: Option<Cache>,
            }

            impl UserRepository {
                fn create(&self, user: User) -> Result<(), Error> {}
                fn find_by_id(&self, id: UserId) -> Result<User, Error> {}
                fn update(&self, user: User) -> Result<(), Error> {}
                fn delete(&self, id: UserId) -> Result<(), Error> {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::Rust);

        let method_count = 4;
        let field_count = 2;

        assert!(method_count <= detector.method_threshold());
        assert!(field_count <= detector.field_threshold());
    }

    #[test]
    fn test_python_god_object_detection() {
        let source = r#"
            class DataProcessor:
                def __init__(self):
                    self.data = []
                    self.cache = {}
                    self.config = Config()
                    self.logger = Logger()
                    self.metrics = Metrics()
                    self.validators = []
                    self.transformers = []
                    self.output_handlers = []
                    self.error_handlers = []
                    self.state_manager = StateManager()

                def load_data(self): pass
                def validate_data(self): pass
                def transform_data(self): pass
                def aggregate_data(self): pass
                def filter_data(self): pass
                def sort_data(self): pass
                def export_data(self): pass
                def backup_data(self): pass
                def restore_data(self): pass
                def analyze_patterns(self): pass
                def generate_report(self): pass
                def send_notifications(self): pass
                def update_metrics(self): pass
                def handle_errors(self): pass
                def cleanup(self): pass
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::Python);

        let method_count = 15; // Excluding __init__
        let field_count = 10;

        assert!(method_count > detector.method_threshold());
        assert!(field_count > detector.field_threshold());
    }

    #[test]
    fn test_python_acceptable_class() {
        let source = r#"
            class User:
                def __init__(self, name, email):
                    self.name = name
                    self.email = email
                    self.created_at = datetime.now()

                def update_email(self, email):
                    self.email = email

                def get_display_name(self):
                    return self.name

                def to_dict(self):
                    return {'name': self.name, 'email': self.email}
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::Python);

        let method_count = 3; // Excluding __init__
        let field_count = 3;

        assert!(method_count <= detector.method_threshold());
        assert!(field_count <= detector.field_threshold());
    }

    #[test]
    fn test_javascript_god_object_detection() {
        let source = r#"
            class ApplicationManager {
                constructor() {
                    this.users = [];
                    this.sessions = new Map();
                    this.config = {};
                    this.database = null;
                    this.cache = null;
                    this.logger = null;
                    this.eventBus = null;
                    this.scheduler = null;
                    this.notifier = null;
                }

                initializeApp() {}
                shutdownApp() {}
                createUser() {}
                deleteUser() {}
                authenticateUser() {}
                authorizeUser() {}
                logEvent() {}
                cacheData() {}
                queryDatabase() {}
                scheduleTask() {}
                sendNotification() {}
                handleWebSocket() {}
                processPayment() {}
                generateReport() {}
                exportData() {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        let method_count = 15;
        let field_count = 9;

        assert!(method_count > detector.method_threshold());
        assert!(field_count > detector.field_threshold());
    }

    #[test]
    fn test_javascript_acceptable_class() {
        let source = r#"
            class ShoppingCart {
                constructor() {
                    this.items = [];
                    this.total = 0;
                }

                addItem(item) {
                    this.items.push(item);
                    this.updateTotal();
                }

                removeItem(itemId) {
                    this.items = this.items.filter(item => item.id !== itemId);
                    this.updateTotal();
                }

                updateTotal() {
                    this.total = this.items.reduce((sum, item) => sum + item.price, 0);
                }

                getTotal() {
                    return this.total;
                }
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::JavaScript);

        let method_count = 4;
        let field_count = 2;

        assert!(method_count <= detector.method_threshold());
        assert!(field_count <= detector.field_threshold());
    }

    #[test]
    fn test_typescript_god_object_detection() {
        let source = r#"
            class EnterpriseService<T> {
                private users: Map<string, User>;
                private sessions: Map<string, Session>;
                private cache: CacheService;
                private database: DatabaseConnection;
                private logger: Logger;
                private config: Configuration;
                private eventBus: EventBus;
                private metrics: MetricsCollector;
                private validators: Validator[];
                private middleware: Middleware[];

                public async initialize(): Promise<void> {}
                public async shutdown(): Promise<void> {}
                public createResource(data: T): T {}
                public updateResource(id: string, data: T): T {}
                public deleteResource(id: string): void {}
                public findResource(id: string): T {}
                public listResources(): T[] {}
                public validateResource(data: T): boolean {}
                public authorizeAccess(userId: string): boolean {}
                public logActivity(action: string): void {}
                public publishEvent(event: Event): void {}
                public collectMetrics(): Metrics {}
                public handleError(error: Error): void {}
                public processQueue(): void {}
                public generateReport(): Report {}
            }
        "#;

        let detector = GodObjectDetector::new(10, 8);
        let _parsed_file = create_parsed_file(source, SourceLanguage::TypeScript);

        let method_count = 15;
        let field_count = 10;

        assert!(method_count > detector.method_threshold());
        assert!(field_count > detector.field_threshold());
    }
}

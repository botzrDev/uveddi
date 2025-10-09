# God Object Anti-pattern Example
class MegaManager:
    def __init__(self):
        self.database = None
        self.file_system = None
        self.network = None
        self.ui = None
        self.cache = None
        self.security = None
        self.validation = None
        self.logging = None
    
    # Too many methods - God Object
    def handle_users(self): pass
    def manage_files(self): pass
    def process_network(self): pass
    def update_ui(self): pass
    def cache_data(self): pass
    def validate_input(self): pass
    def log_events(self): pass
    def handle_security(self): pass
    def manage_config(self): pass
    def process_payments(self): pass
    def generate_reports(self): pass
    def backup_data(self): pass
    def send_notifications(self): pass
    def monitor_system(self): pass
    def analyze_data(self): pass
    def method1(self): pass
    def method2(self): pass
    def method3(self): pass
    def method4(self): pass
    def method5(self): pass

# Dead code - should be detected
def never_called_function():
    print("This function is never used")
    return 42

def another_dead_function():
    unused_variable = "This is dead code"
    return unused_variable

# Magic numbers - should be detected
def calculate_discount(price):
    return price * 0.15  # Magic number 0.15

def get_timeout():
    return 30000  # Magic number 30000

def process_batch():
    batch_size = 500  # Magic number 500
    max_retries = 3   # Magic number 3
    for i in range(batch_size):
        if i % max_retries == 0:
            print(f"Processing batch {i}")
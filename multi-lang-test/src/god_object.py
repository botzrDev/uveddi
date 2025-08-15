#!/usr/bin/env python3
"""
God Object example in Python - handles too many responsibilities
"""
import json
import logging
from typing import Dict, List, Any, Optional
from datetime import datetime


class MassiveApplicationManager:
    """
    A god object that violates the Single Responsibility Principle
    by handling database, network, file operations, UI state, and business logic
    """
    
    def __init__(self):
        # Database management
        self.db_connections = []
        self.query_cache = {}
        
        # File system management
        self.open_files = {}
        self.file_cache = {}
        
        # Network management
        self.http_sessions = {}
        self.api_cache = {}
        
        # UI state management
        self.window_states = {}
        self.ui_components = {}
        
        # Configuration management
        self.config = {}
        self.environment_vars = {}
        
        # Logging and monitoring
        self.log_buffer = []
        self.metrics = {}
        
        # Business logic state
        self.user_sessions = {}
        self.business_rules = []
        
        # Initialize logger
        self.logger = logging.getLogger(__name__)
    
    # Database methods (should be separate service)
    def connect_database(self, connection_string: str) -> bool:
        """Connect to database"""
        try:
            # Simulate connection
            self.db_connections.append(connection_string)
            return True
        except Exception as e:
            self.log_error(f"Database connection failed: {e}")
            return False
    
    def execute_query(self, query: str, params: tuple = ()) -> List[Dict]:
        """Execute database query"""
        if not self.db_connections:
            raise RuntimeError("No database connection available")
        
        # Simulate query execution
        result = [{"id": 1, "name": "test"}]
        
        # Cache the result
        cache_key = f"{query}:{params}"
        self.query_cache[cache_key] = result
        
        return result
    
    def cache_query_result(self, query: str, result: List[Dict]) -> None:
        """Cache query result"""
        self.query_cache[query] = result
    
    # File system methods (should be separate service)
    def read_file(self, file_path: str) -> str:
        """Read file with caching"""
        if file_path in self.file_cache:
            return self.file_cache[file_path]
        
        try:
            # Simulate file reading
            content = "file content"
            self.file_cache[file_path] = content
            return content
        except Exception as e:
            self.log_error(f"Failed to read file {file_path}: {e}")
            raise
    
    def write_file(self, file_path: str, content: str) -> bool:
        """Write content to file"""
        try:
            # Simulate file writing
            self.file_cache[file_path] = content
            return True
        except Exception as e:
            self.log_error(f"Failed to write file {file_path}: {e}")
            return False
    
    # Network methods (should be separate service)
    def make_http_request(self, url: str, method: str = 'GET', data: Dict = None) -> Dict:
        """Make HTTP request"""
        try:
            # Simulate HTTP request
            result = {"status": "success", "data": "response"}
            
            # Cache the response
            cache_key = f"{method}:{url}:{json.dumps(data) if data else ''}"
            self.api_cache[cache_key] = result
            
            return result
        except Exception as e:
            self.log_error(f"HTTP request failed: {e}")
            raise
    
    def get_cached_api_response(self, url: str, method: str = 'GET', data: Dict = None) -> Optional[Dict]:
        """Get cached API response"""
        cache_key = f"{method}:{url}:{json.dumps(data) if data else ''}"
        return self.api_cache.get(cache_key)
    
    # UI methods (should be separate service)
    def set_window_state(self, window_id: str, state: Dict) -> None:
        """Set window state"""
        self.window_states[window_id] = state
    
    def get_window_state(self, window_id: str) -> Optional[Dict]:
        """Get window state"""
        return self.window_states.get(window_id)
    
    def register_ui_component(self, component_id: str, component: Any) -> None:
        """Register UI component"""
        self.ui_components[component_id] = component
    
    def update_ui_component(self, component_id: str, properties: Dict) -> bool:
        """Update UI component properties"""
        if component_id in self.ui_components:
            component = self.ui_components[component_id]
            for key, value in properties.items():
                setattr(component, key, value)
            return True
        return False
    
    # Configuration methods (should be separate service)
    def load_config(self, config_path: str) -> bool:
        """Load configuration from file"""
        try:
            content = self.read_file(config_path)
            self.config = json.loads(content)
            return True
        except Exception as e:
            self.log_error(f"Failed to load config: {e}")
            return False
    
    def save_config(self, config_path: str) -> bool:
        """Save configuration to file"""
        try:
            content = json.dumps(self.config, indent=2)
            return self.write_file(config_path, content)
        except Exception as e:
            self.log_error(f"Failed to save config: {e}")
            return False
    
    def get_config_value(self, key: str, default: Any = None) -> Any:
        """Get configuration value"""
        return self.config.get(key, default)
    
    def set_config_value(self, key: str, value: Any) -> None:
        """Set configuration value"""
        self.config[key] = value
    
    # Logging methods (should be separate service)
    def log_message(self, level: str, message: str) -> None:
        """Log message"""
        timestamp = datetime.now().isoformat()
        log_entry = f"[{timestamp}] {level}: {message}"
        self.log_buffer.append(log_entry)
        
        # Also log to Python logger
        getattr(self.logger, level.lower(), self.logger.info)(message)
    
    def log_error(self, message: str) -> None:
        """Log error message"""
        self.log_message("ERROR", message)
    
    def log_info(self, message: str) -> None:
        """Log info message"""
        self.log_message("INFO", message)
    
    def get_recent_logs(self, count: int = 100) -> List[str]:
        """Get recent log entries"""
        return self.log_buffer[-count:]
    
    # Metrics methods (should be separate service)
    def record_metric(self, name: str, value: float, tags: Dict[str, str] = None) -> None:
        """Record metric value"""
        timestamp = datetime.now().isoformat()
        metric_entry = {
            'name': name,
            'value': value,
            'timestamp': timestamp,
            'tags': tags or {}
        }
        
        if name not in self.metrics:
            self.metrics[name] = []
        
        self.metrics[name].append(metric_entry)
    
    def get_metric_values(self, name: str) -> List[float]:
        """Get all values for a metric"""
        if name in self.metrics:
            return [entry['value'] for entry in self.metrics[name]]
        return []
    
    def get_metric_average(self, name: str) -> Optional[float]:
        """Get average value for a metric"""
        values = self.get_metric_values(name)
        return sum(values) / len(values) if values else None
    
    # Business logic methods (should be separate service)
    def create_user_session(self, user_id: str, session_data: Dict = None) -> str:
        """Create user session"""
        import uuid
        session_id = str(uuid.uuid4())
        
        session = {
            'user_id': user_id,
            'session_id': session_id,
            'created_at': datetime.now().isoformat(),
            'data': session_data or {}
        }
        
        self.user_sessions[session_id] = session
        return session_id
    
    def validate_business_rules(self, data: Dict) -> List[str]:
        """Validate data against business rules"""
        errors = []
        
        for rule in self.business_rules:
            if not self._evaluate_business_rule(rule, data):
                errors.append(f"Business rule violation: {rule.get('description', 'Unknown rule')}")
        
        return errors
    
    def _evaluate_business_rule(self, rule: Dict, data: Dict) -> bool:
        """Evaluate a single business rule"""
        # Simplified rule evaluation
        rule_type = rule.get('type', 'unknown')
        
        if rule_type == 'required_field':
            field_name = rule.get('field')
            return field_name in data and data[field_name] is not None
        
        if rule_type == 'range_check':
            field_name = rule.get('field')
            min_value = rule.get('min', float('-inf'))
            max_value = rule.get('max', float('inf'))
            
            if field_name in data:
                value = data[field_name]
                return min_value <= value <= max_value
        
        return True
    
    def process_business_transaction(self, transaction_data: Dict) -> Dict:
        """Process business transaction"""
        transaction_id = transaction_data.get('id', 'unknown')
        
        # Log transaction start
        self.log_info(f"Processing transaction: {transaction_id}")
        
        # Record metric
        self.record_metric('transactions_processed', 1.0)
        
        # Validate business rules
        validation_errors = self.validate_business_rules(transaction_data)
        if validation_errors:
            self.log_error(f"Transaction validation failed: {validation_errors}")
            return {
                'success': False,
                'errors': validation_errors,
                'transaction_id': transaction_id
            }
        
        # Simulate processing
        processing_result = {
            'success': True,
            'transaction_id': transaction_id,
            'processed_at': datetime.now().isoformat(),
            'result_data': {'status': 'completed'}
        }
        
        # Log success
        self.log_info(f"Transaction processed successfully: {transaction_id}")
        
        return processing_result
    
    # The main method that tries to do everything
    def handle_application_request(self, request_data: Dict) -> Dict:
        """Main method that handles everything - violates SRP"""
        request_id = request_data.get('id', 'unknown')
        
        try:
            # Log request
            self.log_info(f"Handling application request: {request_id}")
            
            # Load configuration
            if not self.load_config('app_config.json'):
                return {'success': False, 'error': 'Failed to load configuration'}
            
            # Connect to database
            db_url = self.get_config_value('database_url', 'data.db')
            if not self.connect_database(db_url):
                return {'success': False, 'error': 'Failed to connect to database'}
            
            # Make API calls
            api_url = self.get_config_value('external_api_url')
            if api_url:
                try:
                    api_response = self.make_http_request(api_url, 'GET')
                    self.log_info(f"API response received: {len(str(api_response))} characters")
                except Exception as e:
                    self.log_error(f"API call failed: {e}")
            
            # Process files
            input_file = request_data.get('input_file')
            if input_file:
                try:
                    file_content = self.read_file(input_file)
                    self.log_info(f"File processed: {len(file_content)} characters")
                except Exception as e:
                    self.log_error(f"File processing failed: {e}")
            
            # Update UI state
            window_id = request_data.get('window_id', 'main')
            self.set_window_state(window_id, {'last_request': request_id})
            
            # Record metrics
            self.record_metric('requests_handled', 1.0)
            
            # Create user session
            user_id = request_data.get('user_id')
            if user_id:
                session_id = self.create_user_session(user_id, request_data.get('session_data'))
                self.log_info(f"User session created: {session_id}")
            
            # Process business transaction
            transaction_data = request_data.get('transaction')
            transaction_result = None
            if transaction_data:
                transaction_result = self.process_business_transaction(transaction_data)
            
            # Execute database queries
            user_query = "SELECT * FROM users WHERE active = 1"
            try:
                users = self.execute_query(user_query)
                self.log_info(f"Found {len(users)} active users")
            except Exception as e:
                self.log_error(f"Database query failed: {e}")
                users = []
            
            # Prepare response
            response = {
                'success': True,
                'request_id': request_id,
                'processed_at': datetime.now().isoformat(),
                'user_count': len(users),
                'transaction_result': transaction_result,
                'metrics': {
                    'requests_handled': len(self.get_metric_values('requests_handled')),
                    'transactions_processed': len(self.get_metric_values('transactions_processed'))
                }
            }
            
            self.log_info(f"Request processed successfully: {request_id}")
            return response
            
        except Exception as e:
            self.log_error(f"Request processing failed: {e}")
            return {
                'success': False,
                'request_id': request_id,
                'error': str(e),
                'processed_at': datetime.now().isoformat()
            }


# Supporting classes for the god object
class BusinessRule:
    def __init__(self, rule_type: str, description: str, **kwargs):
        self.type = rule_type
        self.description = description
        self.parameters = kwargs
    
    def to_dict(self) -> Dict:
        return {
            'type': self.type,
            'description': self.description,
            **self.parameters
        }


# Dead code that is never called
def unused_function():
    """This function is never called"""
    return "unused"


def another_unused_function(param):
    """Another unused function"""
    return param * 2 + unused_helper()


def unused_helper():
    """Helper function that's never used"""
    return 999


# Dead classes
class UnusedClass:
    """Class that's never instantiated"""
    
    def __init__(self, value):
        self.value = value
    
    def unused_method(self):
        """Method that's never called"""
        return self.value * 2
    
    @staticmethod
    def unused_static_method():
        """Static method that's never called"""
        return "static"
    
    @classmethod
    def unused_class_method(cls):
        """Class method that's never called"""
        return cls()


# Dead global variables
UNUSED_CONSTANT = 123
ANOTHER_UNUSED_CONSTANT = "unused"


# Dead decorators
def unused_decorator(func):
    """Decorator that's never used"""
    def wrapper(*args, **kwargs):
        print(f"Calling {func.__name__}")
        return func(*args, **kwargs)
    return wrapper


# Dead generators
def unused_generator():
    """Generator that's never used"""
    for i in range(10):
        yield i * 2


# Dead context managers
class UnusedContextManager:
    """Context manager that's never used"""
    
    def __enter__(self):
        print("Entering context")
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        print("Exiting context")


# Dead exception classes
class UnusedException(Exception):
    """Exception that's never raised"""
    pass


class AnotherUnusedException(ValueError):
    """Another unused exception"""
    def __init__(self, message):
        super().__init__(message)
        self.custom_data = "unused"


# Mixed usage - some used, some not
def mixed_usage_entry():
    """Entry point that uses some helpers"""
    return used_helper()


def used_helper():
    """Helper that is actually used"""
    return 42


def unused_helper_in_mixed():
    """Helper in mixed module that's never used"""
    return 99


# Dead lambda functions
unused_lambda = lambda x: x * 2


# Dead list comprehensions assigned to variables
unused_list_comp = [i * 2 for i in range(100)]


# Dead dictionary
unused_dict = {
    "key1": "value1",
    "key2": "value2",
    "key3": unused_function  # References dead function
}
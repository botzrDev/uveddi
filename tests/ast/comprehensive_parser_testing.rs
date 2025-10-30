use uveddi::ast::tree_sitter_impl::{AstParser, SourceLanguage, AstError};
use std::path::Path;
use tempfile::tempdir;
use std::fs;

#[cfg(test)]
mod multi_language_ast_coverage {
    use super::*;

    fn create_test_file(content: &str, extension: &str) -> std::path::PathBuf {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(format!("test.{}", extension));
        fs::write(&file_path, content).unwrap();
        file_path
    }

    // RUST EDGE CASE TESTS
    #[test]
    fn test_rust_complex_lifetime_annotations() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
struct LifetimeTest<'a, 'b: 'a> {
    data: &'a str,
    reference: &'b Option<&'a str>,
}

impl<'a, 'b> LifetimeTest<'a, 'b>
where 
    'b: 'a,
{
    fn complex_method<'c>(&'c self, input: &'c str) -> &'c str 
    where 
        'a: 'c,
        'b: 'c,
    {
        input
    }
}

fn higher_order_lifetimes<'a, F>(f: F) -> impl Fn(&'a str) -> &'a str
where
    F: Fn(&'a str) -> &'a str,
{
    f
}
"#;
        
        let file_path = create_test_file(content, "rs");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse complex lifetime annotations");
        
        let parsed = result.unwrap();
        assert_eq!(parsed.language, SourceLanguage::Rust);
    }

    #[test]
    fn test_rust_unsafe_blocks_with_raw_pointers() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
use std::ptr;

struct UnsafeOperations {
    raw_ptr: *mut i32,
    const_ptr: *const u8,
}

impl UnsafeOperations {
    unsafe fn manipulate_raw_pointers(&mut self) -> Result<i32, &'static str> {
        if self.raw_ptr.is_null() {
            return Err("Null pointer");
        }
        
        let value = ptr::read(self.raw_ptr);
        ptr::write(self.raw_ptr, value + 1);
        
        // Complex pointer arithmetic
        let offset_ptr = self.raw_ptr.add(1);
        let aligned_ptr = ptr::align_of::<i32>() as *mut i32;
        
        Ok(*self.raw_ptr)
    }
    
    unsafe fn transmute_operations(&self) -> usize {
        std::mem::transmute::<*const u8, usize>(self.const_ptr)
    }
}

union UnsafeUnion {
    int_val: i32,
    float_val: f32,
    bytes: [u8; 4],
}
"#;
        
        let file_path = create_test_file(content, "rs");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse unsafe blocks with raw pointers");
    }

    #[test]
    fn test_rust_macro_expansion_edge_cases() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
macro_rules! complex_macro {
    // Pattern matching with repetitions
    ($($name:ident: $type:ty),* $(,)?) => {
        $(
            pub fn $name() -> $type {
                Default::default()
            }
        )*
    };
    
    // Nested macro calls
    (nested => $($item:tt)*) => {
        complex_macro!($($item)*);
    };
    
    // Token tree manipulation
    (@internal $($tt:tt)*) => {
        stringify!($($tt)*)
    };
}

// Procedural macro style syntax
#[derive(Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ComplexDerive {
    #[serde(skip_serializing_if = "Option::is_none")]
    optional_field: Option<String>,
}

// Complex macro invocations
complex_macro!(
    get_string: String,
    get_number: i32,
);

// Nested macro calls
macro_rules! nested_expansion {
    ($macro_name:ident) => {
        $macro_name!(nested => get_bool: bool,);
    };
}

nested_expansion!(complex_macro);

// Attribute macros with complex syntax
#[cfg(all(feature = "async", not(target_os = "windows")))]
async fn conditional_async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
"#;
        
        let file_path = create_test_file(content, "rs");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse macro expansion edge cases");
    }

    #[test]
    fn test_rust_generic_where_clauses_complex() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
use std::fmt::{Debug, Display};
use std::ops::{Add, Mul};

trait ComplexTrait<T, U> 
where
    T: Debug + Display + Clone,
    U: Add<Output = U> + Mul<Output = U>,
{
    type Associated: Debug;
    
    fn complex_method<V>(
        &self, 
        param1: T, 
        param2: U
    ) -> Self::Associated
    where
        V: Iterator<Item = T>,
        T: PartialEq<U>,
        Self::Associated: From<T>;
}

struct GenericStruct<T, U, V>
where
    T: Clone + Debug + PartialEq,
    U: Add<T, Output = V> + Mul<V, Output = T>,
    V: Display + Into<T>,
{
    field1: T,
    field2: U,
    field3: V,
}

impl<T, U, V> ComplexTrait<T, U> for GenericStruct<T, U, V>
where
    T: Debug + Display + Clone + PartialEq<U>,
    U: Add<Output = U> + Mul<Output = U> + Add<T, Output = V>,
    V: Display + Into<T> + Debug,
{
    type Associated = String;
    
    fn complex_method<W>(
        &self, 
        param1: T, 
        param2: U
    ) -> Self::Associated
    where
        W: Iterator<Item = T>,
        T: PartialEq<U>,
        Self::Associated: From<T>,
    {
        format!("{:?}", param1)
    }
}

// Higher-kinded type bounds
fn higher_kinded_function<F, T, U>(f: F) -> impl Fn(T) -> U
where
    F: Fn(T) -> U + Clone + Send + Sync + 'static,
    T: Debug + Clone + Send,
    U: Display + Send + 'static,
{
    move |x| f(x)
}
"#;
        
        let file_path = create_test_file(content, "rs");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse complex generic where clauses");
    }

    #[test]
    fn test_rust_async_closures_nested() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
use std::future::Future;
use std::pin::Pin;

async fn complex_async_function() -> Result<(), Box<dyn std::error::Error>> {
    // Async closure with move semantics
    let data = vec![1, 2, 3, 4, 5];
    
    let async_closure = move |x: i32| async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(x as u64)).await;
        x * 2
    };
    
    // Nested async operations
    let results: Vec<_> = data.into_iter()
        .map(|x| async_closure(x))
        .collect();
    
    // Complex future chaining
    let chained_future = async {
        let mut sum = 0;
        for future in results {
            sum += future.await;
        }
        sum
    };
    
    let final_result = chained_future.await;
    
    // Async closure returning future
    let higher_order_async = |f: Pin<Box<dyn Future<Output = i32>>>| async move {
        f.await + 10
    };
    
    // Stream processing with async closures
    let stream_processor = |items: Vec<i32>| async move {
        let mut processed = Vec::new();
        for item in items {
            let result = async move { item * item }.await;
            processed.push(result);
        }
        processed
    };
    
    let processed = stream_processor(vec![1, 2, 3]).await;
    
    Ok(())
}

// Complex async trait with associated types
trait AsyncTrait {
    type Output: Send + 'static;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn process(&self) -> Result<Self::Output, Self::Error>;
}

struct AsyncProcessor;

impl AsyncTrait for AsyncProcessor {
    type Output = String;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn process(&self) -> Result<Self::Output, Self::Error> {
        let nested_async = async {
            // Simulate complex async work
            tokio::spawn(async {
                "processed".to_string()
            }).await?
        };
        
        Ok(nested_async.await)
    }
}
"#;
        
        let file_path = create_test_file(content, "rs");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse nested async closures");
    }

    // PYTHON EDGE CASE TESTS
    #[test]
    fn test_python_async_generator_comprehensions() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
import asyncio
from typing import AsyncGenerator, AsyncIterator, List, Optional, Union
from dataclasses import dataclass, field
from functools import wraps

@dataclass
class ComplexClass:
    async_data: List[str] = field(default_factory=list)
    
    async def async_generator_method(self) -> AsyncGenerator[str, None]:
        # Complex async generator with nested comprehensions
        async for item in self._get_async_items():
            processed = [
                await self._process_item(x) 
                async for x in self._nested_async_generator(item)
                if await self._should_include(x)
            ]
            yield f"processed: {processed}"
    
    async def _get_async_items(self) -> AsyncIterator[str]:
        for item in self.async_data:
            await asyncio.sleep(0.1)
            yield item
    
    async def _nested_async_generator(self, base: str) -> AsyncGenerator[str, None]:
        for i in range(3):
            yield f"{base}_{i}"
    
    async def _process_item(self, item: str) -> str:
        return item.upper()
    
    async def _should_include(self, item: str) -> bool:
        return len(item) > 0

# Complex decorators with async
def async_decorator(retry_count: int = 3):
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            for attempt in range(retry_count):
                try:
                    return await func(*args, **kwargs)
                except Exception as e:
                    if attempt == retry_count - 1:
                        raise
                    await asyncio.sleep(2 ** attempt)
        return wrapper
    return decorator

class MetaclassExample(type):
    def __new__(mcs, name, bases, namespace, **kwargs):
        # Complex metaclass with async methods
        for key, value in namespace.items():
            if asyncio.iscoroutinefunction(value):
                namespace[key] = async_decorator()(value)
        return super().__new__(mcs, name, bases, namespace)

class AsyncWithMetaclass(metaclass=MetaclassExample):
    async def complex_async_method(self) -> Optional[Union[str, int]]:
        # Complex type annotations with async
        result: List[Union[str, int]] = []
        
        # Nested async comprehension
        async_results = [
            await self._compute(x) 
            async for x in self._async_range(10)
            if x % 2 == 0
        ]
        
        return async_results[0] if async_results else None
    
    async def _async_range(self, n: int) -> AsyncGenerator[int, None]:
        for i in range(n):
            await asyncio.sleep(0.01)
            yield i
    
    async def _compute(self, x: int) -> Union[str, int]:
        return str(x) if x > 5 else x
"#;
        
        let file_path = create_test_file(content, "py");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse async generator comprehensions");
    }

    #[test]
    fn test_python_metaclass_inheritance_chains() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
from typing import Type, Any, Dict, Tuple
from abc import ABCMeta, abstractmethod

class SingletonMeta(type):
    """Metaclass that creates singleton instances"""
    _instances: Dict[Type, Any] = {}
    
    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]

class ValidatorMeta(type):
    """Metaclass that validates class definitions"""
    
    def __new__(mcs, name: str, bases: Tuple[Type, ...], namespace: Dict[str, Any], **kwargs):
        # Complex validation logic
        required_methods = kwargs.get('required_methods', [])
        for method_name in required_methods:
            if method_name not in namespace:
                raise TypeError(f"Class {name} must implement {method_name}")
        
        # Auto-generate property methods
        for attr_name, attr_value in list(namespace.items()):
            if attr_name.startswith('_') and not attr_name.startswith('__'):
                property_name = attr_name[1:]  # Remove leading underscore
                namespace[f'get_{property_name}'] = lambda self, attr=attr_name: getattr(self, attr)
                namespace[f'set_{property_name}'] = lambda self, value, attr=attr_name: setattr(self, attr, value)
        
        return super().__new__(mcs, name, bases, namespace)

class CombinedMeta(SingletonMeta, ValidatorMeta, ABCMeta):
    """Multiple inheritance in metaclasses"""
    
    def __new__(mcs, name: str, bases: Tuple[Type, ...], namespace: Dict[str, Any], **kwargs):
        # Call all parent metaclass __new__ methods
        cls = super().__new__(mcs, name, bases, namespace, **kwargs)
        
        # Add combined functionality
        cls._metaclass_chain = [SingletonMeta, ValidatorMeta, ABCMeta]
        
        return cls

# Complex inheritance chain with metaclass
class BaseService(metaclass=CombinedMeta, required_methods=['process']):
    """Base service class with metaclass validation"""
    
    def __init__(self, name: str):
        self.name = name
        self._status = "initialized"
    
    @abstractmethod
    def process(self) -> Any:
        pass

class DataProcessor(BaseService):
    """Concrete implementation with complex inheritance"""
    
    def __init__(self, name: str, data_source: str):
        super().__init__(name)
        self.data_source = data_source
    
    def process(self) -> Dict[str, Any]:
        return {
            'name': self.name,
            'source': self.data_source,
            'status': self._status
        }

# Diamond inheritance problem resolution
class Mixin1:
    def method(self):
        print("Mixin1.method")
        super().method()

class Mixin2:
    def method(self):
        print("Mixin2.method")
        super().method()

class Base:
    def method(self):
        print("Base.method")

class ComplexInheritance(Mixin1, Mixin2, Base):
    """Demonstrates method resolution order complexity"""
    
    def method(self):
        print("ComplexInheritance.method")
        super().method()
    
    @classmethod
    def get_mro(cls):
        return cls.__mro__

# Nested class with metaclass
class OuterClass(metaclass=ValidatorMeta, required_methods=['outer_method']):
    
    class NestedMeta(type):
        def __new__(mcs, name, bases, namespace):
            namespace['outer_ref'] = None
            return super().__new__(mcs, name, bases, namespace)
    
    class InnerClass(metaclass=NestedMeta):
        def __init__(self, outer_instance):
            self.outer_ref = outer_instance
    
    def outer_method(self):
        return self.InnerClass(self)
"#;
        
        let file_path = create_test_file(content, "py");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse metaclass inheritance chains");
    }

    #[test]
    fn test_python_decorator_stacking_edge_cases() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
import functools
from typing import Callable, Any, TypeVar, Generic
from dataclasses import dataclass

F = TypeVar('F', bound=Callable[..., Any])

def timing_decorator(func: F) -> F:
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        import time
        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        print(f"{func.__name__} took {end - start:.4f} seconds")
        return result
    return wrapper

def retry_decorator(max_attempts: int = 3):
    def decorator(func: F) -> F:
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            for attempt in range(max_attempts):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    if attempt == max_attempts - 1:
                        raise
                    print(f"Attempt {attempt + 1} failed: {e}")
        return wrapper
    return decorator

def cache_decorator(maxsize: int = 128):
    def decorator(func: F) -> F:
        cache = {}
        
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            # Create cache key from args and kwargs
            key = str(args) + str(sorted(kwargs.items()))
            
            if key in cache:
                return cache[key]
            
            result = func(*args, **kwargs)
            
            if len(cache) >= maxsize:
                # Simple LRU: remove first item
                cache.pop(next(iter(cache)))
            
            cache[key] = result
            return result
        
        wrapper.cache = cache
        wrapper.cache_clear = lambda: cache.clear()
        return wrapper
    return decorator

def validation_decorator(*validators):
    def decorator(func: F) -> F:
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            for validator in validators:
                validator(*args, **kwargs)
            return func(*args, **kwargs)
        return wrapper
    return decorator

# Validator functions
def positive_numbers_validator(*args, **kwargs):
    for arg in args:
        if isinstance(arg, (int, float)) and arg <= 0:
            raise ValueError(f"Expected positive number, got {arg}")

def non_empty_strings_validator(*args, **kwargs):
    for arg in args:
        if isinstance(arg, str) and len(arg) == 0:
            raise ValueError("String arguments cannot be empty")

# Complex decorator stacking
class MathOperations:
    
    @timing_decorator
    @retry_decorator(max_attempts=3)
    @cache_decorator(maxsize=256)
    @validation_decorator(positive_numbers_validator, non_empty_strings_validator)
    def complex_calculation(self, x: float, y: float, operation: str = "multiply") -> float:
        """Method with multiple stacked decorators"""
        if operation == "multiply":
            return x * y
        elif operation == "divide":
            if y == 0:
                raise ZeroDivisionError("Cannot divide by zero")
            return x / y
        elif operation == "power":
            return x ** y
        else:
            raise ValueError(f"Unsupported operation: {operation}")

# Property decorators with complex logic
class ComplexProperties:
    def __init__(self):
        self._value = 0
        self._access_count = 0
    
    @property
    def tracked_value(self) -> int:
        self._access_count += 1
        return self._value
    
    @tracked_value.setter
    def tracked_value(self, value: int):
        if not isinstance(value, int):
            raise TypeError("Value must be integer")
        if value < 0:
            raise ValueError("Value must be non-negative")
        self._value = value
    
    @tracked_value.deleter
    def tracked_value(self):
        print(f"Deleting value {self._value} (accessed {self._access_count} times)")
        self._value = 0
        self._access_count = 0

# Class decorators
def singleton(cls):
    instances = {}
    
    @functools.wraps(cls)
    def get_instance(*args, **kwargs):
        if cls not in instances:
            instances[cls] = cls(*args, **kwargs)
        return instances[cls]
    
    return get_instance

@singleton
@dataclass
class SingletonConfig:
    """Class with multiple decorators"""
    host: str = "localhost"
    port: int = 8080
    debug: bool = False
    
    def get_connection_string(self) -> str:
        return f"{self.host}:{self.port}"

# Nested decorators with closures
def create_context_decorator(context_name: str):
    def decorator(func: F) -> F:
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            print(f"Entering {context_name}")
            try:
                result = func(*args, **kwargs)
                print(f"Exiting {context_name} successfully")
                return result
            except Exception as e:
                print(f"Exiting {context_name} with error: {e}")
                raise
        return wrapper
    return decorator

@create_context_decorator("database_operation")
@timing_decorator
def complex_database_operation(query: str) -> dict:
    # Simulate database operation
    import time
    time.sleep(0.1)
    return {"query": query, "rows": 42}
"#;
        
        let file_path = create_test_file(content, "py");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse decorator stacking edge cases");
    }

    #[test]
    fn test_python_walrus_operator_complex() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
import re
from typing import List, Optional, Dict, Any

def complex_walrus_examples():
    # Walrus operator in list comprehensions
    data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    
    # Complex nested walrus operators
    results = [
        y 
        for x in data 
        if (y := x * 2) > 5 and (z := y + 1) % 3 == 0
        for final in [y + z] 
        if final > 10
    ]
    
    # Walrus in while loop conditions
    lines = ["line1", "line2", "line3", ""]
    iterator = iter(lines)
    while (line := next(iterator, None)) is not None:
        if (processed := line.strip()) and len(processed) > 0:
            print(f"Processing: {processed}")
    
    # Walrus with regex matching
    text = "The phone number is 123-456-7890 and email is test@example.com"
    patterns = [
        r'\d{3}-\d{3}-\d{4}',  # phone
        r'\w+@\w+\.\w+',       # email
    ]
    
    matches = [
        match.group()
        for pattern in patterns
        if (match := re.search(pattern, text))
    ]
    
    # Complex nested dictionary comprehension with walrus
    nested_data = {
        'users': [
            {'name': 'Alice', 'age': 30, 'city': 'New York'},
            {'name': 'Bob', 'age': 25, 'city': 'San Francisco'},
            {'name': 'Charlie', 'age': 35, 'city': 'Chicago'},
        ]
    }
    
    processed_users = {
        (user_id := user['name'].lower()): {
            'display_name': user['name'],
            'age_group': 'senior' if (age := user['age']) > 30 else 'junior',
            'location': city.replace(' ', '_').lower() if (city := user['city']) else 'unknown'
        }
        for user in nested_data['users']
        if age > 20  # Use age from walrus operator above
    }
    
    return results, matches, processed_users

class ComplexWalrusClass:
    def __init__(self, data: List[Dict[str, Any]]):
        self.data = data
    
    def filter_and_transform(self, min_value: int = 0) -> List[Dict[str, Any]]:
        # Walrus operator in class method with complex logic
        return [
            {
                'original': item,
                'transformed': transformed,
                'category': (
                    'high' if transformed > 100 
                    else 'medium' if transformed > 50 
                    else 'low'
                )
            }
            for item in self.data
            if isinstance(value := item.get('value', 0), (int, float))
            and value > min_value
            and (transformed := self._transform_value(value)) is not None
        ]
    
    def _transform_value(self, value: float) -> Optional[float]:
        # Complex transformation logic
        if value <= 0:
            return None
        return value * 2.5 + 10

def advanced_walrus_patterns():
    # Walrus in exception handling
    errors = []
    
    for i in range(5):
        try:
            result = 10 / i
        except ZeroDivisionError as e:
            if (error_msg := str(e)) and error_msg not in errors:
                errors.append(error_msg)
    
    # Walrus with lambda functions
    data = [1, 2, 3, 4, 5]
    
    # Complex lambda with walrus operator
    process_func = lambda x: (
        processed if (processed := x * 2) > 5 else None
    )
    
    filtered_results = list(filter(None, map(process_func, data)))
    
    # Walrus in generator expressions
    def number_generator():
        for i in range(100):
            if (square := i ** 2) % 7 == 0 and (cube := i ** 3) % 11 == 0:
                yield {'number': i, 'square': square, 'cube': cube}
    
    special_numbers = list(number_generator())
    
    # Complex walrus in nested functions
    def outer_function(threshold: int):
        def inner_function(values: List[int]) -> List[int]:
            return [
                val
                for x in values
                if (val := x * 2) > threshold
                and (check := val % 3) == 0
            ]
        
        return inner_function
    
    processor = outer_function(10)
    processed = processor([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    
    return filtered_results, special_numbers, processed

# Walrus operator in class definitions
class WalrusInClass:
    # Class variable with walrus operator (Python 3.8+ feature)
    class_var = (default_value := "initialized") + "_class"
    
    def __init__(self):
        # Instance variable with walrus in initialization
        self.instance_var = (
            computed_value if (computed_value := self._compute_initial()) 
            else "fallback"
        )
    
    def _compute_initial(self) -> Optional[str]:
        return "computed" if True else None
    
    @property
    def complex_property(self) -> str:
        # Property with walrus operator
        return (
            f"processed_{value}"
            if (value := self.instance_var) and len(value) > 0
            else "empty"
        )

# Walrus with async operations (conceptual - would need async framework)
def simulate_async_walrus():
    # Simulating async-like patterns with walrus
    async_results = []
    
    for task_id in range(5):
        # Simulate checking if async task is complete
        if (result := f"task_{task_id}_result") and len(result) > 0:
            async_results.append({
                'id': task_id,
                'result': result,
                'status': 'completed' if (check := len(result)) > 10 else 'pending'
            })
    
    return async_results
"#;
        
        let file_path = create_test_file(content, "py");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse walrus operator complex usage");
    }

    #[test]
    fn test_python_type_hint_union_generics() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
from typing import (
    Union, Optional, List, Dict, Tuple, Set, FrozenSet,
    TypeVar, Generic, Callable, Any, Type, ClassVar,
    Literal, Final, Protocol, runtime_checkable,
    overload, TypedDict, NamedTuple
)
from typing_extensions import ParamSpec, Concatenate
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum

# Complex TypeVar definitions
T = TypeVar('T')
U = TypeVar('U', bound='Comparable')
V = TypeVar('V', int, str, float)
P = ParamSpec('P')

# Protocol definition with complex type hints
@runtime_checkable
class Comparable(Protocol):
    def __lt__(self: T, other: T) -> bool: ...
    def __le__(self: T, other: T) -> bool: ...
    def __gt__(self: T, other: T) -> bool: ...
    def __ge__(self: T, other: T) -> bool: ...

# Complex generic class with multiple type parameters
class GenericContainer(Generic[T, U], ABC):
    def __init__(self, items: List[T], comparator: Callable[[T], U]):
        self._items: List[T] = items
        self._comparator: Callable[[T], U] = comparator
        self._cache: Dict[T, U] = {}
    
    @abstractmethod
    def process(self, item: T) -> U:
        """Process a single item"""
        pass
    
    def batch_process(
        self, 
        items: Union[List[T], Tuple[T, ...], Set[T]]
    ) -> Dict[T, Union[U, Exception]]:
        """Complex union types in method signature"""
        results: Dict[T, Union[U, Exception]] = {}
        
        for item in items:
            try:
                results[item] = self.process(item)
            except Exception as e:
                results[item] = e
        
        return results
    
    @overload
    def get_item(self, index: int) -> T: ...
    
    @overload
    def get_item(self, index: slice) -> List[T]: ...
    
    def get_item(self, index: Union[int, slice]) -> Union[T, List[T]]:
        """Method overloading with union types"""
        if isinstance(index, int):
            return self._items[index]
        else:
            return self._items[index]

# TypedDict with complex structure
class UserData(TypedDict, total=False):
    name: str
    age: int
    email: Optional[str]
    preferences: Dict[str, Union[str, int, bool]]
    tags: List[str]
    metadata: Optional[Dict[str, Any]]

class Address(TypedDict):
    street: str
    city: str
    country: str
    postal_code: Union[str, int]

class ComplexUserProfile(UserData):
    """TypedDict inheritance"""
    addresses: List[Address]
    is_verified: bool
    account_type: Literal['free', 'premium', 'enterprise']

# NamedTuple with complex types
class ProcessingResult(NamedTuple):
    success: bool
    data: Optional[Union[str, Dict[str, Any], List[Any]]]
    errors: List[Union[str, Exception]]
    metadata: Dict[str, Union[int, float, str]]

# Complex function type annotations
def higher_order_function(
    func: Callable[[T], U],
    items: Union[List[T], Tuple[T, ...]],
    error_handler: Optional[Callable[[Exception], U]] = None
) -> Callable[[bool], Union[List[U], Dict[T, U]]]:
    """Function returning function with complex type annotations"""
    
    def processor(as_dict: bool = False) -> Union[List[U], Dict[T, U]]:
        if as_dict:
            result: Dict[T, U] = {}
            for item in items:
                try:
                    result[item] = func(item)
                except Exception as e:
                    if error_handler:
                        result[item] = error_handler(e)
                    else:
                        raise
            return result
        else:
            result_list: List[U] = []
            for item in items:
                try:
                    result_list.append(func(item))
                except Exception as e:
                    if error_handler:
                        result_list.append(error_handler(e))
                    else:
                        raise
            return result_list
    
    return processor

# Async function with complex type hints
async def async_processor(
    data_source: Union[str, Dict[str, Any], Callable[[], Any]],
    transformers: List[Callable[[Any], Union[Any, Exception]]],
    batch_size: int = 100
) -> Tuple[List[Any], List[Exception], Dict[str, Union[int, float]]]:
    """Async function with complex return type"""
    import asyncio
    
    successes: List[Any] = []
    errors: List[Exception] = []
    stats: Dict[str, Union[int, float]] = {'processed': 0, 'success_rate': 0.0}
    
    # Complex type handling
    if callable(data_source):
        raw_data = data_source()
    elif isinstance(data_source, str):
        raw_data = [data_source]
    else:
        raw_data = list(data_source.values()) if isinstance(data_source, dict) else [data_source]
    
    # Batch processing
    for i in range(0, len(raw_data), batch_size):
        batch = raw_data[i:i + batch_size]
        
        for item in batch:
            for transformer in transformers:
                try:
                    result = transformer(item)
                    if not isinstance(result, Exception):
                        successes.append(result)
                        stats['processed'] += 1
                    else:
                        errors.append(result)
                except Exception as e:
                    errors.append(e)
        
        # Simulate async delay
        await asyncio.sleep(0.01)
    
    # Calculate success rate
    total_processed = stats['processed'] + len(errors)
    stats['success_rate'] = stats['processed'] / total_processed if total_processed > 0 else 0.0
    
    return successes, errors, stats

# Class with complex type annotations and generics
@dataclass
class DataProcessor(Generic[T, U]):
    """Data processor with generic types and complex annotations"""
    
    input_validator: Callable[[T], bool]
    transformer: Callable[[T], U]
    output_serializer: Callable[[U], Union[str, bytes, Dict[str, Any]]]
    
    # Class variables with type annotations
    DEFAULT_BATCH_SIZE: ClassVar[int] = 100
    SUPPORTED_FORMATS: ClassVar[FrozenSet[str]] = frozenset(['json', 'xml', 'csv'])
    
    # Final variables
    processor_id: Final[str] = "generic_processor_v1"
    
    def process_batch(
        self, 
        items: Union[List[T], Tuple[T, ...], Set[T]],
        format_type: Literal['json', 'xml', 'csv'] = 'json'
    ) -> Tuple[
        List[Union[str, bytes, Dict[str, Any]]],  # Successful outputs
        List[Tuple[T, Exception]],                # Failed items with errors
        Dict[Literal['total', 'success', 'failed'], int]  # Statistics
    ]:
        """Process batch with complex return type annotation"""
        
        outputs: List[Union[str, bytes, Dict[str, Any]]] = []
        failures: List[Tuple[T, Exception]] = []
        stats: Dict[Literal['total', 'success', 'failed'], int] = {
            'total': 0,
            'success': 0,
            'failed': 0
        }
        
        for item in items:
            stats['total'] += 1
            
            try:
                # Validate input
                if not self.input_validator(item):
                    raise ValueError(f"Input validation failed for item: {item}")
                
                # Transform
                transformed = self.transformer(item)
                
                # Serialize
                serialized = self.output_serializer(transformed)
                outputs.append(serialized)
                stats['success'] += 1
                
            except Exception as e:
                failures.append((item, e))
                stats['failed'] += 1
        
        return outputs, failures, stats

# Enum with complex type integration
class ProcessingStatus(Enum):
    PENDING = "pending"
    PROCESSING = "processing" 
    COMPLETED = "completed"
    FAILED = "failed"

ProcessingCallback = Callable[[ProcessingStatus, Optional[Any], Optional[Exception]], None]

def create_processor_with_callback(
    callback: ProcessingCallback
) -> Callable[[Any], Union[Any, None]]:
    """Function factory with complex callback type"""
    
    def processor(data: Any) -> Union[Any, None]:
        callback(ProcessingStatus.PROCESSING, None, None)
        
        try:
            # Simulate processing
            result = f"processed_{data}"
            callback(ProcessingStatus.COMPLETED, result, None)
            return result
        except Exception as e:
            callback(ProcessingStatus.FAILED, None, e)
            return None
    
    return processor
"#;
        
        let file_path = create_test_file(content, "py");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse complex type hint union generics");
    }

    // JAVASCRIPT EDGE CASE TESTS
    #[test]
    fn test_javascript_proxy_trap_combinations() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
// Complex Proxy with multiple trap combinations
const createComplexProxy = (target) => {
  const accessLog = [];
  const modificationLog = [];
  
  return new Proxy(target, {
    // Property access trap
    get(obj, prop, receiver) {
      accessLog.push({ type: 'get', prop, timestamp: Date.now() });
      
      if (typeof obj[prop] === 'function') {
        return new Proxy(obj[prop], {
          apply(fn, thisArg, args) {
            accessLog.push({ type: 'method_call', prop, args, timestamp: Date.now() });
            return Reflect.apply(fn, thisArg, args);
          }
        });
      }
      
      return Reflect.get(obj, prop, receiver);
    },
    
    // Property setting trap
    set(obj, prop, value, receiver) {
      const oldValue = obj[prop];
      modificationLog.push({ 
        type: 'set', 
        prop, 
        oldValue, 
        newValue: value, 
        timestamp: Date.now() 
      });
      
      // Validate property names
      if (typeof prop === 'string' && prop.startsWith('_')) {
        throw new Error(`Cannot set private property: ${prop}`);
      }
      
      return Reflect.set(obj, prop, value, receiver);
    },
    
    // Property deletion trap
    deleteProperty(obj, prop) {
      if (typeof prop === 'string' && prop.startsWith('_')) {
        throw new Error(`Cannot delete private property: ${prop}`);
      }
      
      modificationLog.push({ type: 'delete', prop, timestamp: Date.now() });
      return Reflect.deleteProperty(obj, prop);
    },
    
    // Object key enumeration trap
    ownKeys(obj) {
      const keys = Reflect.ownKeys(obj);
      // Filter out private properties from enumeration
      return keys.filter(key => !String(key).startsWith('_'));
    },
    
    // Property descriptor trap
    getOwnPropertyDescriptor(obj, prop) {
      if (typeof prop === 'string' && prop.startsWith('_')) {
        return undefined; // Hide private properties
      }
      return Reflect.getOwnPropertyDescriptor(obj, prop);
    },
    
    // Prototype trap
    getPrototypeOf(obj) {
      accessLog.push({ type: 'prototype_access', timestamp: Date.now() });
      return Reflect.getPrototypeOf(obj);
    },
    
    // Property definition trap
    defineProperty(obj, prop, descriptor) {
      if (typeof prop === 'string' && prop.startsWith('_')) {
        throw new Error(`Cannot define private property: ${prop}`);
      }
      
      modificationLog.push({ 
        type: 'define', 
        prop, 
        descriptor, 
        timestamp: Date.now() 
      });
      
      return Reflect.defineProperty(obj, prop, descriptor);
    },
    
    // Extensibility trap
    preventExtensions(obj) {
      modificationLog.push({ type: 'prevent_extensions', timestamp: Date.now() });
      return Reflect.preventExtensions(obj);
    },
    
    // Has property trap
    has(obj, prop) {
      accessLog.push({ type: 'has', prop, timestamp: Date.now() });
      
      if (typeof prop === 'string' && prop.startsWith('_')) {
        return false; // Hide private properties
      }
      
      return Reflect.has(obj, prop);
    }
  });
};

// Nested proxy example
const createNestedProxy = (data) => {
  return new Proxy(data, {
    get(obj, prop) {
      const value = obj[prop];
      
      // Automatically wrap nested objects in proxies
      if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
        return createNestedProxy(value);
      }
      
      // Wrap arrays in proxies with additional functionality
      if (Array.isArray(value)) {
        return new Proxy(value, {
          get(arr, index) {
            if (index === 'sum') {
              return () => arr.reduce((a, b) => a + b, 0);
            }
            if (index === 'average') {
              return () => arr.length > 0 ? arr.reduce((a, b) => a + b, 0) / arr.length : 0;
            }
            return arr[index];
          }
        });
      }
      
      return value;
    }
  });
};

// Proxy for method interception and modification
class ProxyMethodInterceptor {
  constructor(target) {
    this.target = target;
    this.methodCallCounts = new Map();
    
    return new Proxy(this, {
      get(interceptor, prop) {
        if (prop in interceptor.target) {
          const targetProp = interceptor.target[prop];
          
          if (typeof targetProp === 'function') {
            return new Proxy(targetProp, {
              apply(fn, thisArg, args) {
                // Track method calls
                const count = interceptor.methodCallCounts.get(prop) || 0;
                interceptor.methodCallCounts.set(prop, count + 1);
                
                // Add timing
                const start = performance.now();
                const result = Reflect.apply(fn, thisArg, args);
                const end = performance.now();
                
                console.log(`Method ${prop} called ${count + 1} times, took ${end - start}ms`);
                
                return result;
              }
            });
          }
          
          return targetProp;
        }
        
        return interceptor[prop];
      }
    });
  }
  
  getMethodStats() {
    return Object.fromEntries(this.methodCallCounts);
  }
}

// Proxy for dynamic property creation
const createDynamicProxy = () => {
  const data = {};
  
  return new Proxy(data, {
    get(obj, prop) {
      if (!(prop in obj)) {
        // Dynamically create properties based on naming conventions
        if (String(prop).startsWith('get')) {
          const fieldName = String(prop).slice(3).toLowerCase();
          return () => obj[fieldName];
        }
        
        if (String(prop).startsWith('set')) {
          const fieldName = String(prop).slice(3).toLowerCase();
          return (value) => { obj[fieldName] = value; };
        }
        
        if (String(prop).endsWith('Count')) {
          const fieldName = String(prop).slice(0, -5);
          return Array.isArray(obj[fieldName]) ? obj[fieldName].length : 0;
        }
      }
      
      return obj[prop];
    },
    
    set(obj, prop, value) {
      // Auto-convert strings to numbers for numeric properties
      if (String(prop).includes('Number') || String(prop).includes('Count')) {
        if (typeof value === 'string' && !isNaN(value)) {
          value = Number(value);
        }
      }
      
      obj[prop] = value;
      return true;
    }
  });
};

// Complex proxy chain
const createProxyChain = (initialValue) => {
  // Validation proxy
  const validationProxy = new Proxy(initialValue, {
    set(obj, prop, value) {
      if (typeof value === 'string' && value.length === 0) {
        throw new Error(`Empty string not allowed for property ${prop}`);
      }
      obj[prop] = value;
      return true;
    }
  });
  
  // Transformation proxy
  const transformationProxy = new Proxy(validationProxy, {
    get(obj, prop) {
      const value = obj[prop];
      
      // Auto-capitalize string properties
      if (typeof value === 'string' && String(prop).endsWith('Name')) {
        return value.charAt(0).toUpperCase() + value.slice(1);
      }
      
      return value;
    }
  });
  
  // Logging proxy (outermost)
  const loggingProxy = new Proxy(transformationProxy, {
    get(obj, prop) {
      console.log(`Accessing property: ${prop}`);
      return obj[prop];
    },
    
    set(obj, prop, value) {
      console.log(`Setting property ${prop} to:`, value);
      obj[prop] = value;
      return true;
    }
  });
  
  return loggingProxy;
};

// Proxy with Symbol handling
const createSymbolAwareProxy = (target) => {
  const privateData = new Map();
  
  return new Proxy(target, {
    get(obj, prop) {
      // Handle Symbol properties specially
      if (typeof prop === 'symbol') {
        return privateData.get(prop);
      }
      
      // Handle well-known symbols
      if (prop === Symbol.iterator) {
        return function* () {
          for (const key in obj) {
            yield [key, obj[key]];
          }
        };
      }
      
      if (prop === Symbol.toPrimitive) {
        return (hint) => {
          if (hint === 'number') {
            return Object.keys(obj).length;
          }
          if (hint === 'string') {
            return JSON.stringify(obj);
          }
          return Object.keys(obj).length;
        };
      }
      
      return obj[prop];
    },
    
    set(obj, prop, value) {
      if (typeof prop === 'symbol') {
        privateData.set(prop, value);
        return true;
      }
      
      obj[prop] = value;
      return true;
    },
    
    has(obj, prop) {
      if (typeof prop === 'symbol') {
        return privateData.has(prop);
      }
      
      return prop in obj;
    },
    
    ownKeys(obj) {
      return [...Reflect.ownKeys(obj), ...privateData.keys()];
    }
  });
};

// Usage examples
const testProxy = () => {
  const obj = { name: 'test', value: 42 };
  const proxied = createComplexProxy(obj);
  
  // Test various operations
  console.log(proxied.name); // Triggers get trap
  proxied.newProp = 'new value'; // Triggers set trap
  delete proxied.value; // Triggers deleteProperty trap
  
  const dynamic = createDynamicProxy();
  dynamic.userName = 'john';
  console.log(dynamic.getUserName()); // Dynamically created getter
  
  const chained = createProxyChain({ firstName: 'jane', userCount: '5' });
  console.log(chained.firstName); // Capitalized due to transformation proxy
  
  return { proxied, dynamic, chained };
};

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = {
    createComplexProxy,
    createNestedProxy,
    ProxyMethodInterceptor,
    createDynamicProxy,
    createProxyChain,
    createSymbolAwareProxy,
    testProxy
  };
}
"#;
        
        let file_path = create_test_file(content, "js");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse JavaScript proxy trap combinations");
    }

    #[test]
    fn test_javascript_es2022_class_fields() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
// ES2022 Class Features: Private fields, static blocks, etc.

class ModernClass {
  // Public class fields
  publicField = 'public value';
  publicNumber = 42;
  
  // Private class fields
  #privateField = 'private value';
  #privateNumber = 0;
  #privateArray = [];
  
  // Static public fields
  static staticPublicField = 'static public';
  static staticCounter = 0;
  
  // Static private fields
  static #staticPrivateField = 'static private';
  static #instances = new Set();
  
  // Static initialization block
  static {
    console.log('Static block executed');
    this.#staticPrivateField = this.#staticPrivateField.toUpperCase();
    this.staticCounter = Math.floor(Math.random() * 100);
    
    // Complex initialization logic
    const config = {
      maxInstances: 10,
      enableLogging: true
    };
    
    this.#instances = new Set();
    this.config = config;
  }
  
  // Constructor with private field initialization
  constructor(name, options = {}) {
    // Increment static counter
    ModernClass.staticCounter++;
    ModernClass.#instances.add(this);
    
    this.name = name;
    this.#privateNumber = options.initialValue || 0;
    this.#privateArray = options.items || [];
    
    // Private field validation
    if (this.#privateNumber < 0) {
      throw new Error('Initial value cannot be negative');
    }
  }
  
  // Public method accessing private fields
  getValue() {
    return this.#privateNumber;
  }
  
  setValue(value) {
    if (typeof value !== 'number') {
      throw new TypeError('Value must be a number');
    }
    this.#privateNumber = value;
  }
  
  // Private method
  #calculateInternalValue() {
    return this.#privateNumber * 2 + this.#privateArray.length;
  }
  
  // Method using private method
  getProcessedValue() {
    return this.#calculateInternalValue();
  }
  
  // Private getter
  get #computedPrivateValue() {
    return this.#privateField + '_computed';
  }
  
  // Private setter
  set #computedPrivateValue(value) {
    this.#privateField = value.replace('_computed', '');
  }
  
  // Public method using private getter/setter
  updatePrivateField(newValue) {
    this.#computedPrivateValue = newValue + '_computed';
    return this.#computedPrivateValue;
  }
  
  // Static method accessing static private fields
  static getInstanceCount() {
    return this.#instances.size;
  }
  
  static #validateInstance(instance) {
    return instance instanceof ModernClass;
  }
  
  // Static method using private static method
  static addInstance(instance) {
    if (this.#validateInstance(instance)) {
      this.#instances.add(instance);
      return true;
    }
    return false;
  }
  
  // Method with private field in complex expression
  complexCalculation() {
    const multiplier = this.#privateNumber > 10 ? 2 : 1;
    const baseValue = this.#calculateInternalValue();
    
    return {
      base: baseValue,
      multiplied: baseValue * multiplier,
      array_length: this.#privateArray.length,
      has_private_data: this.#privateField.length > 0
    };
  }
  
  // Destructor-like cleanup
  dispose() {
    ModernClass.#instances.delete(this);
    this.#privateArray.length = 0;
    this.#privateNumber = 0;
    this.#privateField = '';
  }
}

// Extended class with additional private fields
class ExtendedModernClass extends ModernClass {
  // Extended class can have its own private fields
  #extendedPrivateField = 'extended private';
  
  // Static fields in extended class
  static extendedStaticField = 'extended static';
  
  // Static block in extended class
  static {
    console.log('Extended class static block');
    this.extendedStaticField = this.extendedStaticField.toUpperCase();
  }
  
  constructor(name, options = {}, extendedOptions = {}) {
    super(name, options);
    
    this.#extendedPrivateField = extendedOptions.extendedValue || 'default extended';
  }
  
  // Method accessing both parent and child private fields
  getCombinedValue() {
    // Can access own private fields
    const extendedValue = this.#extendedPrivateField;
    // Cannot directly access parent's private fields, must use public methods
    const parentValue = this.getValue();
    
    return `${extendedValue}_${parentValue}`;
  }
  
  // Override method
  complexCalculation() {
    const parentResult = super.complexCalculation();
    
    return {
      ...parentResult,
      extended_data: this.#extendedPrivateField,
      is_extended: true
    };
  }
}

// Mixin pattern with private fields
const PrivateFieldMixin = (SuperClass) => {
  return class extends SuperClass {
    // Private fields in mixin
    #mixinPrivateField = 'mixin private';
    
    // Public interface for mixin functionality
    getMixinData() {
      return this.#mixinPrivateField;
    }
    
    setMixinData(value) {
      this.#mixinPrivateField = value;
    }
    
    #mixinPrivateMethod() {
      return this.#mixinPrivateField.toUpperCase();
    }
    
    getProcessedMixinData() {
      return this.#mixinPrivateMethod();
    }
  };
};

// Using mixin with private fields
class MixedClass extends PrivateFieldMixin(ModernClass) {
  #ownPrivateField = 'own private';
  
  constructor(name, options = {}) {
    super(name, options);
    this.#ownPrivateField = options.ownValue || 'default own';
  }
  
  getAllPrivateData() {
    return {
      own: this.#ownPrivateField,
      mixin: this.getMixinData(), // Use public interface
      parent_processed: this.getProcessedValue() // Use parent public method
    };
  }
}

// Class with complex private field interactions
class PrivateFieldInteractions {
  #counter = 0;
  #callbacks = [];
  #isActive = false;
  
  static #globalCounter = 0;
  
  // Private field with function value
  #processor = (value) => {
    return value * this.#counter + PrivateFieldInteractions.#globalCounter;
  };
  
  constructor() {
    PrivateFieldInteractions.#globalCounter++;
    this.#counter = PrivateFieldInteractions.#globalCounter;
  }
  
  // Method returning function that closes over private fields
  createCallback() {
    return (input) => {
      this.#counter++;
      const processed = this.#processor(input);
      this.#callbacks.push({ input, processed, timestamp: Date.now() });
      return processed;
    };
  }
  
  // Private field used in array operations
  #processArray(array) {
    return array
      .map(item => this.#processor(item))
      .filter(result => result > this.#counter);
  }
  
  batchProcess(items) {
    if (!this.#isActive) {
      throw new Error('Processor is not active');
    }
    
    return this.#processArray(items);
  }
  
  activate() {
    this.#isActive = true;
    this.#callbacks = [];
  }
  
  deactivate() {
    this.#isActive = false;
    return [...this.#callbacks]; // Return copy of callback history
  }
  
  // Static method for accessing static private field
  static getGlobalStats() {
    return {
      totalInstances: this.#globalCounter,
      averageProcessedItems: 0 // Would need instance tracking for real calculation
    };
  }
}

// Usage examples and tests
function testModernClassFeatures() {
  // Test basic private fields
  const instance1 = new ModernClass('test1', { initialValue: 10 });
  console.log(instance1.getValue()); // 10
  
  // Private fields are not accessible from outside
  // console.log(instance1.#privateField); // SyntaxError
  
  // Test static features
  console.log(ModernClass.getInstanceCount()); // 1
  console.log(ModernClass.staticCounter); // Should be > 0
  
  // Test extended class
  const extended = new ExtendedModernClass('test2', { initialValue: 20 });
  console.log(extended.getCombinedValue());
  
  // Test mixin
  const mixed = new MixedClass('test3');
  console.log(mixed.getAllPrivateData());
  
  // Test complex interactions
  const interactions = new PrivateFieldInteractions();
  interactions.activate();
  
  const callback = interactions.createCallback();
  const results = [1, 2, 3].map(callback);
  console.log(results);
  
  const history = interactions.deactivate();
  console.log(history);
  
  return {
    instance1,
    extended,
    mixed,
    interactions,
    staticStats: PrivateFieldInteractions.getGlobalStats()
  };
}

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = {
    ModernClass,
    ExtendedModernClass,
    PrivateFieldMixin,
    MixedClass,
    PrivateFieldInteractions,
    testModernClassFeatures
  };
}
"#;
        
        let file_path = create_test_file(content, "js");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse ES2022 class fields");
    }

    #[test]
    fn test_javascript_dynamic_import_edge_cases() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
// Dynamic Import Edge Cases and Advanced Patterns

class ModuleLoader {
  constructor() {
    this.moduleCache = new Map();
    this.loadingPromises = new Map();
    this.errorCounts = new Map();
  }

  // Basic dynamic import with error handling
  async loadModule(modulePath) {
    try {
      const module = await import(modulePath);
      return module;
    } catch (error) {
      console.error(`Failed to load module ${modulePath}:`, error);
      throw error;
    }
  }

  // Dynamic import with caching
  async loadModuleWithCache(modulePath) {
    if (this.moduleCache.has(modulePath)) {
      return this.moduleCache.get(modulePath);
    }

    // Avoid loading the same module multiple times concurrently
    if (this.loadingPromises.has(modulePath)) {
      return this.loadingPromises.get(modulePath);
    }

    const loadingPromise = import(modulePath)
      .then(module => {
        this.moduleCache.set(modulePath, module);
        this.loadingPromises.delete(modulePath);
        return module;
      })
      .catch(error => {
        this.loadingPromises.delete(modulePath);
        throw error;
      });

    this.loadingPromises.set(modulePath, loadingPromise);
    return loadingPromise;
  }

  // Conditional dynamic imports based on feature detection
  async loadFeatureModule(featureName) {
    const featureMap = {
      'webgl': () => this.hasWebGLSupport() ? './modules/webgl-renderer.js' : './modules/canvas-renderer.js',
      'workers': () => typeof Worker !== 'undefined' ? './modules/worker-manager.js' : './modules/sync-processor.js',
      'wasm': () => typeof WebAssembly !== 'undefined' ? './modules/wasm-core.js' : './modules/js-fallback.js'
    };

    const getModulePath = featureMap[featureName];
    if (!getModulePath) {
      throw new Error(`Unknown feature: ${featureName}`);
    }

    const modulePath = getModulePath();
    return this.loadModuleWithCache(modulePath);
  }

  // Dynamic import with retry logic
  async loadModuleWithRetry(modulePath, maxRetries = 3) {
    let lastError;
    
    for (let attempt = 0; attempt < maxRetries; attempt++) {
      try {
        const module = await import(modulePath);
        
        // Reset error count on success
        this.errorCounts.delete(modulePath);
        return module;
      } catch (error) {
        lastError = error;
        
        // Track error count
        const errorCount = (this.errorCounts.get(modulePath) || 0) + 1;
        this.errorCounts.set(modulePath, errorCount);
        
        // Wait before retry (exponential backoff)
        if (attempt < maxRetries - 1) {
          const delay = Math.pow(2, attempt) * 1000;
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }
    
    throw new Error(`Failed to load module ${modulePath} after ${maxRetries} attempts: ${lastError.message}`);
  }

  // Batch loading multiple modules
  async loadModulesBatch(modulePaths) {
    const loadPromises = modulePaths.map(async (path) => {
      try {
        const module = await import(path);
        return { path, module, success: true };
      } catch (error) {
        return { path, error, success: false };
      }
    });

    const results = await Promise.allSettled(loadPromises);
    
    return results.map((result, index) => {
      if (result.status === 'fulfilled') {
        return result.value;
      } else {
        return {
          path: modulePaths[index],
          error: result.reason,
          success: false
        };
      }
    });
  }

  // Load modules based on user interaction or conditions
  async loadOnDemand(trigger, moduleConfig) {
    const { condition, modulePath, fallback } = moduleConfig;
    
    // Wait for trigger event if specified
    if (trigger && typeof trigger.addEventListener === 'function') {
      return new Promise((resolve, reject) => {
        const handler = async () => {
          trigger.removeEventListener('click', handler);
          
          try {
            if (condition && !condition()) {
              if (fallback) {
                resolve(await import(fallback));
              } else {
                reject(new Error('Condition not met and no fallback provided'));
              }
              return;
            }
            
            resolve(await import(modulePath));
          } catch (error) {
            reject(error);
          }
        };
        
        trigger.addEventListener('click', handler);
      });
    }
    
    // Immediate loading with condition check
    if (condition && !condition()) {
      if (fallback) {
        return import(fallback);
      } else {
        throw new Error('Condition not met and no fallback provided');
      }
    }
    
    return import(modulePath);
  }

  // Helper methods
  hasWebGLSupport() {
    try {
      const canvas = document.createElement('canvas');
      return !!(canvas.getContext('webgl') || canvas.getContext('experimental-webgl'));
    } catch (e) {
      return false;
    }
  }
}

// Advanced dynamic import patterns
class PluginSystem {
  constructor() {
    this.plugins = new Map();
    this.pluginHooks = new Map();
  }

  // Load plugin with version checking
  async loadPlugin(pluginName, version = 'latest') {
    const pluginId = `${pluginName}@${version}`;
    
    if (this.plugins.has(pluginId)) {
      return this.plugins.get(pluginId);
    }

    try {
      // Dynamic import with computed path
      const modulePath = `./plugins/${pluginName}/${version}/index.js`;
      const pluginModule = await import(modulePath);
      
      // Validate plugin interface
      if (!pluginModule.default || typeof pluginModule.default.init !== 'function') {
        throw new Error(`Invalid plugin interface for ${pluginName}`);
      }
      
      const plugin = pluginModule.default;
      
      // Initialize plugin
      await plugin.init(this);
      
      this.plugins.set(pluginId, plugin);
      
      // Register plugin hooks
      if (plugin.hooks) {
        for (const [hookName, hookFunction] of Object.entries(plugin.hooks)) {
          if (!this.pluginHooks.has(hookName)) {
            this.pluginHooks.set(hookName, []);
          }
          this.pluginHooks.get(hookName).push(hookFunction);
        }
      }
      
      return plugin;
    } catch (error) {
      console.error(`Failed to load plugin ${pluginName}@${version}:`, error);
      throw error;
    }
  }

  // Load multiple plugins with dependency resolution
  async loadPluginsWithDependencies(pluginConfigs) {
    const loadedPlugins = new Map();
    const loadingQueue = [...pluginConfigs];
    const failedPlugins = [];
    
    // Simple dependency resolution (could be more sophisticated)
    while (loadingQueue.length > 0 && failedPlugins.length < loadingQueue.length) {
      const config = loadingQueue.shift();
      const { name, version, dependencies = [] } = config;
      
      // Check if dependencies are loaded
      const unmetDependencies = dependencies.filter(dep => !loadedPlugins.has(dep));
      
      if (unmetDependencies.length > 0) {
        // Push back to queue if dependencies not met
        loadingQueue.push(config);
        failedPlugins.push(config);
        continue;
      }
      
      try {
        const plugin = await this.loadPlugin(name, version);
        loadedPlugins.set(name, plugin);
        
        // Reset failed count on success
        const failedIndex = failedPlugins.findIndex(f => f.name === name);
        if (failedIndex !== -1) {
          failedPlugins.splice(failedIndex, 1);
        }
      } catch (error) {
        console.error(`Failed to load plugin ${name}:`, error);
      }
    }
    
    return { loaded: loadedPlugins, failed: failedPlugins };
  }
}

// Module factory pattern with dynamic imports
class ModuleFactory {
  static async createInstance(moduleType, config = {}) {
    const moduleMap = {
      'data-processor': './modules/data-processor.js',
      'chart-renderer': './modules/chart-renderer.js',
      'api-client': './modules/api-client.js',
      'cache-manager': './modules/cache-manager.js'
    };
    
    const modulePath = moduleMap[moduleType];
    if (!modulePath) {
      throw new Error(`Unknown module type: ${moduleType}`);
    }
    
    try {
      const module = await import(modulePath);
      const ModuleClass = module.default || module[moduleType];
      
      if (typeof ModuleClass !== 'function') {
        throw new Error(`Module ${moduleType} does not export a constructor`);
      }
      
      return new ModuleClass(config);
    } catch (error) {
      console.error(`Failed to create instance of ${moduleType}:`, error);
      throw error;
    }
  }
  
  // Create multiple instances with different configurations
  static async createInstances(configurations) {
    const instances = new Map();
    
    const creationPromises = configurations.map(async ({ id, type, config }) => {
      try {
        const instance = await ModuleFactory.createInstance(type, config);
        return { id, instance, success: true };
      } catch (error) {
        return { id, error, success: false };
      }
    });
    
    const results = await Promise.allSettled(creationPromises);
    
    results.forEach((result, index) => {
      const config = configurations[index];
      
      if (result.status === 'fulfilled' && result.value.success) {
        instances.set(config.id, result.value.instance);
      } else {
        console.error(`Failed to create instance ${config.id}:`, result.reason || result.value.error);
      }
    });
    
    return instances;
  }
}

// Advanced dynamic import with code splitting
class LazyComponent {
  constructor(componentPath, fallbackComponent = null) {
    this.componentPath = componentPath;
    this.fallbackComponent = fallbackComponent;
    this.loadPromise = null;
    this.loadedComponent = null;
  }
  
  async load() {
    if (this.loadedComponent) {
      return this.loadedComponent;
    }
    
    if (!this.loadPromise) {
      this.loadPromise = this._loadComponent();
    }
    
    return this.loadPromise;
  }
  
  async _loadComponent() {
    try {
      const module = await import(this.componentPath);
      this.loadedComponent = module.default || module;
      return this.loadedComponent;
    } catch (error) {
      console.error(`Failed to load component ${this.componentPath}:`, error);
      
      if (this.fallbackComponent) {
        if (typeof this.fallbackComponent === 'string') {
          const fallbackModule = await import(this.fallbackComponent);
          this.loadedComponent = fallbackModule.default || fallbackModule;
          return this.loadedComponent;
        } else {
          this.loadedComponent = this.fallbackComponent;
          return this.loadedComponent;
        }
      }
      
      throw error;
    }
  }
  
  // Preload component without waiting
  preload() {
    if (!this.loadPromise && !this.loadedComponent) {
      this.loadPromise = this._loadComponent().catch(error => {
        console.warn(`Preload failed for ${this.componentPath}:`, error);
        this.loadPromise = null; // Reset so it can be retried
      });
    }
  }
}

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = {
    ModuleLoader,
    PluginSystem,
    ModuleFactory,
    LazyComponent
  };
}
"#;
        
        let file_path = create_test_file(content, "js");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse dynamic import edge cases");
    }

    #[test]
    fn test_javascript_optional_chaining_complex() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
// Complex Optional Chaining Scenarios

class DataProcessor {
  constructor(config = {}) {
    this.config = config;
    this.cache = new Map();
    this.validators = new Map();
  }
  
  // Complex nested object access with optional chaining
  processUserData(userData) {
    // Deep nested optional chaining
    const profile = {
      name: userData?.profile?.personal?.name?.first || 'Unknown',
      email: userData?.contact?.email?.primary?.address?.toLowerCase?.() || null,
      phone: userData?.contact?.phone?.mobile?.number?.replace?.(/[^\d]/g, '') || null,
      address: {
        street: userData?.address?.current?.street?.line1 || '',
        city: userData?.address?.current?.city?.name || '',
        country: userData?.address?.current?.country?.code?.toUpperCase?.() || 'US'
      },
      preferences: {
        theme: userData?.settings?.ui?.theme?.name || 'default',
        language: userData?.settings?.locale?.language?.code || 'en',
        notifications: userData?.settings?.notifications?.email?.enabled ?? true
      }
    };
    
    // Optional chaining with array access
    const tags = userData?.metadata?.tags?.[0]?.category?.name || 'uncategorized';
    const permissions = userData?.permissions?.roles?.map?.(role => role?.name)?.filter?.(Boolean) || [];
    
    // Method chaining with optional chaining
    const lastLoginFormatted = userData?.activity?.lastLogin?.date
      ?.toISOString?.()
      ?.split?.('T')?.[0]
      ?.split?.('-')
      ?.reverse?.()
      ?.join?.('/') || 'Never';
    
    return {
      ...profile,
      tags,
      permissions,
      lastLoginFormatted,
      hasCompleteProfile: this.validateCompleteProfile(userData)
    };
  }
  
  // Optional chaining in validation logic
  validateCompleteProfile(userData) {
    const requiredFields = [
      userData?.profile?.personal?.name?.first,
      userData?.profile?.personal?.name?.last,
      userData?.contact?.email?.primary?.address,
      userData?.address?.current?.city?.name,
      userData?.address?.current?.country?.code
    ];
    
    // Complex conditional with optional chaining
    return requiredFields.every(field => field?.length > 0) &&
           userData?.profile?.personal?.dateOfBirth?.year > 1900 &&
           userData?.settings?.privacy?.dataProcessing?.consent === true;
  }
  
  // Optional chaining with dynamic property access
  getNestedValue(obj, path) {
    return path
      .split('.')
      .reduce((current, key) => current?.[key], obj);
  }
  
  // Optional chaining in array operations
  processArrayData(dataArray) {
    return dataArray
      ?.filter?.(item => item?.isActive !== false)
      ?.map?.(item => ({
        id: item?.id || `temp_${Date.now()}`,
        name: item?.name?.trim?.() || 'Unnamed',
        value: item?.data?.primary?.value || item?.data?.secondary?.value || 0,
        metadata: {
          created: item?.timestamps?.created?.toISOString?.() || new Date().toISOString(),
          updated: item?.timestamps?.updated?.toISOString?.() || null,
          version: item?.version?.number || 1
        },
        // Nested optional chaining in object creation
        computed: {
          hasValidData: item?.data?.primary?.value != null || item?.data?.secondary?.value != null,
          categoryLevel: item?.category?.level?.priority || 0,
          isRecent: item?.timestamps?.created?.getTime?.() > Date.now() - 86400000
        }
      }))
      ?.sort?.((a, b) => (b?.computed?.categoryLevel || 0) - (a?.computed?.categoryLevel || 0)) || [];
  }
  
  // Optional chaining with function calls and method references
  executeCallback(config, data) {
    // Optional chaining with function calls
    const preprocessed = config?.preprocessor?.execute?.(data) || data;
    
    // Optional chaining with method binding
    const processor = config?.processor?.process?.bind?.(config.processor);
    const processed = processor?.(preprocessed) || preprocessed;
    
    // Optional chaining with async operations (conceptual)
    const postProcessor = config?.postProcessor;
    if (postProcessor?.processAsync) {
      // Would be: return postProcessor.processAsync?.(processed);
      return processed; // Simplified for parsing test
    }
    
    return postProcessor?.process?.(processed) || processed;
  }
  
  // Complex optional chaining in error handling
  handleApiResponse(response) {
    try {
      // Optional chaining for API response validation
      const data = response?.data?.payload?.items || [];
      const hasError = response?.error?.code != null;
      
      if (hasError) {
        const errorMessage = response?.error?.message || 'Unknown error';
        const errorCode = response?.error?.code || 500;
        const errorDetails = response?.error?.details?.map?.(detail => ({
          field: detail?.field || 'unknown',
          message: detail?.message || 'No details'
        })) || [];
        
        throw new Error(`API Error ${errorCode}: ${errorMessage}. Details: ${JSON.stringify(errorDetails)}`);
      }
      
      // Process successful response with optional chaining
      return {
        data,
        metadata: {
          total: response?.metadata?.pagination?.total || data.length,
          page: response?.metadata?.pagination?.current || 1,
          hasMore: response?.metadata?.pagination?.hasNext ?? false,
          timestamp: response?.metadata?.timestamp?.toISOString?.() || new Date().toISOString()
        },
        links: {
          self: response?.links?.self?.href || null,
          next: response?.links?.next?.href || null,
          prev: response?.links?.prev?.href || null
        }
      };
    } catch (error) {
      // Optional chaining in error handling
      const originalError = error?.cause?.message || error?.message || 'Unknown error';
      const statusCode = error?.response?.status || 500;
      
      return {
        error: true,
        message: originalError,
        statusCode,
        timestamp: new Date().toISOString()
      };
    }
  }
  
  // Optional chaining with computed property names
  dynamicPropertyAccess(obj, propertyConfig) {
    const results = {};
    
    for (const [key, config] of Object.entries(propertyConfig)) {
      // Dynamic property access with optional chaining
      const path = config?.path;
      const transform = config?.transform;
      const defaultValue = config?.default;
      
      if (path) {
        // Navigate to nested property
        let value = obj;
        for (const segment of path.split('.')) {
          value = value?.[segment];
        }
        
        // Apply transformation if available
        results[key] = transform?.(value) || value || defaultValue;
      }
    }
    
    return results;
  }
  
  // Optional chaining in iterator patterns
  processIterableData(iterableData) {
    const results = [];
    
    // Optional chaining with Symbol.iterator
    const iterator = iterableData?.[Symbol.iterator]?.();
    
    if (iterator) {
      let result = iterator?.next?.();
      
      while (result && !result?.done) {
        const value = result?.value;
        const processed = this.processItem(value);
        
        if (processed?.isValid) {
          results.push(processed);
        }
        
        result = iterator?.next?.();
      }
    }
    
    return results;
  }
  
  processItem(item) {
    return {
      ...item,
      isValid: item?.data != null && item?.id != null,
      processedAt: new Date().toISOString()
    };
  }
  
  // Optional chaining with WeakMap and Map operations
  getCachedValue(key, factory) {
    // Optional chaining with Map methods
    const cached = this.cache?.get?.(key);
    
    if (cached?.isValid?.() && cached?.expiresAt > Date.now()) {
      return cached.value;
    }
    
    // Create new value and cache it
    const newValue = factory?.();
    
    if (newValue != null) {
      this.cache?.set?.(key, {
        value: newValue,
        isValid: () => true,
        expiresAt: Date.now() + 300000 // 5 minutes
      });
    }
    
    return newValue;
  }
  
  // Optional chaining with DOM operations (if in browser)
  updateDOM(selector, data) {
    const element = document?.querySelector?.(selector);
    
    if (element) {
      // Optional chaining with DOM properties and methods
      element.textContent = data?.text || '';
      element.className = data?.cssClass || '';
      
      // Optional chaining with style object
      if (data?.styles) {
        Object.entries(data.styles).forEach(([property, value]) => {
          element?.style?.setProperty?.(property, value);
        });
      }
      
      // Optional chaining with event listeners
      const onClick = data?.events?.click;
      if (onClick && typeof onClick === 'function') {
        element?.addEventListener?.('click', onClick);
      }
      
      // Optional chaining with dataset
      const dataset = data?.dataset;
      if (dataset) {
        Object.entries(dataset).forEach(([key, value]) => {
          element?.dataset?.[key] = value;
        });
      }
    }
    
    return element;
  }
}

// Function using optional chaining in various contexts
function complexOptionalChainingExamples() {
  const testData = {
    users: [
      {
        id: 1,
        profile: {
          personal: {
            name: { first: 'John', last: 'Doe' },
            dateOfBirth: { year: 1990, month: 5, day: 15 }
          }
        },
        contact: {
          email: { primary: { address: 'john@example.com' } },
          phone: { mobile: { number: '123-456-7890' } }
        },
        settings: {
          privacy: { dataProcessing: { consent: true } }
        },
        activity: {
          lastLogin: { date: new Date('2023-01-15') }
        }
      }
    ]
  };
  
  const processor = new DataProcessor();
  
  // Test various optional chaining scenarios
  const results = {
    processedUser: processor.processUserData(testData.users?.[0]),
    arrayProcessing: processor.processArrayData(testData?.users),
    nestedValue: processor.getNestedValue(testData, 'users.0.profile.personal.name.first'),
    dynamicAccess: processor.dynamicPropertyAccess(testData.users?.[0], {
      userName: { path: 'profile.personal.name.first', default: 'Anonymous' },
      userEmail: { path: 'contact.email.primary.address', transform: val => val?.toLowerCase?.() }
    })
  };
  
  return results;
}

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = {
    DataProcessor,
    complexOptionalChainingExamples
  };
}
"#;
        
        let file_path = create_test_file(content, "js");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse complex optional chaining");
    }

    #[test]
    fn test_javascript_bigint_operations() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
// BigInt Operations and Edge Cases

class BigIntCalculator {
  constructor() {
    this.precision = 100n; // BigInt literal
    this.maxSafeInteger = BigInt(Number.MAX_SAFE_INTEGER);
    this.cache = new Map();
  }
  
  // Basic BigInt arithmetic operations
  basicOperations(a, b) {
    // Ensure inputs are BigInts
    const bigA = typeof a === 'bigint' ? a : BigInt(a);
    const bigB = typeof b === 'bigint' ? b : BigInt(b);
    
    return {
      sum: bigA + bigB,
      difference: bigA - bigB,
      product: bigA * bigB,
      quotient: bigB !== 0n ? bigA / bigB : null,
      remainder: bigB !== 0n ? bigA % bigB : null,
      power: this.power(bigA, bigB)
    };
  }
  
  // BigInt power operation with edge case handling
  power(base, exponent) {
    if (exponent < 0n) {
      throw new Error('BigInt power does not support negative exponents');
    }
    
    if (exponent === 0n) return 1n;
    if (exponent === 1n) return base;
    
    // Use exponentiation by squaring for efficiency
    let result = 1n;
    let currentBase = base;
    let currentExponent = exponent;
    
    while (currentExponent > 0n) {
      if (currentExponent % 2n === 1n) {
        result *= currentBase;
      }
      currentBase *= currentBase;
      currentExponent /= 2n;
    }
    
    return result;
  }
  
  // Factorial calculation with BigInt
  factorial(n) {
    const bigN = typeof n === 'bigint' ? n : BigInt(n);
    
    if (bigN < 0n) {
      throw new Error('Factorial is not defined for negative numbers');
    }
    
    if (bigN <= 1n) return 1n;
    
    // Check cache
    if (this.cache.has(bigN)) {
      return this.cache.get(bigN);
    }
    
    let result = 1n;
    for (let i = 2n; i <= bigN; i++) {
      result *= i;
    }
    
    // Cache result
    this.cache.set(bigN, result);
    return result;
  }
  
  // Fibonacci sequence with BigInt
  fibonacci(n) {
    const bigN = typeof n === 'bigint' ? n : BigInt(n);
    
    if (bigN < 0n) {
      throw new Error('Fibonacci is not defined for negative numbers');
    }
    
    if (bigN <= 1n) return bigN;
    
    let prev = 0n;
    let current = 1n;
    
    for (let i = 2n; i <= bigN; i++) {
      const next = prev + current;
      prev = current;
      current = next;
    }
    
    return current;
  }
  
  // Greatest Common Divisor using Euclidean algorithm
  gcd(a, b) {
    let bigA = typeof a === 'bigint' ? a : BigInt(a);
    let bigB = typeof b === 'bigint' ? b : BigInt(b);
    
    // Make both positive
    bigA = bigA < 0n ? -bigA : bigA;
    bigB = bigB < 0n ? -bigB : bigB;
    
    while (bigB !== 0n) {
      const temp = bigB;
      bigB = bigA % bigB;
      bigA = temp;
    }
    
    return bigA;
  }
  
  // Least Common Multiple
  lcm(a, b) {
    const bigA = typeof a === 'bigint' ? a : BigInt(a);
    const bigB = typeof b === 'bigint' ? b : BigInt(b);
    
    return (bigA * bigB) / this.gcd(bigA, bigB);
  }
  
  // Prime number check with BigInt
  isPrime(n) {
    const bigN = typeof n === 'bigint' ? n : BigInt(n);
    
    if (bigN < 2n) return false;
    if (bigN === 2n) return true;
    if (bigN % 2n === 0n) return false;
    
    // Check odd divisors up to sqrt(n)
    const sqrt = this.sqrt(bigN);
    for (let i = 3n; i <= sqrt; i += 2n) {
      if (bigN % i === 0n) return false;
    }
    
    return true;
  }
  
  // BigInt square root (integer part)
  sqrt(n) {
    const bigN = typeof n === 'bigint' ? n : BigInt(n);
    
    if (bigN < 0n) {
      throw new Error('Square root is not defined for negative numbers');
    }
    
    if (bigN < 2n) return bigN;
    
    // Binary search approach
    let left = 1n;
    let right = bigN;
    let result = 0n;
    
    while (left <= right) {
      const mid = (left + right) / 2n;
      const square = mid * mid;
      
      if (square === bigN) return mid;
      
      if (square < bigN) {
        result = mid;
        left = mid + 1n;
      } else {
        right = mid - 1n;
      }
    }
    
    return result;
  }
  
  // Convert between BigInt and other representations
  conversions(value) {
    const bigValue = typeof value === 'bigint' ? value : BigInt(value);
    
    return {
      binary: bigValue.toString(2),
      octal: bigValue.toString(8),
      decimal: bigValue.toString(10),
      hexadecimal: bigValue.toString(16),
      // Custom base conversion
      base36: bigValue.toString(36),
      // Array of digits
      digits: this.toDigitArray(bigValue),
      // Bit operations
      bitLength: this.getBitLength(bigValue),
      // Comparison with regular numbers
      exceedsMaxSafeInteger: bigValue > this.maxSafeInteger
    };
  }
  
  // Convert BigInt to array of digits
  toDigitArray(bigInt) {
    const str = bigInt.toString();
    return str.split('').map(digit => parseInt(digit, 10));
  }
  
  // Get bit length of BigInt
  getBitLength(bigInt) {
    if (bigInt === 0n) return 1;
    
    const abs = bigInt < 0n ? -bigInt : bigInt;
    return abs.toString(2).length;
  }
  
  // BigInt bitwise operations
  bitwiseOperations(a, b) {
    const bigA = typeof a === 'bigint' ? a : BigInt(a);
    const bigB = typeof b === 'bigint' ? b : BigInt(b);
    
    return {
      and: bigA & bigB,
      or: bigA | bigB,
      xor: bigA ^ bigB,
      not_a: ~bigA,
      not_b: ~bigB,
      leftShift: bigA << 1n,
      rightShift: bigA >> 1n,
      // Custom bit manipulation
      setBit: this.setBit(bigA, 5n),
      clearBit: this.clearBit(bigA, 3n),
      toggleBit: this.toggleBit(bigA, 7n)
    };
  }
  
  // Set specific bit
  setBit(value, position) {
    return value | (1n << position);
  }
  
  // Clear specific bit
  clearBit(value, position) {
    return value & ~(1n << position);
  }
  
  // Toggle specific bit
  toggleBit(value, position) {
    return value ^ (1n << position);
  }
  
  // Range operations with BigInt
  rangeOperations(start, end, step = 1n) {
    const bigStart = typeof start === 'bigint' ? start : BigInt(start);
    const bigEnd = typeof end === 'bigint' ? end : BigInt(end);
    const bigStep = typeof step === 'bigint' ? step : BigInt(step);
    
    const range = [];
    const sum = { value: 0n };
    const product = { value: 1n };
    let count = 0n;
    
    if (bigStep > 0n) {
      for (let i = bigStart; i < bigEnd; i += bigStep) {
        range.push(i);
        sum.value += i;
        product.value *= i;
        count++;
        
        // Prevent memory issues with large ranges
        if (count > 10000n) break;
      }
    } else if (bigStep < 0n) {
      for (let i = bigStart; i > bigEnd; i += bigStep) {
        range.push(i);
        sum.value += i;
        product.value *= i;
        count++;
        
        if (count > 10000n) break;
      }
    }
    
    return {
      range: range.slice(0, 100), // Limit output for practical reasons
      sum: sum.value,
      product: product.value,
      count,
      average: count > 0n ? sum.value / count : 0n
    };
  }
  
  // Complex mathematical operations
  complexOperations() {
    // Calculate very large prime numbers
    const largePrimes = [];
    let candidate = 1000000000000000000000n;
    
    while (largePrimes.length < 3) {
      if (this.isPrime(candidate)) {
        largePrimes.push(candidate);
      }
      candidate += 1n;
    }
    
    // Calculate factorials of large numbers
    const largeFactorials = [50n, 100n, 150n].map(n => ({
      n,
      factorial: this.factorial(n),
      digitCount: this.factorial(n).toString().length
    }));
    
    // Fibonacci numbers
    const largeFibonacci = [100n, 200n, 300n].map(n => ({
      n,
      value: this.fibonacci(n),
      digitCount: this.fibonacci(n).toString().length
    }));
    
    return {
      largePrimes,
      largeFactorials,
      largeFibonacci,
      computationTime: Date.now() // Simplified timing
    };
  }
  
  // Type checking and validation
  validateBigInt(value) {
    return {
      isBigInt: typeof value === 'bigint',
      canConvertToBigInt: this.canConvertToBigInt(value),
      isInteger: typeof value === 'bigint' || Number.isInteger(value),
      isInSafeRange: typeof value === 'number' ? Number.isSafeInteger(value) : null,
      stringRepresentation: value.toString(),
      typeOfValue: typeof value
    };
  }
  
  canConvertToBigInt(value) {
    try {
      BigInt(value);
      return true;
    } catch (error) {
      return false;
    }
  }
}

// Cryptographic operations with BigInt
class BigIntCrypto {
  // Modular exponentiation (a^b mod m)
  static modPow(base, exponent, modulus) {
    const bigBase = typeof base === 'bigint' ? base : BigInt(base);
    const bigExp = typeof exponent === 'bigint' ? exponent : BigInt(exponent);
    const bigMod = typeof modulus === 'bigint' ? modulus : BigInt(modulus);
    
    if (bigMod <= 0n) {
      throw new Error('Modulus must be positive');
    }
    
    let result = 1n;
    let currentBase = bigBase % bigMod;
    let currentExp = bigExp;
    
    while (currentExp > 0n) {
      if (currentExp % 2n === 1n) {
        result = (result * currentBase) % bigMod;
      }
      currentExp = currentExp >> 1n;
      currentBase = (currentBase * currentBase) % bigMod;
    }
    
    return result;
  }
  
  // Simple RSA key generation example (educational purposes)
  static generateSimpleRSAKeys() {
    // Small primes for demonstration
    const p = 61n;
    const q = 53n;
    const n = p * q;
    const phi = (p - 1n) * (q - 1n);
    
    // Common public exponent
    const e = 65537n;
    
    // Calculate private exponent
    const d = this.modInverse(e, phi);
    
    return {
      public: { n, e },
      private: { n, d },
      primes: { p, q }
    };
  }
  
  // Modular multiplicative inverse using extended Euclidean algorithm
  static modInverse(a, m) {
    const bigA = typeof a === 'bigint' ? a : BigInt(a);
    const bigM = typeof m === 'bigint' ? m : BigInt(m);
    
    if (bigM <= 0n) {
      throw new Error('Modulus must be positive');
    }
    
    // Extended Euclidean Algorithm
    let [oldR, r] = [bigA, bigM];
    let [oldS, s] = [1n, 0n];
    
    while (r !== 0n) {
      const quotient = oldR / r;
      [oldR, r] = [r, oldR - quotient * r];
      [oldS, s] = [s, oldS - quotient * s];
    }
    
    if (oldR > 1n) {
      throw new Error('Modular inverse does not exist');
    }
    
    return oldS < 0n ? oldS + bigM : oldS;
  }
}

// Usage examples and tests
function testBigIntOperations() {
  const calculator = new BigIntCalculator();
  
  // Test large number operations
  const largeNumber1 = 123456789012345678901234567890n;
  const largeNumber2 = 987654321098765432109876543210n;
  
  const basicOps = calculator.basicOperations(largeNumber1, largeNumber2);
  console.log('Basic operations:', basicOps);
  
  // Test mathematical functions
  const factorial100 = calculator.factorial(100n);
  console.log('100! has', factorial100.toString().length, 'digits');
  
  const fib1000 = calculator.fibonacci(1000n);
  console.log('Fib(1000) has', fib1000.toString().length, 'digits');
  
  // Test conversions
  const conversions = calculator.conversions(largeNumber1);
  console.log('Conversions:', conversions);
  
  // Test bitwise operations
  const bitwise = calculator.bitwiseOperations(255n, 170n);
  console.log('Bitwise operations:', bitwise);
  
  // Test crypto operations
  const rsaKeys = BigIntCrypto.generateSimpleRSAKeys();
  console.log('RSA keys:', rsaKeys);
  
  return {
    basicOps,
    factorial100: factorial100.toString().slice(0, 50) + '...',
    fibonacci: fib1000.toString().slice(0, 50) + '...',
    conversions,
    bitwise,
    rsaKeys
  };
}

// Export for testing
if (typeof module !== 'undefined') {
  module.exports = {
    BigIntCalculator,
    BigIntCrypto,
    testBigIntOperations
  };
}
"#;
        
        let file_path = create_test_file(content, "js");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse BigInt operations");
    }

    // TYPESCRIPT EDGE CASE TESTS
    #[test]
    fn test_typescript_mapped_types_conditional() {
        let mut parser = AstParser::new().unwrap();
        let content = r#"
// Complex Mapped Types and Conditional Types in TypeScript

// Basic mapped types
type Readonly<T> = {
  readonly [P in keyof T]: T[P];
};

type Partial<T> = {
  [P in keyof T]?: T[P];
};

type Required<T> = {
  [P in keyof T]-?: T[P];
};

// Advanced mapped types with key remapping
type Getters<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K];
};

type Setters<T> = {
  [K in keyof T as `set${Capitalize<string & K>}`]: (value: T[K]) => void;
};

// Conditional mapped types
type NonNullable<T> = {
  [P in keyof T]: T[P] extends null | undefined ? never : T[P];
};

type StringKeys<T> = {
  [K in keyof T]: T[K] extends string ? K : never;
}[keyof T];

type NumberKeys<T> = {
  [K in keyof T]: T[K] extends number ? K : never;
}[keyof T];

// Complex conditional types
type IsArray<T> = T extends (infer U)[] ? true : false;
type ArrayElement<T> = T extends (infer U)[] ? U : never;

type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object
    ? T[P] extends Function
      ? T[P]
      : DeepReadonly<T[P]>
    : T[P];
};

// Recursive mapped types
type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object
    ? T[P] extends Function
      ? T[P]
      : DeepPartial<T[P]>
    : T[P];
};

// Mapped types with template literal types
type EventHandlers<T extends Record<string, any>> = {
  [K in keyof T as `on${Capitalize<string & K>}`]: (value: T[K]) => void;
};

type ApiRoutes<T extends Record<string, any>> = {
  [K in keyof T as `/api/${string & K}`]: T[K];
};

// Complex conditional types with multiple conditions
type ExtractArrayType<T> = T extends readonly (infer U)[]
  ? U
  : T extends readonly [...(infer U)[]]
  ? U
  : never;

type FunctionReturnType<T> = T extends (...args: any[]) => infer R ? R : never;

type FunctionParameters<T> = T extends (...args: infer P) => any ? P : never;

// Distributive conditional types
type ToArray<T> = T extends any ? T[] : never;

type ExcludeFromUnion<T, U> = T extends U ? never : T;

// Infer with constraints
type GetConstructorArgs<T> = T extends new (...args: infer U) => any ? U : never;

type GetMethodNames<T> = {
  [K in keyof T]: T[K] extends Function ? K : never;
}[keyof T];

// Complex mapped type with multiple transformations
type ApiEndpoint<T extends Record<string, any>> = {
  [K in keyof T as T[K] extends Function
    ? `${string & K}Endpoint`
    : never]: T[K] extends (...args: infer P) => infer R
    ? (params: P[0], options?: RequestInit) => Promise<R>
    : never;
};

// Mapped types with key filtering
type PickByType<T, U> = {
  [K in keyof T as T[K] extends U ? K : never]: T[K];
};

type OmitByType<T, U> = {
  [K in keyof T as T[K] extends U ? never : K]: T[K];
};

// Nested conditional types with recursion
type Flatten<T> = T extends readonly (infer U)[]
  ? Flatten<U>
  : T;

type DeepKeyof<T> = T extends object
  ? {
      [K in keyof T]: K extends string
        ? T[K] extends object
          ? K | `${K}.${DeepKeyof<T[K]>}`
          : K
        : never;
    }[keyof T]
  : never;

// Advanced utility types
type Intersection<T extends readonly unknown[]> = T extends readonly [
  infer H,
  ...infer Rest
]
  ? H & Intersection<Rest>
  : unknown;

type Union<T extends readonly unknown[]> = T extends readonly [
  infer H,
  ...infer Rest
]
  ? H | Union<Rest>
  : never;

// Template literal type patterns
type CSSProperty = `--${string}`;
type DataAttribute = `data-${string}`;
type EventName = `on${Capitalize<string>}`;

type RoutePattern<T extends string> = T extends `${infer Start}/:${infer Param}/${infer End}`
  ? { [K in Param]: string } & RoutePattern<`${Start}/${End}`>
  : T extends `${infer Start}/:${infer Param}`
  ? { [K in Param]: string }
  : {};

// Complex class with mapped types
interface UserData {
  id: number;
  name: string;
  email: string;
  age: number;
  isActive: boolean;
  preferences: {
    theme: 'light' | 'dark';
    language: string;
    notifications: boolean;
  };
}

class DataProcessor<T extends Record<string, any>> {
  private data: T;
  private getters: Getters<T>;
  private setters: Setters<T>;

  constructor(initialData: T) {
    this.data = initialData;
    this.getters = this.createGetters();
    this.setters = this.createSetters();
  }

  private createGetters(): Getters<T> {
    const getters = {} as Getters<T>;
    
    for (const key in this.data) {
      const getterName = `get${this.capitalize(key)}` as keyof Getters<T>;
      (getters as any)[getterName] = () => this.data[key];
    }
    
    return getters;
  }

  private createSetters(): Setters<T> {
    const setters = {} as Setters<T>;
    
    for (const key in this.data) {
      const setterName = `set${this.capitalize(key)}` as keyof Setters<T>;
      (setters as any)[setterName] = (value: T[typeof key]) => {
        this.data[key] = value;
      };
    }
    
    return setters;
  }

  private capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }

  public getField<K extends keyof T>(key: K): T[K] {
    return this.data[key];
  }

  public setField<K extends keyof T>(key: K, value: T[K]): void {
    this.data[key] = value;
  }

  public getStringFields(): PickByType<T, string> {
    const result = {} as PickByType<T, string>;
    
    for (const key in this.data) {
      if (typeof this.data[key] === 'string') {
        (result as any)[key] = this.data[key];
      }
    }
    
    return result;
  }

  public getNumberFields(): PickByType<T, number> {
    const result = {} as PickByType<T, number>;
    
    for (const key in this.data) {
      if (typeof this.data[key] === 'number') {
        (result as any)[key] = this.data[key];
      }
    }
    
    return result;
  }

  public toReadonly(): Readonly<T> {
    return { ...this.data } as Readonly<T>;
  }

  public toPartial(): Partial<T> {
    return { ...this.data } as Partial<T>;
  }
}

// Function using complex conditional types
function processApiResponse<T extends Record<string, any>>(
  response: T
): T extends { data: infer U }
  ? U extends any[]
    ? { items: U; count: number }
    : { item: U }
  : { error: string } {
  if ('data' in response) {
    const data = response.data;
    
    if (Array.isArray(data)) {
      return {
        items: data,
        count: data.length
      } as any;
    } else {
      return {
        item: data
      } as any;
    }
  }
  
  return {
    error: 'Invalid response format'
  } as any;
}

// Advanced generic constraints
interface Comparable<T> {
  compareTo(other: T): number;
}

function sort<T extends Comparable<T>>(items: T[]): T[] {
  return items.sort((a, b) => a.compareTo(b));
}

// Mapped type with complex key transformation
type ApiClient<T extends Record<string, (...args: any[]) => any>> = {
  [K in keyof T as `${string & K}Async`]: T[K] extends (...args: infer P) => infer R
    ? (...args: P) => Promise<R>
    : never;
};

// Example service interface
interface UserService {
  getUser(id: number): UserData | null;
  createUser(userData: Omit<UserData, 'id'>): UserData;
  updateUser(id: number, updates: Partial<UserData>): UserData | null;
  deleteUser(id: number): boolean;
}

// Create async API client type
type AsyncUserService = ApiClient<UserService>;

class UserApiClient implements AsyncUserService {
  async getUserAsync(id: number): Promise<UserData | null> {
    // Implementation would make HTTP request
    return null;
  }

  async createUserAsync(userData: Omit<UserData, 'id'>): Promise<UserData> {
    // Implementation would make HTTP request
    return { id: 1, ...userData };
  }

  async updateUserAsync(id: number, updates: Partial<UserData>): Promise<UserData | null> {
    // Implementation would make HTTP request
    return null;
  }

  async deleteUserAsync(id: number): Promise<boolean> {
    // Implementation would make HTTP request
    return false;
  }
}

// Complex conditional type for form validation
type ValidationRules<T> = {
  [K in keyof T]: T[K] extends string
    ? {
        required?: boolean;
        minLength?: number;
        maxLength?: number;
        pattern?: RegExp;
      }
    : T[K] extends number
    ? {
        required?: boolean;
        min?: number;
        max?: number;
      }
    : T[K] extends boolean
    ? {
        required?: boolean;
      }
    : T[K] extends object
    ? ValidationRules<T[K]>
    : {
        required?: boolean;
      };
};

// Form validator using mapped types
class FormValidator<T extends Record<string, any>> {
  private rules: ValidationRules<T>;

  constructor(rules: ValidationRules<T>) {
    this.rules = rules;
  }

  validate(data: T): { isValid: boolean; errors: Partial<Record<keyof T, string>> } {
    const errors: Partial<Record<keyof T, string>> = {};
    let isValid = true;

    for (const key in this.rules) {
      const rule = this.rules[key];
      const value = data[key];

      if (typeof rule === 'object' && 'required' in rule) {
        if (rule.required && (value === undefined || value === null)) {
          errors[key] = `${String(key)} is required`;
          isValid = false;
          continue;
        }

        if (typeof value === 'string' && 'minLength' in rule && rule.minLength) {
          if (value.length < rule.minLength) {
            errors[key] = `${String(key)} must be at least ${rule.minLength} characters`;
            isValid = false;
          }
        }

        if (typeof value === 'number' && 'min' in rule && rule.min !== undefined) {
          if (value < rule.min) {
            errors[key] = `${String(key)} must be at least ${rule.min}`;
            isValid = false;
          }
        }
      }
    }

    return { isValid, errors };
  }
}

// Usage examples
function exampleUsage() {
  // Create data processor
  const userData: UserData = {
    id: 1,
    name: 'John Doe',
    email: 'john@example.com',
    age: 30,
    isActive: true,
    preferences: {
      theme: 'dark',
      language: 'en',
      notifications: true
    }
  };

  const processor = new DataProcessor(userData);
  const stringFields = processor.getStringFields();
  const numberFields = processor.getNumberFields();

  // Form validation
  const validator = new FormValidator<Pick<UserData, 'name' | 'email' | 'age'>>({
    name: {
      required: true,
      minLength: 2,
      maxLength: 50
    },
    email: {
      required: true,
      pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    },
    age: {
      required: true,
      min: 0,
      max: 150
    }
  });

  const validationResult = validator.validate({
    name: 'John',
    email: 'john@example.com',
    age: 30
  });

  return {
    stringFields,
    numberFields,
    validationResult,
    processor: processor.toReadonly()
  };
}

export {
  DataProcessor,
  FormValidator,
  UserApiClient,
  exampleUsage
};

export type {
  UserData,
  ValidationRules,
  ApiClient,
  AsyncUserService,
  DeepReadonly,
  DeepPartial,
  PickByType,
  OmitByType
};
"#;
        
        let file_path = create_test_file(content, "ts");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse TypeScript mapped types and conditionals");
    }

    // MALFORMED/CORRUPTED FILE TESTS
    #[test]
    fn test_parser_resilience_truncated_files() {
        let mut parser = AstParser::new().unwrap();
        
        // Test truncated Rust file
        let truncated_rust = r#"
struct TestStruct {
    field1: i32,
    field2: String,
    // File truncated here - missing closing brace
"#;
        let file_path = create_test_file(truncated_rust, "rs");
        let result = parser.parse_file(&file_path);
        assert!(result.is_err() || result.unwrap().tree.unwrap().root_node().has_error(), 
                "Should handle truncated Rust file");
        
        // Test truncated Python file
        let truncated_python = r#"
def incomplete_function():
    if condition:
        print("test")
    # Missing return statement and proper indentation
"#;
        let file_path = create_test_file(truncated_python, "py");
        let result = parser.parse_file(&file_path);
        // Python parser might be more lenient, but should still handle gracefully
        assert!(result.is_ok(), "Should handle truncated Python file gracefully");
    }

    #[test]
    fn test_parser_resilience_binary_corruption() {
        let mut parser = AstParser::new().unwrap();
        
        // Test file with binary data mixed with source code
        let corrupted_content = "fn main() {\n    \x00\x01\x02\xFF\xFE\n    println!(\"test\");\n}\n";
        let file_path = create_test_file(corrupted_content, "rs");
        let result = parser.parse_file(&file_path);
        
        // Should either fail or handle gracefully
        match result {
            Ok(parsed) => {
                // If it succeeds, tree should indicate errors
                if let Some(tree) = parsed.tree {
                    assert!(tree.root_node().has_error(), "Should detect errors in corrupted content");
                }
            }
            Err(_) => {
                // Failing is also acceptable for corrupted input
            }
        }
    }

    #[test]
    fn test_parser_resilience_encoding_issues() {
        let mut parser = AstParser::new().unwrap();
        
        // Test file with invalid UTF-8 sequences
        let invalid_utf8 = "// Valid comment\nfn test() {\n    // Invalid UTF-8: \xC0\x80\n}\n";
        let file_path = create_test_file(invalid_utf8, "rs");
        let result = parser.parse_file(&file_path);
        
        // Should handle encoding issues gracefully
        match result {
            Ok(_) => {
                // Success is acceptable if parser handles it
            }
            Err(error) => {
                // Error is also acceptable for invalid encoding
                println!("Expected error for invalid encoding: {:?}", error);
            }
        }
    }

    #[test]
    fn test_parser_resilience_extremely_large_files() {
        let mut parser = AstParser::new().unwrap();
        
        // Create a large but syntactically correct file
        let mut large_content = String::from("// Large file test\n");
        for i in 0..1000 {
            large_content.push_str(&format!("fn function_{}() {{ println!(\"Function {}\"); }}\n", i, i));
        }
        
        let file_path = create_test_file(&large_content, "rs");
        let result = parser.parse_file(&file_path);
        
        // Should handle large files, though may be slow
        assert!(result.is_ok(), "Should handle large but valid files");
        
        if let Ok(parsed) = result {
            assert_eq!(parsed.language, SourceLanguage::Rust);
            assert!(parsed.source.len() > 10000, "File should be large");
        }
    }

    #[test]
    fn test_parser_resilience_nested_depth_limits() {
        let mut parser = AstParser::new().unwrap();
        
        // Test deeply nested structures
        let mut nested_rust = String::from("fn main() {\n");
        for _ in 0..100 {
            nested_rust.push_str("    if true {\n");
        }
        nested_rust.push_str("        println!(\"deep\");\n");
        for _ in 0..100 {
            nested_rust.push_str("    }\n");
        }
        nested_rust.push_str("}\n");
        
        let file_path = create_test_file(&nested_rust, "rs");
        let result = parser.parse_file(&file_path);
        
        // Should handle deeply nested structures
        assert!(result.is_ok(), "Should handle deeply nested structures");
        
        if let Ok(parsed) = result {
            assert_eq!(parsed.language, SourceLanguage::Rust);
        }
    }
}
"#;
        
        let file_path = create_test_file(content, "ts");
        let result = parser.parse_file(&file_path);
        assert!(result.is_ok(), "Failed to parse template literal types");
    }
}
"#
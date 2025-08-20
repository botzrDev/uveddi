# method_marathon.py - Extreme Long Methods in Python
# This file demonstrates extremely long methods with complex business logic

class MethodMarathon:
    """Class with extremely long methods"""
    
    def __init__(self):
        self.data = []
        self.config = {}
        self.state = {}
    
    def extremely_long_method_with_nested_logic(self, input_data):
        """
        EXTREMELY LONG METHOD: Deeply nested logic for data processing (400+ lines)
        This method demonstrates excessive nesting and complex business logic
        that should be broken into smaller, more manageable functions.
        """
        print('Starting extremely long method with nested logic...')
        
        # Level 1: Validate input data
        if not input_data or not isinstance(input_data, dict):
            raise ValueError('Invalid input data')
        
        # Level 2: Check data structure
        if 'items' not in input_data or not isinstance(input_data['items'], list):
            raise ValueError('Input data must contain items array')
        
        # Level 3: Validate minimum items count
        if len(input_data['items']) < 1:
            raise ValueError('Input data must contain at least one item')
        
        # Level 4: Process each item with nested logic
        processed_items = []
        
        # Level 5: Iterate through all items
        for item_index, current_item in enumerate(input_data['items']):
            # Level 6: Validate current item
            if current_item and isinstance(current_item, dict):
                # Level 7: Process item ID
                if 'id' not in current_item or not isinstance(current_item['id'], (int, float)):
                    current_item['id'] = item_index + 1
                
                # Level 8: Process item value
                if 'value' not in current_item or not isinstance(current_item['value'], (int, float)):
                    current_item['value'] = 0
                
                # Level 9: Process item name
                if 'name' not in current_item or not isinstance(current_item['name'], str):
                    current_item['name'] = f'Item #{current_item["id"]}'
                
                # Level 10: Process item properties
                if 'properties' not in current_item or not isinstance(current_item['properties'], dict):
                    current_item['properties'] = {}
                
                # Level 11: Add default properties
                if 'created' not in current_item['properties']:
                    current_item['properties']['created'] = __import__('datetime').datetime.now().isoformat()
                
                if 'modified' not in current_item['properties']:
                    current_item['properties']['modified'] = __import__('datetime').datetime.now().isoformat()
                
                if 'version' not in current_item['properties']:
                    current_item['properties']['version'] = '1.0.0'
                
                # Level 12: Process item tags
                if 'tags' not in current_item or not isinstance(current_item['tags'], list):
                    current_item['tags'] = []
                
                # Level 13: Add default tags
                if len(current_item['tags']) == 0:
                    current_item['tags'].append('default')
                
                # Level 14: Limit tags
                if len(current_item['tags']) > 10:
                    current_item['tags'] = current_item['tags'][:10]
                
                # Level 15: Process item status
                if 'status' not in current_item:
                    current_item['status'] = 'pending'
                
                # Level 16: Validate status
                valid_statuses = ['pending', 'processing', 'completed', 'failed', 'cancelled']
                if current_item['status'] not in valid_statuses:
                    current_item['status'] = 'pending'
                
                # Level 17: Process item priority
                if 'priority' not in current_item or not isinstance(current_item['priority'], (int, float)):
                    current_item['priority'] = 0
                
                # Level 18: Validate priority range
                if current_item['priority'] < 0 or current_item['priority'] > 10:
                    current_item['priority'] = max(0, min(10, current_item['priority']))
                
                # Level 19: Process nested properties
                if current_item['properties'] and isinstance(current_item['properties'], dict):
                    prop_keys = list(current_item['properties'].keys())
                    
                    # Level 20: Process each property
                    for prop_key in prop_keys:
                        prop_value = current_item['properties'][prop_key]
                        
                        # Level 21: Validate property value
                        if prop_value is not None:
                            # Level 22: Process string properties
                            if isinstance(prop_value, str):
                                # Level 23: Check string length
                                if len(prop_value) > 100:
                                    # Level 24: Truncate long strings
                                    current_item['properties'][prop_key] = prop_value[:100] + '...'
                            
                            # Level 25: Process number properties
                            elif isinstance(prop_value, (int, float)):
                                # Level 26: Check number range
                                if prop_value < 0:
                                    # Level 27: Convert negative numbers to positive
                                    current_item['properties'][prop_key] = abs(prop_value)
                            
                            # Level 28: Process array properties
                            elif isinstance(prop_value, list):
                                # Level 29: Limit array size
                                if len(prop_value) > 50:
                                    # Level 30: Truncate large arrays
                                    current_item['properties'][prop_key] = prop_value[:50]
                            
                            # Level 31: Process object properties
                            elif isinstance(prop_value, dict):
                                # Level 32: Limit object properties
                                obj_keys = list(prop_value.keys())
                                if len(obj_keys) > 20:
                                    # Level 33: Remove excess properties
                                    new_obj = {}
                                    for k in obj_keys[:20]:
                                        new_obj[k] = prop_value[k]
                                    current_item['properties'][prop_key] = new_obj
                
                # Level 34: Process nested arrays
                if 'nested_arrays' in current_item and isinstance(current_item['nested_arrays'], list):
                    # Level 35: Process each nested array
                    for k, nested_array in enumerate(current_item['nested_arrays']):
                        # Level 36: Validate nested array
                        if isinstance(nested_array, list):
                            # Level 37: Limit nested array size
                            if len(nested_array) > 30:
                                # Level 38: Truncate nested arrays
                                current_item['nested_arrays'][k] = nested_array[:30]
                            
                            # Level 39: Process nested array items
                            for l, nested_item in enumerate(nested_array):
                                # Level 40: Validate nested item
                                if nested_item and isinstance(nested_item, dict):
                                    # Level 41: Process nested item properties
                                    nested_keys = list(nested_item.keys())
                                    for nested_key in nested_keys:
                                        nested_value = nested_item[nested_key]
                                        
                                        # Level 42: Sanitize nested values
                                        if isinstance(nested_value, str):
                                            # Level 43: Remove special characters
                                            nested_item[nested_key] = nested_value.replace('<', '').replace('>', '')
                
                # Level 44: Add to processed items
                processed_items.append(current_item)
        
        # Level 45: Validate processed items
        if len(processed_items) == 0:
            raise ValueError('No items were processed successfully')
        
        # Level 46: Calculate statistics
        stats = {
            'total_items': len(processed_items),
            'average_value': 0,
            'min_value': float('inf'),
            'max_value': float('-inf'),
            'sum': 0
        }
        
        # Level 47: Calculate sum and find min/max
        for item in processed_items:
            # Level 48: Validate item value
            if isinstance(item['value'], (int, float)):
                stats['sum'] += item['value']
                
                # Level 49: Update min value
                if item['value'] < stats['min_value']:
                    stats['min_value'] = item['value']
                
                # Level 50: Update max value
                if item['value'] > stats['max_value']:
                    stats['max_value'] = item['value']
        
        # Level 51: Calculate average
        if len(processed_items) > 0:
            stats['average_value'] = stats['sum'] / len(processed_items)
        
        # Level 52: Validate statistics
        if stats['min_value'] == float('inf'):
            stats['min_value'] = 0
        
        if stats['max_value'] == float('-inf'):
            stats['max_value'] = 0
        
        # Level 53: Create result object
        result = {
            'items': processed_items,
            'statistics': stats,
            'metadata': {
                'processed_at': __import__('datetime').datetime.now().isoformat(),
                'processor_version': '1.0.0',
                'input_item_count': len(input_data['items']),
                'output_item_count': len(processed_items)
            }
        }
        
        # Level 54: Apply final transformations
        result['metadata']['processing_time'] = __import__('time').time() - getattr(self, '_start_time', __import__('time').time())
        result['metadata']['efficiency'] = (result['statistics']['total_items'] / len(input_data['items'])) * 100
        
        # Level 55: Validate result
        if len(result['items']) > 0:
            # Level 56: Sort items by value
            result['items'].sort(key=lambda x: x['value'])
            
            # Level 57: Apply final formatting
            result['formatted'] = True
            result['version'] = '2.0.0'
            
            # Level 58: Add checksum
            checksum = 0
            for item in result['items']:
                checksum += item.get('id', 0)
            result['checksum'] = checksum
            
            # Level 59: Final validation
            if result['checksum'] > 0:
                return result
            else:
                raise ValueError('Result validation failed')
        else:
            raise ValueError('No valid items in result')

    def complex_business_logic_processor(self, config, data, options):
        """
        EXTREMELY LONG METHOD: Complex business logic with multiple responsibilities (300+ lines)
        This method handles multiple business domains in a single function, violating
        the Single Responsibility Principle.
        """
        print('Starting complex business logic processing...')
        
        # Validate inputs
        if not config or not isinstance(config, dict):
            raise ValueError('Configuration is required')
        
        if not data or not isinstance(data, dict):
            raise ValueError('Data is required')
        
        if not options or not isinstance(options, dict):
            raise ValueError('Options are required')
        
        # Extract configuration values
        enable_validation = config.get('enable_validation', True)
        enable_transformation = config.get('enable_transformation', True)
        enable_enrichment = config.get('enable_enrichment', True)
        enable_filtering = config.get('enable_filtering', True)
        enable_sorting = config.get('enable_sorting', True)
        enable_pagination = config.get('enable_pagination', True)
        enable_caching = config.get('enable_caching', True)
        enable_logging = config.get('enable_logging', True)
        validation_rules = config.get('validation_rules', {})
        transformation_rules = config.get('transformation_rules', {})
        enrichment_rules = config.get('enrichment_rules', {})
        filtering_rules = config.get('filtering_rules', {})
        sorting_rules = config.get('sorting_rules', {})
        pagination_settings = config.get('pagination_settings', {})
        cache_settings = config.get('cache_settings', {})
        logging_settings = config.get('logging_settings', {})
        
        # Extract data components
        users = data.get('users', [])
        products = data.get('products', [])
        orders = data.get('orders', [])
        transactions = data.get('transactions', [])
        inventory = data.get('inventory', [])
        categories = data.get('categories', [])
        suppliers = data.get('suppliers', [])
        customers = data.get('customers', [])
        
        # Extract options
        user_id = options.get('user_id')
        product_id = options.get('product_id')
        order_id = options.get('order_id')
        from_date = options.get('from_date')
        to_date = options.get('to_date')
        min_amount = options.get('min_amount', 0)
        max_amount = options.get('max_amount', float('inf'))
        status = options.get('status')
        category = options.get('category')
        supplier = options.get('supplier')
        customer = options.get('customer')
        sort_by = options.get('sort_by', 'id')
        sort_order = options.get('sort_order', 'asc')
        page = options.get('page', 1)
        limit = options.get('limit', 100)
        include_related = options.get('include_related', True)
        include_metadata = options.get('include_metadata', True)
        include_statistics = options.get('include_statistics', True)
        
        # Initialize result object
        result = {
            'data': [],
            'metadata': {},
            'statistics': {}
        }
        
        # Initialize processing steps
        validation_passed = True
        transformation_applied = False
        enrichment_completed = False
        filtering_applied = False
        sorting_applied = False
        pagination_applied = False
        
        # Step 1: Validation
        if enable_validation:
            print('Applying validation rules...')
            
            # Validate users
            for i, user in enumerate(users):
                # Validate user ID
                if 'id' not in user or not isinstance(user['id'], (int, float)):
                    if validation_rules.get('user_id_required') is not False:
                        validation_passed = False
                        break
                
                # Validate user email
                if 'email' not in user or not isinstance(user['email'], str):
                    if validation_rules.get('email_required') is not False:
                        validation_passed = False
                        break
                else:
                    # Validate email format
                    import re
                    email_regex = r'^[^\s@]+@[^\s@]+\.[^\s@]+$'
                    if not re.match(email_regex, user['email']):
                        if validation_rules.get('email_format_required') is not False:
                            validation_passed = False
                            break
                
                # Validate user name
                if 'name' not in user or not isinstance(user['name'], str):
                    if validation_rules.get('name_required') is not False:
                        validation_passed = False
                        break
                
                # Validate user status
                if 'status' in user and isinstance(user['status'], str):
                    valid_statuses = ['active', 'inactive', 'suspended', 'deleted']
                    if user['status'] not in valid_statuses:
                        if validation_rules.get('status_validation') is not False:
                            validation_passed = False
                            break
            
            # Validate products if validation still passed
            if validation_passed:
                for i, product in enumerate(products):
                    # Validate product ID
                    if 'id' not in product or not isinstance(product['id'], (int, float)):
                        if validation_rules.get('product_id_required') is not False:
                            validation_passed = False
                            break
                    
                    # Validate product name
                    if 'name' not in product or not isinstance(product['name'], str):
                        if validation_rules.get('product_name_required') is not False:
                            validation_passed = False
                            break
                    
                    # Validate product price
                    if 'price' not in product or not isinstance(product['price'], (int, float)):
                        if validation_rules.get('price_required') is not False:
                            validation_passed = False
                            break
                    elif product['price'] < 0:
                        if validation_rules.get('price_positive') is not False:
                            validation_passed = False
                            break
                    
                    # Validate product category
                    if 'category' in product and isinstance(product['category'], str):
                        if len(categories) > 0:
                            category_found = False
                            for category_item in categories:
                                if category_item.get('id') == product['category'] or category_item.get('name') == product['category']:
                                    category_found = True
                                    break
                            if not category_found and validation_rules.get('category_validation') is not False:
                                validation_passed = False
                                break
            
            # Validate orders if validation still passed
            if validation_passed:
                for i, order in enumerate(orders):
                    # Validate order ID
                    if 'id' not in order or not isinstance(order['id'], (int, float)):
                        if validation_rules.get('order_id_required') is not False:
                            validation_passed = False
                            break
                    
                    # Validate order user ID
                    if 'user_id' not in order or not isinstance(order['user_id'], (int, float)):
                        if validation_rules.get('order_user_id_required') is not False:
                            validation_passed = False
                            break
                    
                    # Validate order items
                    if 'items' not in order or not isinstance(order['items'], list):
                        if validation_rules.get('order_items_required') is not False:
                            validation_passed = False
                            break
                    elif len(order['items']) == 0:
                        if validation_rules.get('order_items_not_empty') is not False:
                            validation_passed = False
                            break
                    else:
                        # Validate each order item
                        for j, item in enumerate(order['items']):
                            # Validate item product ID
                            if 'product_id' not in item or not isinstance(item['product_id'], (int, float)):
                                if validation_rules.get('order_item_product_id_required') is not False:
                                    validation_passed = False
                                    break
                            
                            # Validate item quantity
                            if 'quantity' not in item or not isinstance(item['quantity'], (int, float)):
                                if validation_rules.get('order_item_quantity_required') is not False:
                                    validation_passed = False
                                    break
                            elif item['quantity'] <= 0:
                                if validation_rules.get('order_item_quantity_positive') is not False:
                                    validation_passed = False
                                    break
                            
                            # Validate item price
                            if 'price' not in item or not isinstance(item['price'], (int, float)):
                                if validation_rules.get('order_item_price_required') is not False:
                                    validation_passed = False
                                    break
                            elif item['price'] < 0:
                                if validation_rules.get('order_item_price_positive') is not False:
                                    validation_passed = False
                                    break
                        
                        # Break outer loop if validation failed in items
                        if not validation_passed:
                            break
                    
                    # Validate order total
                    if 'total' not in order or not isinstance(order['total'], (int, float)):
                        if validation_rules.get('order_total_required') is not False:
                            validation_passed = False
                            break
                    elif order['total'] < 0:
                        if validation_rules.get('order_total_positive') is not False:
                            validation_passed = False
                            break
                    
                    # Validate order status
                    if 'status' in order and isinstance(order['status'], str):
                        valid_statuses = ['pending', 'processing', 'shipped', 'delivered', 'cancelled', 'refunded']
                        if order['status'] not in valid_statuses:
                            if validation_rules.get('order_status_validation') is not False:
                                validation_passed = False
                                break
        
        # Check validation result
        if not validation_passed:
            raise ValueError('Data validation failed')
        
        # Step 2: Transformation
        if enable_transformation:
            print('Applying transformation rules...')
            transformation_applied = True
            
            # Transform users
            for i, user in enumerate(users):
                # Normalize email
                if 'email' in user and isinstance(user['email'], str):
                    user['email'] = user['email'].lower().strip()
                
                # Format name
                if 'name' in user and isinstance(user['name'], str):
                    user['name'] = ' '.join(user['name'].strip().split())
                
                # Add full name if first and last name exist
                if 'first_name' in user and 'last_name' in user:
                    user['full_name'] = f"{user['first_name']} {user['last_name']}"
                
                # Convert dates
                if 'created_at' in user and isinstance(user['created_at'], str):
                    user['created_at'] = __import__('datetime').datetime.fromisoformat(user['created_at'].replace('Z', '+00:00'))
                
                if 'updated_at' in user and isinstance(user['updated_at'], str):
                    user['updated_at'] = __import__('datetime').datetime.fromisoformat(user['updated_at'].replace('Z', '+00:00'))
                
                # Calculate account age
                if 'created_at' in user and isinstance(user['created_at'], __import__('datetime').datetime):
                    user['account_age_days'] = ( __import__('datetime').datetime.now(user['created_at'].tzinfo) - user['created_at']).days
            
            # Transform products
            for i, product in enumerate(products):
                # Format name
                if 'name' in product and isinstance(product['name'], str):
                    product['name'] = product['name'].strip()
                
                # Format description
                if 'description' in product and isinstance(product['description'], str):
                    product['description'] = product['description'].strip()
                
                # Convert price to fixed decimal
                if 'price' in product and isinstance(product['price'], (int, float)):
                    product['price'] = round(product['price'], 2)
                
                # Convert dates
                if 'created_at' in product and isinstance(product['created_at'], str):
                    product['created_at'] = __import__('datetime').datetime.fromisoformat(product['created_at'].replace('Z', '+00:00'))
                
                if 'updated_at' in product and isinstance(product['updated_at'], str):
                    product['updated_at'] = __import__('datetime').datetime.fromisoformat(product['updated_at'].replace('Z', '+00:00'))
                
                # Add price range category
                if 'price' in product and isinstance(product['price'], (int, float)):
                    if product['price'] < 10:
                        product['price_range'] = 'budget'
                    elif product['price'] < 50:
                        product['price_range'] = 'mid-range'
                    elif product['price'] < 100:
                        product['price_range'] = 'premium'
                    else:
                        product['price_range'] = 'luxury'
            
            # Transform orders
            for i, order in enumerate(orders):
                # Convert dates
                if 'created_at' in order and isinstance(order['created_at'], str):
                    order['created_at'] = __import__('datetime').datetime.fromisoformat(order['created_at'].replace('Z', '+00:00'))
                
                if 'updated_at' in order and isinstance(order['updated_at'], str):
                    order['updated_at'] = __import__('datetime').datetime.fromisoformat(order['updated_at'].replace('Z', '+00:00'))
                
                if 'shipped_at' in order and isinstance(order['shipped_at'], str):
                    order['shipped_at'] = __import__('datetime').datetime.fromisoformat(order['shipped_at'].replace('Z', '+00:00'))
                
                if 'delivered_at' in order and isinstance(order['delivered_at'], str):
                    order['delivered_at'] = __import__('datetime').datetime.fromisoformat(order['delivered_at'].replace('Z', '+00:00'))
                
                # Calculate order age
                if 'created_at' in order and isinstance(order['created_at'], __import__('datetime').datetime):
                    order['order_age_days'] = (__import__('datetime').datetime.now(order['created_at'].tzinfo) - order['created_at']).days
                
                # Calculate processing time
                if 'created_at' in order and 'shipped_at' in order and isinstance(order['created_at'], __import__('datetime').datetime) and isinstance(order['shipped_at'], __import__('datetime').datetime):
                    order['processing_time_hours'] = (order['shipped_at'] - order['created_at']).total_seconds() / 3600
                
                # Calculate delivery time
                if 'shipped_at' in order and 'delivered_at' in order and isinstance(order['shipped_at'], __import__('datetime').datetime) and isinstance(order['delivered_at'], __import__('datetime').datetime):
                    order['delivery_time_days'] = (order['delivered_at'] - order['shipped_at']).total_seconds() / 86400
                
                # Calculate order total if not present
                if 'total' not in order and 'items' in order and isinstance(order['items'], list):
                    calculated_total = 0
                    for item in order['items']:
                        if 'price' in item and 'quantity' in item and isinstance(item['price'], (int, float)) and isinstance(item['quantity'], (int, float)):
                            calculated_total += item['price'] * item['quantity']
                    order['total'] = round(calculated_total, 2)
        
        # Steps 3-10 would continue with data enrichment, filtering, sorting, pagination, etc.
        # For brevity, we'll end here but in a real implementation these would be fully implemented.
        
        # Step 3: Data enrichment (simplified for this example)
        if enable_enrichment:
            print('Applying data enrichment...')
            enrichment_completed = True
            # In a full implementation, this would enrich data with additional information
        
        # Step 4: Data filtering (simplified for this example)
        if enable_filtering:
            print('Applying data filtering...')
            filtering_applied = True
            # In a full implementation, this would filter data based on criteria
        
        # Step 5: Data sorting (simplified for this example)
        if enable_sorting:
            print('Applying data sorting...')
            sorting_applied = True
            # In a full implementation, this would sort data based on criteria
        
        # Step 6: Pagination (simplified for this example)
        if enable_pagination:
            print('Applying pagination...')
            pagination_applied = True
            # In a full implementation, this would paginate the data
        
        # Prepare simplified result
        result['data'] = {
            'users': users,
            'products': products,
            'orders': orders
        }
        
        return result

    def deeply_nested_conditional_logic(self, data):
        """
        EXTREMELY LONG METHOD: Deeply nested conditional logic (250+ lines)
        This method demonstrates excessive conditional nesting that makes
        the code difficult to read and maintain.
        """
        print('Processing deeply nested conditional logic...')
        
        # Level 1: Initial data validation
        if data is not None:
            if isinstance(data, dict):
                if 'type' in data:
                    data_type = data['type']
                    
                    # Level 2: Type-specific processing
                    if data_type == 'user':
                        if 'id' in data:
                            user_id = data['id']
                            
                            # Level 3: User ID validation
                            if isinstance(user_id, int):
                                if user_id > 0:
                                    if user_id < 1000000:
                                        if 'profile' in data:
                                            profile = data['profile']
                                            
                                            # Level 4: Profile validation
                                            if isinstance(profile, dict):
                                                if 'name' in profile:
                                                    name = profile['name']
                                                    
                                                    # Level 5: Name validation
                                                    if isinstance(name, str):
                                                        if len(name) > 0:
                                                            if len(name) < 100:
                                                                if 'email' in profile:
                                                                    email = profile['email']
                                                                    
                                                                    # Level 6: Email validation
                                                                    if isinstance(email, str):
                                                                        if len(email) > 0:
                                                                            if len(email) < 255:
                                                                                if '@' in email:
                                                                                    if '.' in email:
                                                                                        if email.count('@') == 1:
                                                                                            # Valid user data
                                                                                            return {
                                                                                                'status': 'success',
                                                                                                'message': 'Valid user data',
                                                                                                'data': data
                                                                                            }
                                                                                        else:
                                                                                            return {
                                                                                                'status': 'error',
                                                                                                'message': 'Email must contain exactly one @ symbol',
                                                                                                'data': None
                                                                                            }
                                                                                    else:
                                                                                        return {
                                                                                            'status': 'error',
                                                                                            'message': 'Email must contain a dot',
                                                                                            'data': None
                                                                                        }
                                                                                else:
                                                                                    return {
                                                                                        'status': 'error',
                                                                                        'message': 'Email must contain @ symbol',
                                                                                        'data': None
                                                                                    }
                                                                            else:
                                                                                return {
                                                                                    'status': 'error',
                                                                                    'message': 'Email too long',
                                                                                    'data': None
                                                                                }
                                                                        else:
                                                                            return {
                                                                                'status': 'error',
                                                                                'message': 'Email too short',
                                                                                'data': None
                                                                            }
                                                                    else:
                                                                        return {
                                                                            'status': 'error',
                                                                            'message': 'Email must be a string',
                                                                            'data': None
                                                                        }
                                                                else:
                                                                    return {
                                                                        'status': 'error',
                                                                        'message': 'Email missing from profile',
                                                                        'data': None
                                                                    }
                                                            else:
                                                                return {
                                                                    'status': 'error',
                                                                    'message': 'Name too long',
                                                                    'data': None
                                                                }
                                                        else:
                                                            return {
                                                                'status': 'error',
                                                                'message': 'Name too short',
                                                                'data': None
                                                            }
                                                    else:
                                                        return {
                                                            'status': 'error',
                                                            'message': 'Name must be a string',
                                                            'data': None
                                                        }
                                                else:
                                                    return {
                                                        'status': 'error',
                                                        'message': 'Name missing from profile',
                                                        'data': None
                                                    }
                                            else:
                                                return {
                                                    'status': 'error',
                                                    'message': 'Profile must be a dictionary',
                                                    'data': None
                                                }
                                        else:
                                            return {
                                                'status': 'error',
                                                'message': 'Profile missing from data',
                                                'data': None
                                            }
                                    else:
                                        return {
                                            'status': 'error',
                                            'message': 'User ID too large',
                                            'data': None
                                        }
                                else:
                                    return {
                                        'status': 'error',
                                        'message': 'User ID must be positive',
                                        'data': None
                                    }
                            else:
                                return {
                                    'status': 'error',
                                    'message': 'User ID must be an integer',
                                    'data': None
                                }
                        else:
                            return {
                                'status': 'error',
                                'message': 'ID missing from user data',
                                'data': None
                            }
                    elif data_type == 'product':
                        # Similar nested logic for product type
                        if 'id' in data:
                            product_id = data['id']
                            
                            if isinstance(product_id, int):
                                if product_id > 0:
                                    if product_id < 1000000:
                                        if 'details' in data:
                                            details = data['details']
                                            
                                            if isinstance(details, dict):
                                                if 'name' in details:
                                                    name = details['name']
                                                    
                                                    if isinstance(name, str):
                                                        if len(name) > 0:
                                                            if len(name) < 100:
                                                                if 'price' in details:
                                                                    price = details['price']
                                                                    
                                                                    if isinstance(price, (int, float)):
                                                                        if price >= 0:
                                                                            if price < 1000000:
                                                                                # Valid product data
                                                                                return {
                                                                                    'status': 'success',
                                                                                    'message': 'Valid product data',
                                                                                    'data': data
                                                                                }
                                                                            else:
                                                                                return {
                                                                                    'status': 'error',
                                                                                    'message': 'Price too high',
                                                                                    'data': None
                                                                                }
                                                                        else:
                                                                            return {
                                                                                'status': 'error',
                                                                                'message': 'Price cannot be negative',
                                                                                'data': None
                                                                            }
                                                                    else:
                                                                        return {
                                                                            'status': 'error',
                                                                            'message': 'Price must be a number',
                                                                            'data': None
                                                                        }
                                                                else:
                                                                    return {
                                                                        'status': 'error',
                                                                        'message': 'Price missing from details',
                                                                        'data': None
                                                                    }
                                                            else:
                                                                return {
                                                                    'status': 'error',
                                                                    'message': 'Name too long',
                                                                    'data': None
                                                                }
                                                        else:
                                                            return {
                                                                'status': 'error',
                                                                'message': 'Name too short',
                                                                'data': None
                                                            }
                                                    else:
                                                        return {
                                                            'status': 'error',
                                                            'message': 'Name must be a string',
                                                            'data': None
                                                        }
                                                else:
                                                    return {
                                                        'status': 'error',
                                                        'message': 'Name missing from details',
                                                        'data': None
                                                    }
                                            else:
                                                return {
                                                    'status': 'error',
                                                    'message': 'Details must be a dictionary',
                                                    'data': None
                                                }
                                        else:
                                            return {
                                                'status': 'error',
                                                'message': 'Details missing from data',
                                                'data': None
                                            }
                                    else:
                                        return {
                                            'status': 'error',
                                            'message': 'Product ID too large',
                                            'data': None
                                        }
                                else:
                                    return {
                                        'status': 'error',
                                        'message': 'Product ID must be positive',
                                        'data': None
                                    }
                            else:
                                return {
                                    'status': 'error',
                                    'message': 'Product ID must be an integer',
                                    'data': None
                                }
                        else:
                            return {
                                'status': 'error',
                                'message': 'ID missing from product data',
                                'data': None
                            }
                    else:
                        return {
                            'status': 'error',
                            'message': 'Unknown data type',
                            'data': None
                        }
                else:
                    return {
                        'status': 'error',
                        'message': 'Type missing from data',
                        'data': None
                    }
            else:
                return {
                    'status': 'error',
                    'message': 'Data must be a dictionary',
                    'data': None
                }
        else:
            return {
                'status': 'error',
                'message': 'Data cannot be None',
                'data': None
            }

# Demonstrate the extremely long methods
def demonstrate_method_marathon():
    """Demonstrate the method marathon with extremely long methods"""
    print('Demonstrating method marathon...')
    
    marathon = MethodMarathon()
    
    # Test the extremely long method with nested logic
    try:
        test_data = {
            'items': [
                {'id': 1, 'value': 100, 'name': 'Item 1'},
                {'id': 2, 'value': 200, 'name': 'Item 2'},
                {'id': 3, 'value': 150, 'name': 'Item 3'}
            ]
        }
        
        result = marathon.extremely_long_method_with_nested_logic(test_data)
        print('Extremely long method result:', result['statistics'])
        
    except Exception as e:
        print('Error in extremely long method:', str(e))
    
    # Test the complex business logic processor
    try:
        config = {
            'enable_validation': True,
            'enable_transformation': True,
            'enable_enrichment': True
        }
        
        data = {
            'users': [
                {'id': 1, 'name': 'John Doe', 'email': 'john@example.com'},
                {'id': 2, 'name': 'Jane Smith', 'email': 'jane@example.com'}
            ],
            'products': [
                {'id': 1, 'name': 'Product 1', 'price': 29.99},
                {'id': 2, 'name': 'Product 2', 'price': 49.99}
            ],
            'orders': []
        }
        
        options = {
            'sort_by': 'id',
            'sort_order': 'asc'
        }
        
        result = marathon.complex_business_logic_processor(config, data, options)
        print('Complex business logic processor result keys:', list(result.keys()))
        
    except Exception as e:
        print('Error in complex business logic processor:', str(e))
    
    # Test the deeply nested conditional logic
    try:
        user_data = {
            'type': 'user',
            'id': 123,
            'profile': {
                'name': 'John Doe',
                'email': 'john@example.com'
            }
        }
        
        result = marathon.deeply_nested_conditional_logic(user_data)
        print('Deeply nested conditional logic result:', result['status'])
        
    except Exception as e:
        print('Error in deeply nested conditional logic:', str(e))

# Example usage
if __name__ == "__main__":
    demonstrate_method_marathon()
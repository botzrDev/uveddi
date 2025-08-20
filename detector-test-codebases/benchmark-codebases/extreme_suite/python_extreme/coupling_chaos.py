# coupling_chaos.py - Extreme Tight Coupling in Python
# This file demonstrates extreme tight coupling through circular imports and shared global state

# Global state shared across all modules - TIGHT COUPLING ANTIPATTERN
_global_state = {
    'user': None,
    'session': None,
    'permissions': [],
    'cache': {},
    'notifications': [],
    'ui_state': {
        'current_view': None,
        'modal_open': False,
        'loading': False
    }
}

class AuthModule:
    """Authentication module tightly coupled to global state and other modules"""
    
    @staticmethod
    def login(username, password):
        """Login method with tight coupling to multiple modules"""
        import session_module  # Circular import - TIGHT COUPLING
        import permission_module  # Circular import - TIGHT COUPLING
        import cache_module  # Circular import - TIGHT COUPLING
        import notification_module  # Circular import - TIGHT COUPLING
        import ui_module  # Circular import - TIGHT COUPLING
        import logging_module  # Circular import - TIGHT COUPLING
        
        if username and password:
            # Tightly coupled to global state
            _global_state['user'] = {
                'id': 1,
                'username': username,
                'email': f"{username}@example.com"
            }
            
            # Tightly coupled to session module
            session = session_module.SessionModule.create_session(_global_state['user'])
            
            # Tightly coupled to permission module
            permissions = permission_module.PermissionModule.load_user_permissions(_global_state['user']['id'])
            
            # Tightly coupled to cache module
            cache_module.CacheModule.set('user_permissions', permissions)
            
            # Tightly coupled to notification module
            notification_module.NotificationModule.add_notification('Login successful', 'success')
            
            # Tightly coupled to UI module
            ui_module.UIModule.update_user_interface()
            
            # Tightly coupled to logging module
            logging_module.LoggingModule.log(f"User logged in: {username}", 'auth')
            
            return {
                'user': _global_state['user'],
                'session': session,
                'permissions': permissions
            }
        else:
            # Tightly coupled to notification module
            notification_module.NotificationModule.add_notification('Login failed', 'error')
            
            # Tightly coupled to UI module
            ui_module.UIModule.show_login_error('Invalid credentials')
            
            # Tightly coupled to logging module
            logging_module.LoggingModule.log(f"Login failed for: {username}", 'auth')
            
            raise Exception('Invalid credentials')
    
    @staticmethod
    def logout():
        """Logout method with tight coupling to multiple modules"""
        import session_module  # Circular import - TIGHT COUPLING
        import permission_module  # Circular import - TIGHT COUPLING
        import cache_module  # Circular import - TIGHT COUPLING
        import notification_module  # Circular import - TIGHT COUPLING
        import ui_module  # Circular import - TIGHT COUPLING
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to session module
        session_module.SessionModule.destroy_session(_global_state['session'])
        
        # Tightly coupled to global state
        _global_state['user'] = None
        _global_state['session'] = None
        _global_state['permissions'] = []
        
        # Tightly coupled to cache module
        cache_module.CacheModule.clear()
        
        # Tightly coupled to notification module
        notification_module.NotificationModule.add_notification('Logged out successfully', 'info')
        
        # Tightly coupled to UI module
        ui_module.UIModule.reset_user_interface()
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log('User logged out', 'auth')

class SessionModule:
    """Session module tightly coupled to global state and other modules"""
    
    @staticmethod
    def create_session(user):
        """Create session with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        session = {
            'id': f"sess_{int(__import__('time').time() * 1000)}",
            'user_id': user['id'],
            'created_at': __import__('datetime').datetime.now(),
            'expires_at': __import__('datetime').datetime.now() + __import__('datetime').timedelta(hours=1)
        }
        
        # Tightly coupled to global state
        _global_state['session'] = session
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Session created for user: {user['id']}", 'session')
        
        return session
    
    @staticmethod
    def destroy_session(session):
        """Destroy session with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        if _global_state['session'] and _global_state['session']['id'] == session['id']:
            _global_state['session'] = None
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Session destroyed: {session['id']}", 'session')
    
    @staticmethod
    def validate_session():
        """Validate session with tight coupling to global state"""
        import auth_module  # Circular import - TIGHT COUPLING
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        if not _global_state['session']:
            raise Exception('No session found')
        
        # Tightly coupled to global state
        if _global_state['session']['expires_at'] < __import__('datetime').datetime.now():
            # Tightly coupled to auth module
            auth_module.AuthModule.logout()
            raise Exception('Session expired')
        
        return _global_state['session']

class PermissionModule:
    """Permission module tightly coupled to global state and cache"""
    
    @staticmethod
    def load_user_permissions(user_id):
        """Load user permissions with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Simulate database query tightly coupled to global state
        permissions = [
            'read_user', 'write_user', 'delete_user',
            'read_admin', 'write_admin', 'delete_admin',
            'read_config', 'write_config', 'delete_config',
            'read_logs', 'write_logs', 'delete_logs',
            'read_reports', 'write_reports', 'delete_reports',
            'read_analytics', 'write_analytics', 'delete_analytics',
            'read_security', 'write_security', 'delete_security'
        ] * 2  # 42 permissions to exceed threshold
        
        # Tightly coupled to global state
        _global_state['permissions'] = permissions
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Loaded {len(permissions)} permissions for user: {user_id}", 'permissions')
        
        return permissions
    
    @staticmethod
    def has_permission(permission):
        """Check permission with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        has_perm = permission in _global_state['permissions']
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Permission check: {permission} = {has_perm}", 'permissions')
        
        return has_perm
    
    @staticmethod
    def check_permission_and_execute(permission, action):
        """Check permission and execute action with tight coupling"""
        import notification_module  # Circular import - TIGHT COUPLING
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to permission method
        if not PermissionModule.has_permission(permission):
            # Tightly coupled to notification module
            notification_module.NotificationModule.add_notification('Insufficient permissions', 'error')
            
            # Tightly coupled to logging module
            logging_module.LoggingModule.log(f"Permission denied: {permission}", 'permissions')
            
            raise Exception('Insufficient permissions')
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Executing action with permission: {permission}", 'permissions')
        
        # Tightly coupled to action execution
        return action()

class CacheModule:
    """Cache module tightly coupled to global state"""
    
    @staticmethod
    def set(key, value):
        """Set cache value with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        _global_state['cache'][key] = value
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Cached item: {key}", 'cache')
    
    @staticmethod
    def get(key):
        """Get cache value with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        value = _global_state['cache'].get(key)
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Retrieved cached item: {key}", 'cache')
        
        return value
    
    @staticmethod
    def clear():
        """Clear cache with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        _global_state['cache'].clear()
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log('Cache cleared', 'cache')

class NotificationModule:
    """Notification module tightly coupled to global state"""
    
    @staticmethod
    def add_notification(message, notification_type):
        """Add notification with tight coupling to global state"""
        import ui_module  # Circular import - TIGHT COUPLING
        import logging_module  # Circular import - TIGHT COUPLING
        
        notification = {
            'id': int(__import__('time').time() * 1000),
            'message': message,
            'type': notification_type,
            'timestamp': __import__('datetime').datetime.now()
        }
        
        # Tightly coupled to global state
        _global_state['notifications'].append(notification)
        
        # Tightly coupled to UI module
        ui_module.UIModule.show_notification(notification)
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Notification added: {message}", 'notification')
    
    @staticmethod
    def remove_notification(notification_id):
        """Remove notification with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to global state
        _global_state['notifications'] = [
            n for n in _global_state['notifications'] 
            if n['id'] != notification_id
        ]
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Notification removed: {notification_id}", 'notification')

class UIModule:
    """UI module tightly coupled to global state"""
    
    @staticmethod
    def update_user_interface():
        """Update UI with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Simulate DOM manipulation tightly coupled to global state
        _global_state['ui_state']['current_view'] = 'dashboard'
        _global_state['ui_state']['loading'] = False
        
        # Tightly coupled to logging module
        user_name = _global_state['user']['username'] if _global_state['user'] else 'Unknown'
        logging_module.LoggingModule.log(f"UI updated for user: {user_name}", 'ui')
    
    @staticmethod
    def show_login_error(message):
        """Show login error with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Simulate DOM manipulation tightly coupled to global state
        _global_state['ui_state']['loading'] = False
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Login error displayed: {message}", 'ui')
    
    @staticmethod
    def reset_user_interface():
        """Reset UI with tight coupling to global state"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Simulate DOM manipulation tightly coupled to global state
        _global_state['ui_state']['current_view'] = 'login'
        _global_state['ui_state']['modal_open'] = False
        _global_state['ui_state']['loading'] = False
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log('UI reset to login view', 'ui')
    
    @staticmethod
    def show_notification(notification):
        """Show notification with tight coupling to modules"""
        import logging_module  # Circular import - TIGHT COUPLING
        
        # Simulate DOM manipulation for notification
        print(f"Showing notification: {notification['message']}")
        
        # Tightly coupled to logging module
        logging_module.LoggingModule.log(f"Notification displayed: {notification['message']}", 'ui')

class LoggingModule:
    """Logging module with tight coupling to all other modules"""
    
    @staticmethod
    def log(message, category):
        """Log message with tight coupling to all modules"""
        import notification_module  # Circular import - TIGHT COUPLING
        import cache_module  # Circular import - TIGHT COUPLING
        import ui_module  # Circular import - TIGHT COUPLING
        
        log_entry = {
            'timestamp': __import__('datetime').datetime.now().isoformat(),
            'message': message,
            'category': category,
            'user_id': _global_state['user']['id'] if _global_state['user'] else None,
            'session_id': _global_state['session']['id'] if _global_state['session'] else None,
            'permissions': len(_global_state['permissions']),
            'cache_size': len(_global_state['cache']),
            'notification_count': len(_global_state['notifications']),
            'current_view': _global_state['ui_state']['current_view']
        }
        
        # Simulate writing to log file with tight coupling to all modules
        print(f"LOG: {log_entry}")
        
        # Tightly coupled to notification module for error notifications
        if category == 'error':
            notification_module.NotificationModule.add_notification(f"Error occurred: {message}", 'error')
            
            # Tightly coupled to cache module for caching errors
            cache_module.CacheModule.set('last_error', log_entry)
            
            # Tightly coupled to UI module for error UI updates
            ui_module.UIModule.show_notification({'message': 'Error logged', 'type': 'info'})

# Circular dependency simulation - Module A depends on B, B depends on C, C depends on A
class ModuleA:
    """Module A with circular dependencies"""
    
    @staticmethod
    def process_data(data):
        import module_b  # Circular import - TIGHT COUPLING
        
        is_valid = module_b.ModuleB.validate_data(data)
        if is_valid:
            import module_c  # Circular import - TIGHT COUPLING
            return module_c.ModuleC.transform_data(data)
        else:
            raise Exception('Invalid data')

class ModuleB:
    """Module B with circular dependencies"""
    
    @staticmethod
    def validate_data(data):
        import module_c  # Circular import - TIGHT COUPLING
        
        is_correct_format = module_c.ModuleC.check_data_format(data)
        return is_correct_format and len(data) > 0

class ModuleC:
    """Module C with circular dependencies"""
    
    @staticmethod
    def transform_data(data):
        import module_a  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to ModuleA
        module_a.ModuleA.process_data(data[:len(data)//2])
        
        return [item.upper() if hasattr(item, 'upper') else item for item in data]
    
    @staticmethod
    def check_data_format(data):
        import module_a  # Circular import - TIGHT COUPLING
        
        # Tightly coupled to ModuleA
        module_a.ModuleA.process_data([])
        
        return isinstance(data, list)

# Demonstrate tight coupling by creating instances and calling methods
def demonstrate_tight_coupling_chaos():
    """Demonstrate the tight coupling chaos"""
    print('Demonstrating tight coupling chaos...')
    
    # This would create circular import issues, so we'll just show the structure
    print('_global_state:', _global_state)
    print('AuthModule:', AuthModule)
    print('SessionModule:', SessionModule)
    print('PermissionModule:', PermissionModule)
    print('CacheModule:', CacheModule)
    print('NotificationModule:', NotificationModule)
    print('UIModule:', UIModule)
    print('LoggingModule:', LoggingModule)
    print('ModuleA:', ModuleA)
    print('ModuleB:', ModuleB)
    print('ModuleC:', ModuleC)

# Example usage that showcases tight coupling issues
if __name__ == "__main__":
    demonstrate_tight_coupling_chaos()
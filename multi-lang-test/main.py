#!/usr/bin/env python3
"""
Main script demonstrating usage of problematic code
"""

from src.god_object import MassiveApplicationManager


def main():
    """Main function"""
    manager = MassiveApplicationManager()
    
    # Use the god object
    request_data = {
        'id': 'req_123',
        'user_id': 'user_456',
        'input_file': 'test_input.txt',
        'window_id': 'main_window',
        'transaction': {
            'id': 'tx_789',
            'type': 'payment',
            'amount': 100.0
        },
        'session_data': {
            'theme': 'dark',
            'language': 'en'
        }
    }
    
    result = manager.handle_application_request(request_data)
    print(f"Request result: {result}")
    
    # This calls active code (not dead)
    print(f"Used helper result: {manager.user_sessions}")


if __name__ == "__main__":
    main()
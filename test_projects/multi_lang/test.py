#!/usr/bin/env python3

def main():
    print("Hello from Python!")

def unused_function():
    """This function is never called - should be detected as dead code"""
    print("Never called")

class LargeClass:
    """A deliberately large class to test detection"""
    
    def __init__(self):
        self.field1 = 1
        self.field2 = "test"
        self.field3 = []
        self.field4 = None
        self.field5 = True
        self.field6 = 3.14
        self.field7 = []
        self.field8 = 42
        self.field9 = 99
        self.field10 = "more"
        
    def method1(self): return self.field1
    def method2(self): return self.field2
    def method3(self): return self.field3
    def method4(self): return self.field4
    def method5(self): return self.field5
    def method6(self): return self.field6
    def method7(self): return self.field7
    def method8(self): return self.field8
    def method9(self): return self.field9
    def method10(self): return self.field10
    def method11(self): return "extra"
    def method12(self): return "more methods"
    # Large class - should be detected

if __name__ == "__main__":
    main()
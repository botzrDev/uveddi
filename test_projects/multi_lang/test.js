// JavaScript test file

function main() {
    console.log("Hello from JavaScript!");
}

function unusedFunction() {
    // This function is never called - should be detected as dead code
    console.log("Never called");
}

class LargeClass {
    constructor() {
        this.field1 = 1;
        this.field2 = "test";
        this.field3 = [];
        this.field4 = null;
        this.field5 = true;
        this.field6 = 3.14;
        this.field7 = [];
        this.field8 = 42;
        this.field9 = 99;
        this.field10 = "more";
        this.field11 = "extra";
        this.field12 = "data";
    }
    
    method1() { return this.field1; }
    method2() { return this.field2; }
    method3() { return this.field3; }
    method4() { return this.field4; }
    method5() { return this.field5; }
    method6() { return this.field6; }
    method7() { return this.field7; }
    method8() { return this.field8; }
    method9() { return this.field9; }
    method10() { return this.field10; }
    method11() { return this.field11; }
    method12() { return this.field12; }
    method13() { return "extra method"; }
    method14() { return "more methods"; }
    method15() { return "even more"; }
    // Large class - should be detected
}

main();
// TypeScript test file

interface TestInterface {
    field1: number;
    field2: string;
}

function main(): void {
    console.log("Hello from TypeScript!");
}

function unusedFunction(): void {
    // This function is never called - should be detected as dead code
    console.log("Never called");
}

class LargeClass implements TestInterface {
    field1: number;
    field2: string;
    private field3: Array<number>;
    private field4: string | null;
    private field5: boolean;
    private field6: number;
    private field7: Array<string>;
    private field8: number;
    private field9: number;
    private field10: string;
    private field11: string;
    private field12: string;
    
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
    
    method1(): number { return this.field1; }
    method2(): string { return this.field2; }
    method3(): Array<number> { return this.field3; }
    method4(): string | null { return this.field4; }
    method5(): boolean { return this.field5; }
    method6(): number { return this.field6; }
    method7(): Array<string> { return this.field7; }
    method8(): number { return this.field8; }
    method9(): number { return this.field9; }
    method10(): string { return this.field10; }
    method11(): string { return this.field11; }
    method12(): string { return this.field12; }
    method13(): string { return "extra method"; }
    method14(): string { return "more methods"; }
    method15(): string { return "even more"; }
    method16(): string { return "way too many methods"; }
    // Large class - should be detected
}

main();
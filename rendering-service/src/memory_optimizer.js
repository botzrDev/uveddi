// UV-8: Advanced Memory Optimization for Rendering Service
// Implements object pooling and memory management for hot paths
// Related Jira: UV-8, UV-145, UV-148

class ObjectPool {
  constructor(createFn, size = 10) {
    this.createFn = createFn;
    this.pool = [];
    for (let i = 0; i < size; i++) {
      this.pool.push(this.createFn());
    }
  }
  acquire() {
    return this.pool.length ? this.pool.pop() : this.createFn();
  }
  release(obj) {
    this.pool.push(obj);
  }
  size() {
    return this.pool.length;
  }
}

// Example: Pool for rendering buffers (can be extended for diagram objects)
const bufferPool = new ObjectPool(() => Buffer.alloc(1024 * 1024), 4); // 1MB buffers

module.exports = { ObjectPool, bufferPool };

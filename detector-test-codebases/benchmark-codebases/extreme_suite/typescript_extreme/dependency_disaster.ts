// dependency_disaster.ts - Extreme Tight Coupling in TypeScript
// This file demonstrates extreme tight coupling through circular dependencies and shared global state

// Global state shared across all modules - TIGHT COUPLING ANTIPATTERN
interface GlobalState {
  user: any;
  session: any;
  permissions: string[];
  cache: Map<string, any>;
  notifications: any[];
  uiState: {
    currentView: string | null;
    modalOpen: boolean;
    loading: boolean;
  };
}

const globalState: GlobalState = {
  user: null,
  session: null,
  permissions: [],
  cache: new Map(),
  notifications: [],
  uiState: {
    currentView: null,
    modalOpen: false,
    loading: false
  }
};

class AuthModule {
  /** Authentication module tightly coupled to global state and other modules */
  
  static login(username: string, password: string): Promise<any> {
    return new Promise((resolve, reject) => {
      // Dynamically import modules to simulate circular dependencies - TIGHT COUPLING
      import('./dependency_disaster').then(() => {
        if (username && password) {
          // Tightly coupled to global state
          globalState.user = {
            id: 1,
            username: username,
            email: `${username}@example.com`
          };
          
          // Tightly coupled to session module
          SessionModule.createSession(globalState.user)
            .then((session: any) => {
              // Tightly coupled to permission module
              return PermissionModule.loadUserPermissions(globalState.user.id);
            })
            .then((permissions: string[]) => {
              // Tightly coupled to cache module
              return CacheModule.set('user_permissions', permissions);
            })
            .then(() => {
              // Tightly coupled to notification module
              return NotificationModule.addNotification('Login successful', 'success');
            })
            .then(() => {
              // Tightly coupled to UI module
              return UIModule.updateUserInterface();
            })
            .then(() => {
              // Tightly coupled to logging module
              return LoggingModule.log(`User logged in: ${username}`, 'auth');
            })
            .then(() => {
              resolve({
                user: globalState.user,
                session: globalState.session,
                permissions: globalState.permissions
              });
            })
            .catch((error: any) => {
              reject(error);
            });
        } else {
          // Tightly coupled to notification module
          NotificationModule.addNotification('Login failed', 'error')
            .then(() => {
              // Tightly coupled to UI module
              return UIModule.showLoginError('Invalid credentials');
            })
            .then(() => {
              // Tightly coupled to logging module
              return LoggingModule.log(`Login failed for: ${username}`, 'auth');
            })
            .then(() => {
              reject(new Error('Invalid credentials'));
            })
            .catch((error: any) => {
              reject(error);
            });
        }
      });
    });
  }
  
  static logout(): Promise<void> {
    return new Promise((resolve, reject) => {
      // Tightly coupled to session module
      SessionModule.destroySession(globalState.session)
        .then(() => {
          // Tightly coupled to global state
          globalState.user = null;
          globalState.session = null;
          globalState.permissions = [];
          
          // Tightly coupled to cache module
          return CacheModule.clear();
        })
        .then(() => {
          // Tightly coupled to notification module
          return NotificationModule.addNotification('Logged out successfully', 'info');
        })
        .then(() => {
          // Tightly coupled to UI module
          return UIModule.resetUserInterface();
        })
        .then(() => {
          // Tightly coupled to logging module
          return LoggingModule.log('User logged out', 'auth');
        })
        .then(() => {
          resolve();
        })
        .catch((error: any) => {
          reject(error);
        });
    });
  }
}

class SessionModule {
  /** Session module tightly coupled to global state and other modules */
  
  static createSession(user: any): Promise<any> {
    return new Promise((resolve) => {
      const session = {
        id: `sess_${Date.now()}`,
        userId: user.id,
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 3600000) // 1 hour
      };
      
      // Tightly coupled to global state
      globalState.session = session;
      
      // Tightly coupled to logging module
      LoggingModule.log(`Session created for user: ${user.id}`, 'session')
        .then(() => {
          resolve(session);
        });
    });
  }
  
  static destroySession(session: any): Promise<void> {
    return new Promise((resolve) => {
      // Tightly coupled to global state
      if (globalState.session && globalState.session.id === session.id) {
        globalState.session = null;
      }
      
      // Tightly coupled to logging module
      LoggingModule.log(`Session destroyed: ${session.id}`, 'session')
        .then(() => {
          resolve();
        });
    });
  }
  
  static validateSession(): Promise<any> {
    return new Promise((resolve, reject) => {
      // Tightly coupled to global state
      if (!globalState.session) {
        reject(new Error('No session found'));
        return;
      }
      
      // Tightly coupled to global state
      if (globalState.session.expiresAt < new Date()) {
        // Tightly coupled to auth module
        AuthModule.logout()
          .then(() => {
            reject(new Error('Session expired'));
          })
          .catch(() => {
            reject(new Error('Session expired'));
          });
        return;
      }
      
      resolve(globalState.session);
    });
  }
}

class PermissionModule {
  /** Permission module tightly coupled to global state and cache */
  
  static loadUserPermissions(userId: number): Promise<string[]> {
    return new Promise((resolve) => {
      // Simulate database query tightly coupled to global state
      const permissions: string[] = [
        'read_user', 'write_user', 'delete_user',
        'read_admin', 'write_admin', 'delete_admin',
        'read_config', 'write_config', 'delete_config',
        'read_logs', 'write_logs', 'delete_logs',
        'read_reports', 'write_reports', 'delete_reports',
        'read_analytics', 'write_analytics', 'delete_analytics',
        'read_security', 'write_security', 'delete_security'
      ];
      
      // Duplicate permissions to exceed threshold
      const duplicatedPermissions = [...permissions, ...permissions];
      
      // Tightly coupled to global state
      globalState.permissions = duplicatedPermissions;
      
      // Tightly coupled to logging module
      LoggingModule.log(`Loaded ${duplicatedPermissions.length} permissions for user: ${userId}`, 'permissions')
        .then(() => {
          resolve(duplicatedPermissions);
        });
    });
  }
  
  static hasPermission(permission: string): Promise<boolean> {
    return new Promise((resolve) => {
      // Tightly coupled to global state
      const hasPerm = globalState.permissions.includes(permission);
      
      // Tightly coupled to logging module
      LoggingModule.log(`Permission check: ${permission} = ${hasPerm}`, 'permissions')
        .then(() => {
          resolve(hasPerm);
        });
    });
  }
  
  static checkPermissionAndExecute<T>(permission: string, action: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      // Tightly coupled to permission method
      PermissionModule.hasPermission(permission)
        .then((hasPerm: boolean) => {
          if (!hasPerm) {
            // Tightly coupled to notification module
            return NotificationModule.addNotification('Insufficient permissions', 'error')
              .then(() => {
                // Tightly coupled to logging module
                return LoggingModule.log(`Permission denied: ${permission}`, 'permissions');
              })
              .then(() => {
                throw new Error('Insufficient permissions');
              });
          }
          
          // Tightly coupled to logging module
          return LoggingModule.log(`Executing action with permission: ${permission}`, 'permissions')
            .then(() => {
              // Tightly coupled to action execution
              return action();
            });
        })
        .then((result: T) => {
          resolve(result);
        })
        .catch((error: any) => {
          reject(error);
        });
    });
  }
}

class CacheModule {
  /** Cache module tightly coupled to global state */
  
  static set(key: string, value: any): Promise<void> {
    return new Promise((resolve) => {
      // Tightly coupled to global state
      globalState.cache.set(key, value);
      
      // Tightly coupled to logging module
      LoggingModule.log(`Cached item: ${key}`, 'cache')
        .then(() => {
          resolve();
        });
    });
  }
  
  static get(key: string): Promise<any> {
    return new Promise((resolve) => {
      // Tightly coupled to global state
      const value = globalState.cache.get(key);
      
      // Tightly coupled to logging module
      LoggingModule.log(`Retrieved cached item: ${key}`, 'cache')
        .then(() => {
          resolve(value);
        });
    });
  }
  
  static clear(): Promise<void> {
    return new Promise((resolve) => {
      // Tightly coupled to global state
      globalState.cache.clear();
      
      // Tightly coupled to logging module
      LoggingModule.log('Cache cleared', 'cache')
        .then(() => {
          resolve();
        });
    });
  }
}

class NotificationModule {
  /** Notification module tightly coupled to global state */
  
  static addNotification(message: string, type: string): Promise<void> {
    return new Promise((resolve) => {
      const notification = {
        id: Date.now(),
        message: message,
        type: type,
        timestamp: new Date()
      };
      
      // Tightly coupled to global state
      globalState.notifications.push(notification);
      
      // Tightly coupled to UI module
      UIModule.showNotification(notification)
        .then(() => {
          // Tightly coupled to logging module
          return LoggingModule.log(`Notification added: ${message}`, 'notification');
        })
        .then(() => {
          resolve();
        });
    });
  }
  
  static removeNotification(notificationId: number): Promise<void> {
    return new Promise((resolve) => {
      // Tightly coupled to global state
      globalState.notifications = globalState.notifications.filter(
        n => n.id !== notificationId
      );
      
      // Tightly coupled to logging module
      LoggingModule.log(`Notification removed: ${notificationId}`, 'notification')
        .then(() => {
          resolve();
        });
    });
  }
}

class UIModule {
  /** UI module tightly coupled to global state */
  
  static updateUserInterface(): Promise<void> {
    return new Promise((resolve) => {
      // Simulate DOM manipulation tightly coupled to global state
      globalState.uiState.currentView = 'dashboard';
      globalState.uiState.loading = false;
      
      // Tightly coupled to logging module
      const userName = globalState.user ? globalState.user.username : 'Unknown';
      LoggingModule.log(`UI updated for user: ${userName}`, 'ui')
        .then(() => {
          resolve();
        });
    });
  }
  
  static showLoginError(message: string): Promise<void> {
    return new Promise((resolve) => {
      // Simulate DOM manipulation tightly coupled to global state
      globalState.uiState.loading = false;
      
      // Tightly coupled to logging module
      LoggingModule.log(`Login error displayed: ${message}`, 'ui')
        .then(() => {
          resolve();
        });
    });
  }
  
  static resetUserInterface(): Promise<void> {
    return new Promise((resolve) => {
      // Simulate DOM manipulation tightly coupled to global state
      globalState.uiState.currentView = 'login';
      globalState.uiState.modalOpen = false;
      globalState.uiState.loading = false;
      
      // Tightly coupled to logging module
      LoggingModule.log('UI reset to login view', 'ui')
        .then(() => {
          resolve();
        });
    });
  }
  
  static showNotification(notification: any): Promise<void> {
    return new Promise((resolve) => {
      // Simulate DOM manipulation for notification
      console.log('Showing notification:', notification.message);
      
      // Tightly coupled to logging module
      LoggingModule.log(`Notification displayed: ${notification.message}`, 'ui')
        .then(() => {
          resolve();
        });
    });
  }
}

class LoggingModule {
  /** Logging module with tight coupling to all other modules */
  
  static log(message: string, category: string): Promise<void> {
    return new Promise((resolve) => {
      const logEntry = {
        timestamp: new Date().toISOString(),
        message: message,
        category: category,
        userId: globalState.user ? globalState.user.id : null,
        sessionId: globalState.session ? globalState.session.id : null,
        permissions: globalState.permissions.length,
        cacheSize: globalState.cache.size,
        notificationCount: globalState.notifications.length,
        currentView: globalState.uiState.currentView
      };
      
      // Simulate writing to log file with tight coupling to all modules
      console.log('LOG:', JSON.stringify(logEntry));
      
      // Tightly coupled to notification module for error notifications
      if (category === 'error') {
        NotificationModule.addNotification(`Error occurred: ${message}`, 'error')
          .then(() => {
            // Tightly coupled to cache module for caching errors
            return CacheModule.set('last_error', logEntry);
          })
          .then(() => {
            // Tightly coupled to UI module for error UI updates
            return UIModule.showNotification({ message: 'Error logged', type: 'info' });
          })
          .then(() => {
            resolve();
          });
      } else {
        resolve();
      }
    });
  }
}

// Circular dependency simulation - Module A depends on B, B depends on C, C depends on A
class ModuleA {
  /** Module A with circular dependencies */
  
  static processData(data: any[]): Promise<any[]> {
    return import('./dependency_disaster').then(() => {
      return ModuleB.validateData(data)
        .then((isValid: boolean) => {
          if (isValid) {
            return ModuleC.transformData(data);
          } else {
            throw new Error('Invalid data');
          }
        });
    });
  }
}

class ModuleB {
  /** Module B with circular dependencies */
  
  static validateData(data: any[]): Promise<boolean> {
    return import('./dependency_disaster').then(() => {
      return ModuleC.checkDataFormat(data)
        .then((isCorrectFormat: boolean) => {
          return isCorrectFormat && data.length > 0;
        });
    });
  }
}

class ModuleC {
  /** Module C with circular dependencies */
  
  static transformData(data: any[]): Promise<any[]> {
    return import('./dependency_disaster').then(() => {
      // Tightly coupled to ModuleA
      return ModuleA.processData(data.slice(0, Math.floor(data.length / 2)))
        .then(() => {
          return data.map(item => typeof item === 'string' ? item.toUpperCase() : item);
        });
    });
  }
  
  static checkDataFormat(data: any[]): Promise<boolean> {
    return import('./dependency_disaster').then(() => {
      // Tightly coupled to ModuleA
      return ModuleA.processData([])
        .then(() => {
          return Array.isArray(data);
        });
    });
  }
}

// Demonstrate tight coupling by calling methods
function demonstrateDependencyDisaster(): void {
  console.log('Demonstrating dependency disaster...');
  
  // This would create circular import issues, so we'll just show the structure
  console.log('Global state:', globalState);
  console.log('AuthModule:', AuthModule);
  console.log('SessionModule:', SessionModule);
  console.log('PermissionModule:', PermissionModule);
  console.log('CacheModule:', CacheModule);
  console.log('NotificationModule:', NotificationModule);
  console.log('UIModule:', UIModule);
  console.log('LoggingModule:', LoggingModule);
  console.log('ModuleA:', ModuleA);
  console.log('ModuleB:', ModuleB);
  console.log('ModuleC:', ModuleC);
}

// Example usage that showcases tight coupling issues
if (require.main === module) {
  demonstrateDependencyDisaster();
}

export {
  globalState,
  AuthModule,
  SessionModule,
  PermissionModule,
  CacheModule,
  NotificationModule,
  UIModule,
  LoggingModule,
  ModuleA,
  ModuleB,
  ModuleC,
  demonstrateDependencyDisaster
};
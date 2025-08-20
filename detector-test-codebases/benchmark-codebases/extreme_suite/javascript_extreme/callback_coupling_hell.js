// callback_coupling_hell.js - Extreme Tight Coupling with Callbacks
// This file demonstrates extreme tight coupling through nested callbacks and shared global state

// Global state shared across all modules - TIGHT COUPLING ANTIPATTERN
const GlobalStateManager = {
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

// Authentication module tightly coupled to global state and other modules
const AuthModule = {
  // Directly modifies global state - TIGHT COUPLING
  login: function(username, password, callback) {
    // Simulate API call
    setTimeout(() => {
      if (username && password) {
        // Tightly coupled to global state
        GlobalStateManager.user = {
          id: 1,
          username: username,
          email: username + '@example.com'
        };
        
        // Tightly coupled to session module
        SessionModule.createSession(GlobalStateManager.user, (sessionError, session) => {
          if (sessionError) {
            callback(sessionError, null);
            return;
          }
          
          // Tightly coupled to permission module
          PermissionModule.loadUserPermissions(GlobalStateManager.user.id, (permError, perms) => {
            if (permError) {
              callback(permError, null);
              return;
            }
            
            // Tightly coupled to cache module
            CacheModule.set('user_permissions', perms, (cacheError) => {
              if (cacheError) {
                callback(cacheError, null);
                return;
              }
              
              // Tightly coupled to notification module
              NotificationModule.addNotification('Login successful', 'success', () => {
                // Tightly coupled to UI module
                UIModule.updateUserInterface(() => {
                  // Tightly coupled to logging module
                  LoggingModule.log('User logged in: ' + username, 'auth', () => {
                    callback(null, {
                      user: GlobalStateManager.user,
                      session: session,
                      permissions: perms
                    });
                  });
                });
              });
            });
          });
        });
      } else {
        // Tightly coupled to notification module
        NotificationModule.addNotification('Login failed', 'error', () => {
          // Tightly coupled to UI module
          UIModule.showLoginError('Invalid credentials', () => {
            // Tightly coupled to logging module
            LoggingModule.log('Login failed for: ' + username, 'auth', () => {
              callback(new Error('Invalid credentials'), null);
            });
          });
        });
      }
    }, 100);
  },
  
  // Tightly coupled to multiple modules
  logout: function(callback) {
    // Tightly coupled to session module
    SessionModule.destroySession(GlobalStateManager.session, (sessionError) => {
      if (sessionError) {
        callback(sessionError);
        return;
      }
      
      // Tightly coupled to global state
      GlobalStateManager.user = null;
      GlobalStateManager.session = null;
      GlobalStateManager.permissions = [];
      
      // Tightly coupled to cache module
      CacheModule.clear((cacheError) => {
        if (cacheError) {
          callback(cacheError);
          return;
        }
        
        // Tightly coupled to notification module
        NotificationModule.addNotification('Logged out successfully', 'info', () => {
          // Tightly coupled to UI module
          UIModule.resetUserInterface(() => {
            // Tightly coupled to logging module
            LoggingModule.log('User logged out', 'auth', () => {
              callback(null);
            });
          });
        });
      });
    });
  }
};

// Session module tightly coupled to global state and other modules
const SessionModule = {
  // Tightly coupled to global state
  createSession: function(user, callback) {
    setTimeout(() => {
      const session = {
        id: 'sess_' + Date.now(),
        userId: user.id,
        createdAt: new Date(),
        expiresAt: new Date(Date.now() + 3600000) // 1 hour
      };
      
      // Tightly coupled to global state
      GlobalStateManager.session = session;
      
      // Tightly coupled to logging module
      LoggingModule.log('Session created for user: ' + user.id, 'session', () => {
        callback(null, session);
      });
    }, 50);
  },
  
  // Tightly coupled to global state
  destroySession: function(session, callback) {
    setTimeout(() => {
      // Tightly coupled to global state
      if (GlobalStateManager.session && GlobalStateManager.session.id === session.id) {
        GlobalStateManager.session = null;
      }
      
      // Tightly coupled to logging module
      LoggingModule.log('Session destroyed: ' + session.id, 'session', () => {
        callback(null);
      });
    }, 50);
  },
  
  // Tightly coupled to global state
  validateSession: function(callback) {
    // Tightly coupled to global state
    if (!GlobalStateManager.session) {
      callback(new Error('No session found'));
      return;
    }
    
    // Tightly coupled to global state
    if (GlobalStateManager.session.expiresAt < new Date()) {
      // Tightly coupled to auth module
      AuthModule.logout((logoutError) => {
        callback(new Error('Session expired'));
      });
      return;
    }
    
    callback(null, GlobalStateManager.session);
  }
};

// Permission module tightly coupled to global state and cache
const PermissionModule = {
  // Tightly coupled to global state and database simulation
  loadUserPermissions: function(userId, callback) {
    setTimeout(() => {
      // Simulate database query tightly coupled to global state
      const permissions = [
        'read_user',
        'write_user',
        'delete_user',
        'read_admin',
        'write_admin',
        'delete_admin',
        'read_config',
        'write_config',
        'delete_config',
        'read_logs',
        'write_logs',
        'delete_logs',
        'read_reports',
        'write_reports',
        'delete_reports',
        'read_analytics',
        'write_analytics',
        'delete_analytics',
        'read_security',
        'write_security',
        'delete_security'
      ];
      
      // Tightly coupled to global state
      GlobalStateManager.permissions = permissions;
      
      // Tightly coupled to logging module
      LoggingModule.log('Loaded ' + permissions.length + ' permissions for user: ' + userId, 'permissions', () => {
        callback(null, permissions);
      });
    }, 75);
  },
  
  // Tightly coupled to global state
  hasPermission: function(permission, callback) {
    // Tightly coupled to global state
    const hasPerm = GlobalStateManager.permissions.includes(permission);
    
    // Tightly coupled to logging module
    LoggingModule.log('Permission check: ' + permission + ' = ' + hasPerm, 'permissions', () => {
      callback(null, hasPerm);
    });
  },
  
  // Tightly coupled to multiple modules
  checkPermissionAndExecute: function(permission, action, callback) {
    // Tightly coupled to permission module
    this.hasPermission(permission, (permError, hasPerm) => {
      if (permError) {
        callback(permError);
        return;
      }
      
      if (!hasPerm) {
        // Tightly coupled to notification module
        NotificationModule.addNotification('Insufficient permissions', 'error', () => {
          // Tightly coupled to logging module
          LoggingModule.log('Permission denied: ' + permission, 'permissions', () => {
            callback(new Error('Insufficient permissions'));
          });
        });
        return;
      }
      
      // Tightly coupled to logging module
      LoggingModule.log('Executing action with permission: ' + permission, 'permissions', () => {
        // Tightly coupled to action execution
        action((actionError, result) => {
          if (actionError) {
            callback(actionError);
            return;
          }
          
          callback(null, result);
        });
      });
    });
  }
};

// Cache module tightly coupled to global state
const CacheModule = {
  // Tightly coupled to global state
  set: function(key, value, callback) {
    setTimeout(() => {
      // Tightly coupled to global state
      GlobalStateManager.cache.set(key, value);
      
      // Tightly coupled to logging module
      LoggingModule.log('Cached item: ' + key, 'cache', () => {
        callback(null);
      });
    }, 10);
  },
  
  // Tightly coupled to global state
  get: function(key, callback) {
    setTimeout(() => {
      // Tightly coupled to global state
      const value = GlobalStateManager.cache.get(key);
      
      // Tightly coupled to logging module
      LoggingModule.log('Retrieved cached item: ' + key, 'cache', () => {
        callback(null, value);
      });
    }, 10);
  },
  
  // Tightly coupled to global state
  clear: function(callback) {
    setTimeout(() => {
      // Tightly coupled to global state
      GlobalStateManager.cache.clear();
      
      // Tightly coupled to logging module
      LoggingModule.log('Cache cleared', 'cache', () => {
        callback(null);
      });
    }, 25);
  }
};

// Notification module tightly coupled to global state
const NotificationModule = {
  // Tightly coupled to global state
  addNotification: function(message, type, callback) {
    setTimeout(() => {
      const notification = {
        id: Date.now(),
        message: message,
        type: type,
        timestamp: new Date()
      };
      
      // Tightly coupled to global state
      GlobalStateManager.notifications.push(notification);
      
      // Tightly coupled to UI module
      UIModule.showNotification(notification, () => {
        // Tightly coupled to logging module
        LoggingModule.log('Notification added: ' + message, 'notification', () => {
          callback(null);
        });
      });
    }, 5);
  },
  
  // Tightly coupled to global state
  removeNotification: function(notificationId, callback) {
    setTimeout(() => {
      // Tightly coupled to global state
      GlobalStateManager.notifications = GlobalStateManager.notifications.filter(
        n => n.id !== notificationId
      );
      
      // Tightly coupled to logging module
      LoggingModule.log('Notification removed: ' + notificationId, 'notification', () => {
        callback(null);
      });
    }, 5);
  }
};

// UI module tightly coupled to global state and DOM (simulated)
const UIModule = {
  // Tightly coupled to global state
  updateUserInterface: function(callback) {
    setTimeout(() => {
      // Simulate DOM manipulation tightly coupled to global state
      GlobalStateManager.uiState.currentView = 'dashboard';
      GlobalStateManager.uiState.loading = false;
      
      // Tightly coupled to logging module
      LoggingModule.log('UI updated for user: ' + GlobalStateManager.user?.username, 'ui', () => {
        callback(null);
      });
    }, 30);
  },
  
  // Tightly coupled to global state
  showLoginError: function(message, callback) {
    setTimeout(() => {
      // Simulate DOM manipulation tightly coupled to global state
      GlobalStateManager.uiState.loading = false;
      
      // Tightly coupled to logging module
      LoggingModule.log('Login error displayed: ' + message, 'ui', () => {
        callback(null);
      });
    }, 15);
  },
  
  // Tightly coupled to global state
  resetUserInterface: function(callback) {
    setTimeout(() => {
      // Simulate DOM manipulation tightly coupled to global state
      GlobalStateManager.uiState.currentView = 'login';
      GlobalStateManager.uiState.modalOpen = false;
      GlobalStateManager.uiState.loading = false;
      
      // Tightly coupled to logging module
      LoggingModule.log('UI reset to login view', 'ui', () => {
        callback(null);
      });
    }, 20);
  },
  
  // Tightly coupled to notification module and global state
  showNotification: function(notification, callback) {
    setTimeout(() => {
      // Simulate DOM manipulation for notification
      console.log('Showing notification:', notification.message);
      
      // Tightly coupled to logging module
      LoggingModule.log('Notification displayed: ' + notification.message, 'ui', () => {
        callback(null);
      });
    }, 5);
  }
};

// Logging module with tight coupling to all other modules
const LoggingModule = {
  // Deeply nested callback chain - EXTREME TIGHT COUPLING
  log: function(message, category, callback) {
    setTimeout(() => {
      const logEntry = {
        timestamp: new Date(),
        message: message,
        category: category,
        userId: GlobalStateManager.user?.id,
        sessionId: GlobalStateManager.session?.id,
        permissions: GlobalStateManager.permissions.length,
        cacheSize: GlobalStateManager.cache.size,
        notificationCount: GlobalStateManager.notifications.length,
        currentView: GlobalStateManager.uiState.currentView
      };
      
      // Simulate writing to log file with tight coupling to all modules
      console.log('LOG:', JSON.stringify(logEntry));
      
      // Tightly coupled to notification module for error notifications
      if (category === 'error') {
        NotificationModule.addNotification('Error occurred: ' + message, 'error', () => {
          // Tightly coupled to cache module for caching errors
          CacheModule.set('last_error', logEntry, () => {
            // Tightly coupled to UI module for error UI updates
            UIModule.showNotification({ message: 'Error logged', type: 'info' }, () => {
              callback(null);
            });
          });
        });
      } else {
        callback(null);
      }
    }, 1);
  },
  
  // Tightly coupled to all modules for comprehensive logging
  logSystemState: function(callback) {
    setTimeout(() => {
      const systemState = {
        user: GlobalStateManager.user,
        session: GlobalStateManager.session,
        permissions: GlobalStateManager.permissions,
        cacheSize: GlobalStateManager.cache.size,
        notificationCount: GlobalStateManager.notifications.length,
        uiState: GlobalStateManager.uiState
      };
      
      console.log('SYSTEM STATE LOG:', JSON.stringify(systemState, null, 2));
      callback(null);
    }, 50);
  }
};

// Circular dependency simulation - Module A depends on B, B depends on C, C depends on A
const ModuleA = {
  processData: function(data, callback) {
    // Tightly coupled to ModuleB
    ModuleB.validateData(data, (validateError, isValid) => {
      if (validateError) {
        callback(validateError);
        return;
      }
      
      if (isValid) {
        // Tightly coupled to ModuleC
        ModuleC.transformData(data, (transformError, transformedData) => {
          if (transformError) {
            callback(transformError);
            return;
          }
          
          callback(null, transformedData);
        });
      } else {
        callback(new Error('Invalid data'));
      }
    });
  }
};

const ModuleB = {
  validateData: function(data, callback) {
    // Tightly coupled to ModuleC
    ModuleC.checkDataFormat(data, (formatError, isCorrectFormat) => {
      if (formatError) {
        callback(formatError);
        return;
      }
      
      const isValid = isCorrectFormat && data.length > 0;
      callback(null, isValid);
    });
  }
};

const ModuleC = {
  transformData: function(data, callback) {
    // Tightly coupled to ModuleA
    ModuleA.processData(data.slice(0, Math.floor(data.length / 2)), (processError, partialResult) => {
      if (processError) {
        callback(processError);
        return;
      }
      
      const transformed = data.map(item => item.toUpperCase ? item.toUpperCase() : item);
      callback(null, transformed);
    });
  },
  
  checkDataFormat: function(data, callback) {
    // Tightly coupled to ModuleA
    ModuleA.processData([], (processError) => {
      if (processError) {
        callback(processError);
        return;
      }
      
      const isCorrectFormat = Array.isArray(data);
      callback(null, isCorrectFormat);
    });
  }
};

// Example usage demonstrating the tight coupling nightmare
function demonstrateTightCouplingHell() {
  console.log('Demonstrating tight coupling hell...');
  
  // This will create an infinite loop due to circular dependencies
  // AuthModule.login('testuser', 'password123', (error, result) => {
  //   if (error) {
  //     console.error('Login failed:', error);
  //   } else {
  //     console.log('Login successful:', result);
  //   }
  // });
  
  // Instead, let's show the structure by logging the modules
  console.log('Global State Manager:', GlobalStateManager);
  console.log('Auth Module:', AuthModule);
  console.log('Session Module:', SessionModule);
  console.log('Permission Module:', PermissionModule);
  console.log('Cache Module:', CacheModule);
  console.log('Notification Module:', NotificationModule);
  console.log('UI Module:', UIModule);
  console.log('Logging Module:', LoggingModule);
  console.log('Module A:', ModuleA);
  console.log('Module B:', ModuleB);
  console.log('Module C:', ModuleC);
}

// Export modules for external use (but they're still tightly coupled internally)
module.exports = {
  GlobalStateManager,
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
  demonstrateTightCouplingHell
};
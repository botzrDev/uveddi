// callback_pyramid.js - Extreme Long Methods with Callback Hell
// This file demonstrates extremely long methods with deep callback nesting

class CallbackPyramidNightmare {
  constructor() {
    this.data = [];
    this.config = {};
    this.state = {};
  }

  // EXTREMELY LONG METHOD 1: Deeply nested callback hell for data processing (300+ lines)
  processDataWithCallbackHell(inputData, callback) {
    console.log('Starting data processing with callback hell...');
    
    // Level 1: Validate input data
    if (!inputData || typeof inputData !== 'object') {
      callback(new Error('Invalid input data'), null);
      return;
    }
    
    // Level 2: Check data structure
    if (!Array.isArray(inputData.items)) {
      callback(new Error('Input data must contain items array'), null);
      return;
    }
    
    // Level 3: Validate minimum items count
    if (inputData.items.length < 1) {
      callback(new Error('Input data must contain at least one item'), null);
      return;
    }
    
    // Level 4: Process each item with nested callbacks
    let processedItems = [];
    let itemIndex = 0;
    
    function processNextItem() {
      // Level 5: Check if all items are processed
      if (itemIndex >= inputData.items.length) {
        // Level 6: Validate processed items
        if (processedItems.length === 0) {
          callback(new Error('No items were processed successfully'), null);
          return;
        }
        
        // Level 7: Calculate statistics
        let stats = {
          totalItems: processedItems.length,
          averageValue: 0,
          minValue: Number.MAX_VALUE,
          maxValue: Number.MIN_VALUE,
          sum: 0
        };
        
        // Level 8: Calculate sum and find min/max
        for (let i = 0; i < processedItems.length; i++) {
          let item = processedItems[i];
          
          // Level 9: Validate item value
          if (typeof item.value === 'number') {
            stats.sum += item.value;
            
            // Level 10: Update min value
            if (item.value < stats.minValue) {
              stats.minValue = item.value;
            }
            
            // Level 11: Update max value
            if (item.value > stats.maxValue) {
              stats.maxValue = item.value;
            }
          }
          
          // Level 12: Process nested properties
          if (item.properties && typeof item.properties === 'object') {
            let propKeys = Object.keys(item.properties);
            
            // Level 13: Process each property
            for (let j = 0; j < propKeys.length; j++) {
              let propKey = propKeys[j];
              let propValue = item.properties[propKey];
              
              // Level 14: Validate property value
              if (propValue !== null && propValue !== undefined) {
                // Level 15: Process string properties
                if (typeof propValue === 'string') {
                  // Level 16: Check string length
                  if (propValue.length > 100) {
                    // Level 17: Truncate long strings
                    item.properties[propKey] = propValue.substring(0, 100) + '...';
                  }
                }
                // Level 18: Process number properties
                else if (typeof propValue === 'number') {
                  // Level 19: Check number range
                  if (propValue < 0) {
                    // Level 20: Convert negative numbers to positive
                    item.properties[propKey] = Math.abs(propValue);
                  }
                }
                // Level 21: Process array properties
                else if (Array.isArray(propValue)) {
                  // Level 22: Limit array size
                  if (propValue.length > 50) {
                    // Level 23: Truncate large arrays
                    item.properties[propKey] = propValue.slice(0, 50);
                  }
                }
                // Level 24: Process object properties
                else if (typeof propValue === 'object') {
                  // Level 25: Limit object properties
                  let objKeys = Object.keys(propValue);
                  if (objKeys.length > 20) {
                    // Level 26: Remove excess properties
                    let newObj = {};
                    for (let k = 0; k < 20; k++) {
                      newObj[objKeys[k]] = propValue[objKeys[k]];
                    }
                    item.properties[propKey] = newObj;
                  }
                }
              }
            }
          }
          
          // Level 27: Process nested arrays
          if (item.nestedArrays && Array.isArray(item.nestedArrays)) {
            // Level 28: Process each nested array
            for (let k = 0; k < item.nestedArrays.length; k++) {
              let nestedArray = item.nestedArrays[k];
              
              // Level 29: Validate nested array
              if (Array.isArray(nestedArray)) {
                // Level 30: Limit nested array size
                if (nestedArray.length > 30) {
                  // Level 31: Truncate nested arrays
                  item.nestedArrays[k] = nestedArray.slice(0, 30);
                }
                
                // Level 32: Process nested array items
                for (let l = 0; l < nestedArray.length; l++) {
                  let nestedItem = nestedArray[l];
                  
                  // Level 33: Validate nested item
                  if (nestedItem && typeof nestedItem === 'object') {
                    // Level 34: Process nested item properties
                    let nestedKeys = Object.keys(nestedItem);
                    for (let m = 0; m < nestedKeys.length; m++) {
                      let nestedKey = nestedKeys[m];
                      let nestedValue = nestedItem[nestedKey];
                      
                      // Level 35: Sanitize nested values
                      if (typeof nestedValue === 'string') {
                        // Level 36: Remove special characters
                        nestedItem[nestedKey] = nestedValue.replace(/[<>]/g, '');
                      }
                    }
                  }
                }
              }
            }
          }
        }
        
        // Level 37: Calculate average
        if (processedItems.length > 0) {
          stats.averageValue = stats.sum / processedItems.length;
        }
        
        // Level 38: Validate statistics
        if (stats.minValue === Number.MAX_VALUE) {
          stats.minValue = 0;
        }
        
        if (stats.maxValue === Number.MIN_VALUE) {
          stats.maxValue = 0;
        }
        
        // Level 39: Create result object
        let result = {
          items: processedItems,
          statistics: stats,
          metadata: {
            processedAt: new Date().toISOString(),
            processorVersion: '1.0.0',
            inputItemCount: inputData.items.length,
            outputItemCount: processedItems.length
          }
        };
        
        // Level 40: Apply final transformations
        result.metadata.processingTime = Date.now() - startTime;
        result.metadata.efficiency = (result.statistics.totalItems / inputData.items.length) * 100;
        
        // Level 41: Validate result
        if (result.items.length > 0) {
          // Level 42: Sort items by value
          result.items.sort((a, b) => {
            // Level 43: Compare values
            if (a.value < b.value) return -1;
            if (a.value > b.value) return 1;
            return 0;
          });
          
          // Level 44: Apply final formatting
          result.formatted = true;
          result.version = '2.0.0';
          
          // Level 45: Add checksum
          let checksum = 0;
          for (let i = 0; i < result.items.length; i++) {
            checksum += result.items[i].id || 0;
          }
          result.checksum = checksum;
          
          // Level 46: Final validation
          if (result.checksum > 0) {
            callback(null, result);
          } else {
            callback(new Error('Result validation failed'), null);
          }
        } else {
          callback(new Error('No valid items in result'), null);
        }
      } else {
        // Level 47: Process current item
        let currentItem = inputData.items[itemIndex];
        
        // Level 48: Validate current item
        if (currentItem && typeof currentItem === 'object') {
          // Level 49: Process item ID
          if (typeof currentItem.id !== 'number') {
            currentItem.id = itemIndex + 1;
          }
          
          // Level 50: Process item value
          if (typeof currentItem.value !== 'number') {
            currentItem.value = 0;
          }
          
          // Level 51: Process item name
          if (typeof currentItem.name !== 'string') {
            currentItem.name = 'Item #' + currentItem.id;
          }
          
          // Level 52: Process item properties
          if (!currentItem.properties || typeof currentItem.properties !== 'object') {
            currentItem.properties = {};
          }
          
          // Level 53: Add default properties
          if (!currentItem.properties.created) {
            currentItem.properties.created = new Date().toISOString();
          }
          
          if (!currentItem.properties.modified) {
            currentItem.properties.modified = new Date().toISOString();
          }
          
          if (!currentItem.properties.version) {
            currentItem.properties.version = '1.0.0';
          }
          
          // Level 54: Process item tags
          if (!currentItem.tags || !Array.isArray(currentItem.tags)) {
            currentItem.tags = [];
          }
          
          // Level 55: Add default tags
          if (currentItem.tags.length === 0) {
            currentItem.tags.push('default');
          }
          
          // Level 56: Limit tags
          if (currentItem.tags.length > 10) {
            currentItem.tags = currentItem.tags.slice(0, 10);
          }
          
          // Level 57: Process item status
          if (!currentItem.status) {
            currentItem.status = 'pending';
          }
          
          // Level 58: Validate status
          const validStatuses = ['pending', 'processing', 'completed', 'failed', 'cancelled'];
          if (!validStatuses.includes(currentItem.status)) {
            currentItem.status = 'pending';
          }
          
          // Level 59: Process item priority
          if (typeof currentItem.priority !== 'number') {
            currentItem.priority = 0;
          }
          
          // Level 60: Validate priority range
          if (currentItem.priority < 0 || currentItem.priority > 10) {
            currentItem.priority = Math.max(0, Math.min(10, currentItem.priority));
          }
          
          // Level 61: Add to processed items
          processedItems.push(currentItem);
        }
        
        // Level 62: Move to next item
        itemIndex++;
        
        // Level 63: Continue processing with delay to simulate async work
        setTimeout(processNextItem, 1);
      }
    }
    
    // Record start time for processing time calculation
    const startTime = Date.now();
    
    // Start processing items
    processNextItem();
  }

  // EXTREMELY LONG METHOD 2: Complex business logic with multiple responsibilities (250+ lines)
  complexBusinessLogicProcessor(config, data, options, callback) {
    console.log('Starting complex business logic processing...');
    
    // Validate inputs
    if (!config || typeof config !== 'object') {
      callback(new Error('Configuration is required'), null);
      return;
    }
    
    if (!data || typeof data !== 'object') {
      callback(new Error('Data is required'), null);
      return;
    }
    
    if (!options || typeof options !== 'object') {
      callback(new Error('Options are required'), null);
      return;
    }
    
    // Extract configuration values
    const {
      enableValidation = true,
      enableTransformation = true,
      enableEnrichment = true,
      enableFiltering = true,
      enableSorting = true,
      enablePagination = true,
      enableCaching = true,
      enableLogging = true,
      validationRules = {},
      transformationRules = {},
      enrichmentRules = {},
      filteringRules = {},
      sortingRules = {},
      paginationSettings = {},
      cacheSettings = {},
      loggingSettings = {}
    } = config;
    
    // Extract data components
    const {
      users = [],
      products = [],
      orders = [],
      transactions = [],
      inventory = [],
      categories = [],
      suppliers = [],
      customers = []
    } = data;
    
    // Extract options
    const {
      userId = null,
      productId = null,
      orderId = null,
      fromDate = null,
      toDate = null,
      minAmount = 0,
      maxAmount = Number.MAX_VALUE,
      status = null,
      category = null,
      supplier = null,
      customer = null,
      sortBy = 'id',
      sortOrder = 'asc',
      page = 1,
      limit = 100,
      includeRelated = true,
      includeMetadata = true,
      includeStatistics = true
    } = options;
    
    // Initialize result object
    let result = {
      data: [],
      metadata: {},
      statistics: {}
    };
    
    // Initialize processing steps
    let validationPassed = true;
    let transformationApplied = false;
    let enrichmentCompleted = false;
    let filteringApplied = false;
    let sortingApplied = false;
    let paginationApplied = false;
    
    // Step 1: Validation
    if (enableValidation) {
      console.log('Applying validation rules...');
      
      // Validate users
      for (let i = 0; i < users.length; i++) {
        const user = users[i];
        
        // Validate user ID
        if (!user.id || typeof user.id !== 'number') {
          if (validationRules.userIdRequired !== false) {
            validationPassed = false;
            break;
          }
        }
        
        // Validate user email
        if (!user.email || typeof user.email !== 'string') {
          if (validationRules.emailRequired !== false) {
            validationPassed = false;
            break;
          }
        } else {
          // Validate email format
          const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
          if (!emailRegex.test(user.email)) {
            if (validationRules.emailFormatRequired !== false) {
              validationPassed = false;
              break;
            }
          }
        }
        
        // Validate user name
        if (!user.name || typeof user.name !== 'string') {
          if (validationRules.nameRequired !== false) {
            validationPassed = false;
            break;
          }
        }
        
        // Validate user status
        if (user.status && typeof user.status === 'string') {
          const validStatuses = ['active', 'inactive', 'suspended', 'deleted'];
          if (!validStatuses.includes(user.status)) {
            if (validationRules.statusValidation !== false) {
              validationPassed = false;
              break;
            }
          }
        }
      }
      
      // Validate products if validation still passed
      if (validationPassed) {
        for (let i = 0; i < products.length; i++) {
          const product = products[i];
          
          // Validate product ID
          if (!product.id || typeof product.id !== 'number') {
            if (validationRules.productIdRequired !== false) {
              validationPassed = false;
              break;
            }
          }
          
          // Validate product name
          if (!product.name || typeof product.name !== 'string') {
            if (validationRules.productNameRequired !== false) {
              validationPassed = false;
              break;
            }
          }
          
          // Validate product price
          if (product.price === undefined || typeof product.price !== 'number') {
            if (validationRules.priceRequired !== false) {
              validationPassed = false;
              break;
            }
        } else if (product.price < 0) {
            if (validationRules.pricePositive !== false) {
              validationPassed = false;
              break;
            }
          }
          
          // Validate product category
          if (product.category && typeof product.category === 'string') {
            if (categories.length > 0) {
              let categoryFound = false;
              for (let j = 0; j < categories.length; j++) {
                if (categories[j].id === product.category || categories[j].name === product.category) {
                  categoryFound = true;
                  break;
                }
              }
              if (!categoryFound && validationRules.categoryValidation !== false) {
                validationPassed = false;
                break;
              }
            }
          }
        }
      }
      
      // Validate orders if validation still passed
      if (validationPassed) {
        for (let i = 0; i < orders.length; i++) {
          const order = orders[i];
          
          // Validate order ID
          if (!order.id || typeof order.id !== 'number') {
            if (validationRules.orderIdRequired !== false) {
              validationPassed = false;
              break;
            }
          }
          
          // Validate order user ID
          if (!order.userId || typeof order.userId !== 'number') {
            if (validationRules.orderUserIdRequired !== false) {
              validationPassed = false;
              break;
            }
          }
          
          // Validate order items
          if (!order.items || !Array.isArray(order.items)) {
            if (validationRules.orderItemsRequired !== false) {
              validationPassed = false;
              break;
            }
          } else if (order.items.length === 0) {
            if (validationRules.orderItemsNotEmpty !== false) {
              validationPassed = false;
              break;
            }
          } else {
            // Validate each order item
            for (let j = 0; j < order.items.length; j++) {
              const item = order.items[j];
              
              // Validate item product ID
              if (!item.productId || typeof item.productId !== 'number') {
                if (validationRules.orderItemProductIdRequired !== false) {
                  validationPassed = false;
                  break;
                }
              }
              
              // Validate item quantity
              if (!item.quantity || typeof item.quantity !== 'number') {
                if (validationRules.orderItemQuantityRequired !== false) {
                  validationPassed = false;
                  break;
                }
              } else if (item.quantity <= 0) {
                if (validationRules.orderItemQuantityPositive !== false) {
                  validationPassed = false;
                  break;
                }
              }
              
              // Validate item price
              if (item.price === undefined || typeof item.price !== 'number') {
                if (validationRules.orderItemPriceRequired !== false) {
                  validationPassed = false;
                  break;
                }
              } else if (item.price < 0) {
                if (validationRules.orderItemPricePositive !== false) {
                  validationPassed = false;
                  break;
                }
              }
            }
            
            // Break outer loop if validation failed in items
            if (!validationPassed) {
              break;
            }
          }
          
          // Validate order total
          if (order.total === undefined || typeof order.total !== 'number') {
            if (validationRules.orderTotalRequired !== false) {
              validationPassed = false;
              break;
            }
          } else if (order.total < 0) {
            if (validationRules.orderTotalPositive !== false) {
              validationPassed = false;
              break;
            }
          }
          
          // Validate order status
          if (order.status && typeof order.status === 'string') {
            const validStatuses = ['pending', 'processing', 'shipped', 'delivered', 'cancelled', 'refunded'];
            if (!validStatuses.includes(order.status)) {
              if (validationRules.orderStatusValidation !== false) {
                validationPassed = false;
                break;
              }
            }
          }
        }
      }
    }
    
    // Check validation result
    if (!validationPassed) {
      callback(new Error('Data validation failed'), null);
      return;
    }
    
    // Step 2: Transformation
    if (enableTransformation) {
      console.log('Applying transformation rules...');
      transformationApplied = true;
      
      // Transform users
      for (let i = 0; i < users.length; i++) {
        const user = users[i];
        
        // Normalize email
        if (user.email && typeof user.email === 'string') {
          user.email = user.email.toLowerCase().trim();
        }
        
        // Format name
        if (user.name && typeof user.name === 'string') {
          user.name = user.name.trim().replace(/\s+/g, ' ');
        }
        
        // Add full name if first and last name exist
        if (user.firstName && user.lastName) {
          user.fullName = user.firstName + ' ' + user.lastName;
        }
        
        // Convert dates
        if (user.createdAt && typeof user.createdAt === 'string') {
          user.createdAt = new Date(user.createdAt);
        }
        
        if (user.updatedAt && typeof user.updatedAt === 'string') {
          user.updatedAt = new Date(user.updatedAt);
        }
        
        // Calculate account age
        if (user.createdAt instanceof Date) {
          user.accountAgeDays = Math.floor((Date.now() - user.createdAt.getTime()) / (1000 * 60 * 60 * 24));
        }
      }
      
      // Transform products
      for (let i = 0; i < products.length; i++) {
        const product = products[i];
        
        // Format name
        if (product.name && typeof product.name === 'string') {
          product.name = product.name.trim();
        }
        
        // Format description
        if (product.description && typeof product.description === 'string') {
          product.description = product.description.trim();
        }
        
        // Convert price to fixed decimal
        if (typeof product.price === 'number') {
          product.price = parseFloat(product.price.toFixed(2));
        }
        
        // Convert dates
        if (product.createdAt && typeof product.createdAt === 'string') {
          product.createdAt = new Date(product.createdAt);
        }
        
        if (product.updatedAt && typeof product.updatedAt === 'string') {
          product.updatedAt = new Date(product.updatedAt);
        }
        
        // Add price range category
        if (typeof product.price === 'number') {
          if (product.price < 10) {
            product.priceRange = 'budget';
          } else if (product.price < 50) {
            product.priceRange = 'mid-range';
          } else if (product.price < 100) {
            product.priceRange = 'premium';
          } else {
            product.priceRange = 'luxury';
          }
        }
      }
      
      // Transform orders
      for (let i = 0; i < orders.length; i++) {
        const order = orders[i];
        
        // Convert dates
        if (order.createdAt && typeof order.createdAt === 'string') {
          order.createdAt = new Date(order.createdAt);
        }
        
        if (order.updatedAt && typeof order.updatedAt === 'string') {
          order.updatedAt = new Date(order.updatedAt);
        }
        
        if (order.shippedAt && typeof order.shippedAt === 'string') {
          order.shippedAt = new Date(order.shippedAt);
        }
        
        if (order.deliveredAt && typeof order.deliveredAt === 'string') {
          order.deliveredAt = new Date(order.deliveredAt);
        }
        
        // Calculate order age
        if (order.createdAt instanceof Date) {
          order.orderAgeDays = Math.floor((Date.now() - order.createdAt.getTime()) / (1000 * 60 * 60 * 24));
        }
        
        // Calculate processing time
        if (order.createdAt instanceof Date && order.shippedAt instanceof Date) {
          order.processingTimeHours = Math.floor((order.shippedAt.getTime() - order.createdAt.getTime()) / (1000 * 60 * 60));
        }
        
        // Calculate delivery time
        if (order.shippedAt instanceof Date && order.deliveredAt instanceof Date) {
          order.deliveryTimeDays = Math.floor((order.deliveredAt.getTime() - order.shippedAt.getTime()) / (1000 * 60 * 60 * 24));
        }
        
        // Calculate order total if not present
        if (order.total === undefined && order.items && Array.isArray(order.items)) {
          let calculatedTotal = 0;
          for (let j = 0; j < order.items.length; j++) {
            const item = order.items[j];
            if (typeof item.price === 'number' && typeof item.quantity === 'number') {
              calculatedTotal += item.price * item.quantity;
            }
          }
          order.total = parseFloat(calculatedTotal.toFixed(2));
        }
      }
    }
    
    // Step 3: Data enrichment (continued on next message due to length limit)
    
    // CONTINUATION OF EXTREMELY LONG METHOD 2
    // Step 3: Data enrichment
    if (enableEnrichment) {
      console.log('Applying data enrichment...');
      enrichmentCompleted = true;
      
      // Enrich users with order information
      for (let i = 0; i < users.length; i++) {
        const user = users[i];
        user.orders = [];
        user.totalSpent = 0;
        user.orderCount = 0;
        user.lastOrderDate = null;
        user.favoriteCategory = null;
        user.averageOrderValue = 0;
        
        // Find user's orders
        for (let j = 0; j < orders.length; j++) {
          const order = orders[j];
          if (order.userId === user.id) {
            user.orders.push(order);
            user.orderCount++;
            
            if (typeof order.total === 'number') {
              user.totalSpent += order.total;
            }
            
            if (order.createdAt instanceof Date) {
              if (!user.lastOrderDate || order.createdAt > user.lastOrderDate) {
                user.lastOrderDate = order.createdAt;
              }
            }
          }
        }
        
        // Calculate average order value
        if (user.orderCount > 0) {
          user.averageOrderValue = parseFloat((user.totalSpent / user.orderCount).toFixed(2));
        }
        
        // Find favorite category
        if (user.orders.length > 0) {
          let categoryCounts = {};
          
          // Count categories from order items
          for (let j = 0; j < user.orders.length; j++) {
            const order = user.orders[j];
            if (order.items && Array.isArray(order.items)) {
              for (let k = 0; k < order.items.length; k++) {
                const item = order.items[k];
                if (item.productId) {
                  // Find product to get category
                  for (let l = 0; l < products.length; l++) {
                    const product = products[l];
                    if (product.id === item.productId && product.category) {
                      if (!categoryCounts[product.category]) {
                        categoryCounts[product.category] = 0;
                      }
                      categoryCounts[product.category]++;
                      break;
                    }
                  }
                }
              }
            }
          }
          
          // Find category with highest count
          let maxCount = 0;
          for (let category in categoryCounts) {
            if (categoryCounts[category] > maxCount) {
              maxCount = categoryCounts[category];
              user.favoriteCategory = category;
            }
          }
        }
      }
      
      // Enrich products with sales information
      for (let i = 0; i < products.length; i++) {
        const product = products[i];
        product.salesCount = 0;
        product.totalRevenue = 0;
        product.averageRating = 0;
        product.reviewCount = 0;
        product.lastSoldDate = null;
        product.topCustomer = null;
        
        // Find product sales from order items
        for (let j = 0; j < orders.length; j++) {
          const order = orders[j];
          if (order.items && Array.isArray(order.items)) {
            for (let k = 0; k < order.items.length; k++) {
              const item = order.items[k];
              if (item.productId === product.id) {
                product.salesCount += item.quantity || 0;
                
                if (typeof item.price === 'number' && typeof item.quantity === 'number') {
                  product.totalRevenue += item.price * item.quantity;
                }
                
                if (order.createdAt instanceof Date) {
                  if (!product.lastSoldDate || order.createdAt > product.lastSoldDate) {
                    product.lastSoldDate = order.createdAt;
                  }
                }
              }
            }
          }
        }
        
        // Format revenue
        product.totalRevenue = parseFloat(product.totalRevenue.toFixed(2));
      }
      
      // Enrich orders with product and user details
      for (let i = 0; i < orders.length; i++) {
        const order = orders[i];
        
        // Add user details
        for (let j = 0; j < users.length; j++) {
          const user = users[j];
          if (user.id === order.userId) {
            order.userDetails = {
              id: user.id,
              name: user.name || user.fullName,
              email: user.email,
              status: user.status
            };
            break;
          }
        }
        
        // Add product details to items
        if (order.items && Array.isArray(order.items)) {
          for (let j = 0; j < order.items.length; j++) {
            const item = order.items[j];
            if (item.productId) {
              for (let k = 0; k < products.length; k++) {
                const product = products[k];
                if (product.id === item.productId) {
                  item.productDetails = {
                    id: product.id,
                    name: product.name,
                    price: product.price,
                    category: product.category
                  };
                  break;
                }
              }
            }
          }
        }
      }
    }
    
    // Step 4: Data filtering (continued on next message due to length limit)
    
    // CONTINUATION OF EXTREMELY LONG METHOD 2
    // Step 4: Data filtering
    if (enableFiltering) {
      console.log('Applying data filtering...');
      filteringApplied = true;
      
      // Filter users
      if (userId) {
        users = users.filter(user => user.id === userId);
      }
      
      if (status) {
        users = users.filter(user => user.status === status);
      }
      
      // Filter products
      if (productId) {
        products = products.filter(product => product.id === productId);
      }
      
      if (category) {
        products = products.filter(product => product.category === category);
      }
      
      if (supplier) {
        products = products.filter(product => product.supplierId === supplier);
      }
      
      // Filter orders
      if (orderId) {
        orders = orders.filter(order => order.id === orderId);
      }
      
      if (fromDate) {
        const fromDateObj = new Date(fromDate);
        orders = orders.filter(order => order.createdAt >= fromDateObj);
      }
      
      if (toDate) {
        const toDateObj = new Date(toDate);
        orders = orders.filter(order => order.createdAt <= toDateObj);
      }
      
      if (minAmount > 0) {
        orders = orders.filter(order => order.total >= minAmount);
      }
      
      if (maxAmount < Number.MAX_VALUE) {
        orders = orders.filter(order => order.total <= maxAmount);
      }
      
      if (status) {
        orders = orders.filter(order => order.status === status);
      }
      
      if (customer) {
        orders = orders.filter(order => order.userId === customer);
      }
    }
    
    // Step 5: Data sorting
    if (enableSorting) {
      console.log('Applying data sorting...');
      sortingApplied = true;
      
      // Sort users
      if (users.length > 0) {
        users.sort((a, b) => {
          let aValue, bValue;
          
          switch (sortBy) {
            case 'id':
              aValue = a.id;
              bValue = b.id;
              break;
            case 'name':
              aValue = (a.name || a.fullName || '').toLowerCase();
              bValue = (b.name || b.fullName || '').toLowerCase();
              break;
            case 'email':
              aValue = (a.email || '').toLowerCase();
              bValue = (b.email || '').toLowerCase();
              break;
            case 'createdAt':
              aValue = a.createdAt instanceof Date ? a.createdAt.getTime() : 0;
              bValue = b.createdAt instanceof Date ? b.createdAt.getTime() : 0;
              break;
            case 'totalSpent':
              aValue = a.totalSpent || 0;
              bValue = b.totalSpent || 0;
              break;
            case 'orderCount':
              aValue = a.orderCount || 0;
              bValue = b.orderCount || 0;
              break;
            default:
              aValue = a.id;
              bValue = b.id;
          }
          
          if (sortOrder === 'desc') {
            return aValue > bValue ? -1 : aValue < bValue ? 1 : 0;
          } else {
            return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
          }
        });
      }
      
      // Sort products
      if (products.length > 0) {
        products.sort((a, b) => {
          let aValue, bValue;
          
          switch (sortBy) {
            case 'id':
              aValue = a.id;
              bValue = b.id;
              break;
            case 'name':
              aValue = (a.name || '').toLowerCase();
              bValue = (b.name || '').toLowerCase();
              break;
            case 'price':
              aValue = a.price || 0;
              bValue = b.price || 0;
              break;
            case 'createdAt':
              aValue = a.createdAt instanceof Date ? a.createdAt.getTime() : 0;
              bValue = b.createdAt instanceof Date ? b.createdAt.getTime() : 0;
              break;
            case 'salesCount':
              aValue = a.salesCount || 0;
              bValue = b.salesCount || 0;
              break;
            case 'totalRevenue':
              aValue = a.totalRevenue || 0;
              bValue = b.totalRevenue || 0;
              break;
            default:
              aValue = a.id;
              bValue = b.id;
          }
          
          if (sortOrder === 'desc') {
            return aValue > bValue ? -1 : aValue < bValue ? 1 : 0;
          } else {
            return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
          }
        });
      }
      
      // Sort orders
      if (orders.length > 0) {
        orders.sort((a, b) => {
          let aValue, bValue;
          
          switch (sortBy) {
            case 'id':
              aValue = a.id;
              bValue = b.id;
              break;
            case 'userId':
              aValue = a.userId;
              bValue = b.userId;
              break;
            case 'total':
              aValue = a.total || 0;
              bValue = b.total || 0;
              break;
            case 'createdAt':
              aValue = a.createdAt instanceof Date ? a.createdAt.getTime() : 0;
              bValue = b.createdAt instanceof Date ? b.createdAt.getTime() : 0;
              break;
            case 'orderAgeDays':
              aValue = a.orderAgeDays || 0;
              bValue = b.orderAgeDays || 0;
              break;
            default:
              aValue = a.id;
              bValue = b.id;
          }
          
          if (sortOrder === 'desc') {
            return aValue > bValue ? -1 : aValue < bValue ? 1 : 0;
          } else {
            return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
          }
        });
      }
    }
    
    // Step 6: Pagination
    if (enablePagination) {
      console.log('Applying pagination...');
      paginationApplied = true;
      
      // Paginate users
      if (users.length > limit) {
        const startIndex = (page - 1) * limit;
        const endIndex = startIndex + limit;
        users = users.slice(startIndex, endIndex);
      }
      
      // Paginate products
      if (products.length > limit) {
        const startIndex = (page - 1) * limit;
        const endIndex = startIndex + limit;
        products = products.slice(startIndex, endIndex);
      }
      
      // Paginate orders
      if (orders.length > limit) {
        const startIndex = (page - 1) * limit;
        const endIndex = startIndex + limit;
        orders = orders.slice(startIndex, endIndex);
      }
    }
    
    // Step 7: Prepare result
    result.data = {
      users: users,
      products: products,
      orders: orders
    };
    
    // Step 8: Add metadata
    if (includeMetadata) {
      result.metadata = {
        processedAt: new Date().toISOString(),
        processingSteps: {
          validation: validationPassed,
          transformation: transformationApplied,
          enrichment: enrichmentCompleted,
          filtering: filteringApplied,
          sorting: sortingApplied,
          pagination: paginationApplied
        },
        filtersApplied: {
          userId: userId || null,
          productId: productId || null,
          orderId: orderId || null,
          fromDate: fromDate || null,
          toDate: toDate || null,
          minAmount: minAmount > 0 ? minAmount : null,
          maxAmount: maxAmount < Number.MAX_VALUE ? maxAmount : null,
          status: status || null,
          category: category || null,
          supplier: supplier || null,
          customer: customer || null
        },
        sortingApplied: {
          sortBy: sortBy,
          sortOrder: sortOrder
        },
        paginationApplied: {
          page: page,
          limit: limit,
          totalUsers: users.length,
          totalProducts: products.length,
          totalOrders: orders.length
        }
      };
    }
    
    // Step 9: Add statistics
    if (includeStatistics) {
      result.statistics = {
        totals: {
          users: users.length,
          products: products.length,
          orders: orders.length
        },
        userStats: {
          totalSpent: users.reduce((sum, user) => sum + (user.totalSpent || 0), 0),
          averageSpent: users.length > 0 ? parseFloat((users.reduce((sum, user) => sum + (user.totalSpent || 0), 0) / users.length).toFixed(2)) : 0,
          mostActiveUser: users.length > 0 ? users.reduce((max, user) => (user.orderCount || 0) > (max.orderCount || 0) ? user : max, users[0]) : null
        },
        productStats: {
          totalRevenue: products.reduce((sum, product) => sum + (product.totalRevenue || 0), 0),
          bestSellingProduct: products.length > 0 ? products.reduce((max, product) => (product.salesCount || 0) > (max.salesCount || 0) ? product : max, products[0]) : null,
          averagePrice: products.length > 0 ? parseFloat((products.reduce((sum, product) => sum + (product.price || 0), 0) / products.length).toFixed(2)) : 0
        },
        orderStats: {
          totalRevenue: orders.reduce((sum, order) => sum + (order.total || 0), 0),
          averageOrderValue: orders.length > 0 ? parseFloat((orders.reduce((sum, order) => sum + (order.total || 0), 0) / orders.length).toFixed(2)) : 0,
          largestOrder: orders.length > 0 ? orders.reduce((max, order) => (order.total || 0) > (max.total || 0) ? order : max, orders[0]) : null
        }
      };
    }
    
    // Step 10: Return result
    callback(null, result);
  }

  // EXTREMELY LONG METHOD 3: Event handler complexity (200+ lines)
  handleComplexEventScenario(eventData, callback) {
    console.log('Handling complex event scenario...');
    
    // Validate event data
    if (!eventData || typeof eventData !== 'object') {
      callback(new Error('Invalid event data'), null);
      return;
    }
    
    // Extract event properties
    const {
      eventType = 'unknown',
      timestamp = Date.now(),
      userId = null,
      sessionId = null,
      ipAddress = null,
      userAgent = null,
      payload = {},
      metadata = {},
      context = {}
    } = eventData;
    
    // Initialize processing result
    let result = {
      eventId: 'evt_' + timestamp + '_' + Math.random().toString(36).substr(2, 9),
      processedAt: new Date().toISOString(),
      eventType: eventType,
      status: 'pending',
      actionsTaken: [],
      errors: [],
      warnings: [],
      metadata: {
        processingTime: 0,
        stepsCompleted: 0
      }
    };
    
    // Record start time
    const startTime = Date.now();
    
    // Step 1: Event type validation
    const validEventTypes = [
      'user_login', 'user_logout', 'user_register', 'user_update',
      'order_create', 'order_update', 'order_cancel', 'order_complete',
      'payment_success', 'payment_failure', 'payment_refund',
      'product_view', 'product_add_to_cart', 'product_purchase',
      'cart_add', 'cart_remove', 'cart_update',
      'search_query', 'search_result_click',
      'page_view', 'page_exit', 'click', 'hover',
      'error_occurred', 'system_alert', 'security_violation',
      'api_call', 'api_response', 'api_error',
      'file_upload', 'file_download', 'file_delete',
      'email_sent', 'email_opened', 'email_clicked',
      'notification_sent', 'notification_read',
      'feature_used', 'feature_enabled', 'feature_disabled',
      'integration_connected', 'integration_disconnected',
      'backup_created', 'backup_restored',
      'report_generated', 'report_downloaded',
      'audit_log', 'system_log', 'security_log'
    ];
    
    if (!validEventTypes.includes(eventType)) {
      result.warnings.push('Unknown event type: ' + eventType);
    }
    
    // Step 2: User validation
    if (userId) {
      // Simulate user lookup
      setTimeout(() => {
        // Simulate database call
        const userExists = Math.random() > 0.1; // 90% chance user exists
        
        if (!userExists) {
          result.errors.push('User not found: ' + userId);
          result.status = 'failed';
          result.metadata.processingTime = Date.now() - startTime;
          callback(new Error('User not found'), result);
          return;
        }
        
        // Step 3: Session validation
        if (sessionId) {
          // Simulate session lookup
          setTimeout(() => {
            // Simulate database call
            const sessionValid = Math.random() > 0.05; // 95% chance session is valid
            
            if (!sessionValid) {
              result.errors.push('Invalid session: ' + sessionId);
              result.status = 'failed';
              result.metadata.processingTime = Date.now() - startTime;
              callback(new Error('Invalid session'), result);
              return;
            }
            
            // Step 4: IP address validation
            if (ipAddress) {
              // Simulate IP validation
              setTimeout(() => {
                // Simulate security check
                const ipBlocked = Math.random() > 0.98; // 2% chance IP is blocked
                
                if (ipBlocked) {
                  result.errors.push('IP address blocked: ' + ipAddress);
                  result.status = 'failed';
                  result.metadata.processingTime = Date.now() - startTime;
                  callback(new Error('IP address blocked'), result);
                  return;
                }
                
                // Step 5: Rate limiting check
                // Simulate rate limiting
                setTimeout(() => {
                  // Simulate rate limit check
                  const rateLimitExceeded = Math.random() > 0.95; // 5% chance rate limit exceeded
                  
                  if (rateLimitExceeded) {
                    result.errors.push('Rate limit exceeded for user: ' + userId);
                    result.status = 'failed';
                    result.metadata.processingTime = Date.now() - startTime;
                    callback(new Error('Rate limit exceeded'), result);
                    return;
                  }
                  
                  // Step 6: Payload validation
                  if (payload && typeof payload === 'object') {
                    const payloadKeys = Object.keys(payload);
                    if (payloadKeys.length > 100) {
                      result.warnings.push('Large payload size: ' + payloadKeys.length + ' keys');
                    }
                    
                    // Validate specific payload fields based on event type
                    switch (eventType) {
                      case 'user_login':
                        if (!payload.username) {
                          result.errors.push('Missing username in login payload');
                        }
                        if (!payload.password) {
                          result.errors.push('Missing password in login payload');
                        }
                        break;
                        
                      case 'order_create':
                        if (!payload.items || !Array.isArray(payload.items)) {
                          result.errors.push('Missing or invalid items in order payload');
                        }
                        if (!payload.total || typeof payload.total !== 'number') {
                          result.errors.push('Missing or invalid total in order payload');
                        }
                        break;
                        
                      case 'payment_success':
                        if (!payload.amount || typeof payload.amount !== 'number') {
                          result.errors.push('Missing or invalid amount in payment payload');
                        }
                        if (!payload.currency || typeof payload.currency !== 'string') {
                          result.errors.push('Missing or invalid currency in payment payload');
                        }
                        break;
                    }
                  }
                  
                  // Step 7: Context validation
                  if (context && typeof context === 'object') {
                    const contextKeys = Object.keys(context);
                    if (contextKeys.length > 50) {
                      result.warnings.push('Large context size: ' + contextKeys.length + ' keys');
                    }
                  }
                  
                  // Step 8: Business logic processing
                  // This would normally contain complex business rules
                  setTimeout(() => {
                    // Simulate business logic processing
                    result.actionsTaken.push('Event validated');
                    result.metadata.stepsCompleted++;
                    
                    // Step 9: Data enrichment
                    // Add additional context to the event
                    setTimeout(() => {
                      result.enrichedData = {
                        userSegment: 'premium', // Simulated user segment
                        geoLocation: { // Simulated geo-location
                          country: 'US',
                          region: 'CA',
                          city: 'San Francisco'
                        },
                        deviceInfo: { // Simulated device info
                          type: userAgent.includes('Mobile') ? 'mobile' : 'desktop',
                          os: userAgent.includes('Windows') ? 'Windows' : 
                              userAgent.includes('Mac') ? 'Mac' : 
                              userAgent.includes('Linux') ? 'Linux' : 'Other',
                          browser: userAgent.includes('Chrome') ? 'Chrome' : 
                                  userAgent.includes('Firefox') ? 'Firefox' : 
                                  userAgent.includes('Safari') ? 'Safari' : 'Other'
                        },
                        timeInfo: { // Time-based information
                          hourOfDay: new Date(timestamp).getHours(),
                          dayOfWeek: new Date(timestamp).getDay(),
                          isWeekend: [0, 6].includes(new Date(timestamp).getDay()),
                          isBusinessHours: new Date(timestamp).getHours() >= 9 && new Date(timestamp).getHours() <= 17
                        }
                      };
                      
                      result.actionsTaken.push('Data enriched');
                      result.metadata.stepsCompleted++;
                      
                      // Step 10: Notification processing
                      // Determine if notifications need to be sent
                      setTimeout(() => {
                        const notificationTypes = [];
                        
                        // Determine notification types based on event type
                        switch (eventType) {
                          case 'user_register':
                            notificationTypes.push('welcome_email');
                            notificationTypes.push('admin_alert');
                            break;
                          case 'order_create':
                            notificationTypes.push('order_confirmation');
                            notificationTypes.push('inventory_update');
                            break;
                          case 'payment_success':
                            notificationTypes.push('payment_receipt');
                            notificationTypes.push('loyalty_points');
                            break;
                          case 'security_violation':
                            notificationTypes.push('security_alert');
                            notificationTypes.push('admin_alert');
                            break;
                        }
                        
                        if (notificationTypes.length > 0) {
                          result.actionsTaken.push('Scheduled notifications: ' + notificationTypes.join(', '));
                        }
                        
                        result.metadata.stepsCompleted++;
                        
                        // Step 11: Analytics processing
                        // Update analytics based on the event
                        setTimeout(() => {
                          result.analyticsUpdates = {
                            eventType: eventType,
                            userId: userId,
                            timestamp: timestamp,
                            metrics: {
                              eventCount: 1,
                              uniqueUsers: userId ? 1 : 0,
                              uniqueSessions: sessionId ? 1 : 0,
                              eventsByType: { [eventType]: 1 },
                              eventsByHour: { [new Date(timestamp).getHours()]: 1 }
                            }
                          };
                          
                          result.actionsTaken.push('Analytics updated');
                          result.metadata.stepsCompleted++;
                          
                          // Step 12: Audit logging
                          // Log the event for audit purposes
                          setTimeout(() => {
                            result.auditLog = {
                              eventId: result.eventId,
                              eventType: eventType,
                              userId: userId,
                              sessionId: sessionId,
                              timestamp: timestamp,
                              ipAddress: ipAddress,
                              success: result.errors.length === 0,
                              actions: result.actionsTaken
                            };
                            
                            result.actionsTaken.push('Audit log recorded');
                            result.metadata.stepsCompleted++;
                            
                            // Step 13: Final processing
                            // Complete the event processing
                            result.status = result.errors.length > 0 ? 'failed' : 'completed';
                            result.metadata.processingTime = Date.now() - startTime;
                            result.metadata.stepsCompleted++;
                            
                            // Step 14: Callback execution
                            // Execute the callback with the result
                            callback(null, result);
                          }, 1); // Simulate async delay
                        }, 1); // Simulate async delay
                      }, 1); // Simulate async delay
                    }, 1); // Simulate async delay
                  }, 1); // Simulate async delay
                }, 1); // Simulate async delay
              }, 1); // Simulate async delay
            } else {
              result.errors.push('Missing IP address');
              result.status = 'failed';
              result.metadata.processingTime = Date.now() - startTime;
              callback(new Error('Missing IP address'), result);
            }
          }, 1); // Simulate async delay
        } else {
          result.errors.push('Missing session ID');
          result.status = 'failed';
          result.metadata.processingTime = Date.now() - startTime;
          callback(new Error('Missing session ID'), result);
        }
      }, 1); // Simulate async delay
    } else {
      result.errors.push('Missing user ID');
      result.status = 'failed';
      result.metadata.processingTime = Date.now() - startTime;
      callback(new Error('Missing user ID'), result);
    }
  }
}

// Export the class
module.exports = CallbackPyramidNightmare;
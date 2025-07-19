// UV-60: Enhanced automated regression check for benchmark results
const fs = require('fs');
const path = require('path');

const reportPath = path.join(__dirname, '../benchmark-report.json');
const baselinePath = path.join(__dirname, '../benchmark-baseline.json');
const configPath = path.join(__dirname, '../performance-config.json');

// Default configuration
const defaultConfig = {
  thresholds: {
    overall_success_rate: 0.05,      // 5% degradation
    uv78_target_compliance: 0.10,    // 10% degradation
    performance_grade: 0.0,          // No grade degradation allowed
    cache_speedup: 0.15,             // 15% cache performance degradation
    mean_response_time: 0.20,        // 20% slower response time
    p95_response_time: 0.25          // 25% slower P95 response time
  },
  alerts: {
    slack_webhook: process.env.SLACK_WEBHOOK_URL,
    email_recipients: process.env.ALERT_EMAIL_RECIPIENTS?.split(',') || [],
    github_issue: process.env.CREATE_GITHUB_ISSUE === 'true'
  },
  historical_tracking: {
    enabled: true,
    max_history_entries: 100,
    trend_analysis_window: 10
  }
};

function loadJSON(file) {
  if (!fs.existsSync(file)) return null;
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch (error) {
    console.warn(`Warning: Could not parse ${file}: ${error.message}`);
    return null;
  }
}

function saveJSON(file, data) {
  try {
    fs.writeFileSync(file, JSON.stringify(data, null, 2));
    return true;
  } catch (error) {
    console.error(`Error saving ${file}: ${error.message}`);
    return false;
  }
}

function loadConfig() {
  const userConfig = loadJSON(configPath);
  return userConfig ? { ...defaultConfig, ...userConfig } : defaultConfig;
}

function calculateTrend(values, window = 5) {
  if (values.length < window) return 'insufficient_data';
  
  const recent = values.slice(-window);
  const older = values.slice(-window * 2, -window);
  
  if (older.length === 0) return 'insufficient_data';
  
  const recentAvg = recent.reduce((a, b) => a + b, 0) / recent.length;
  const olderAvg = older.reduce((a, b) => a + b, 0) / older.length;
  
  const change = (recentAvg - olderAvg) / olderAvg;
  
  if (Math.abs(change) < 0.02) return 'stable';
  return change > 0 ? 'improving' : 'degrading';
}

function updateHistoricalData(report, config) {
  const historyPath = path.join(__dirname, '../performance-history.json');
  let history = loadJSON(historyPath) || { entries: [] };
  
  const entry = {
    timestamp: new Date().toISOString(),
    metrics: report.summary,
    cache_performance: report.cache_performance,
    git_commit: process.env.GITHUB_SHA || 'unknown',
    git_branch: process.env.GITHUB_REF_NAME || 'unknown'
  };
  
  history.entries.push(entry);
  
  // Limit history size
  if (history.entries.length > config.historical_tracking.max_history_entries) {
    history.entries = history.entries.slice(-config.historical_tracking.max_history_entries);
  }
  
  saveJSON(historyPath, history);
  return history;
}

function analyzeTrends(history, config) {
  if (!history || history.entries.length < 2) return {};
  
  const metrics = ['overall_success_rate', 'uv78_target_compliance'];
  const trends = {};
  
  for (const metric of metrics) {
    const values = history.entries.map(e => e.metrics[metric]).filter(v => v !== undefined);
    trends[metric] = calculateTrend(values, config.historical_tracking.trend_analysis_window);
  }
  
  return trends;
}

function compareMetric(metric, current, baseline, threshold, metricConfig = {}) {
  if (baseline === undefined || current === undefined) return { pass: true, reason: 'no_data' };
  
  const delta = (baseline - current) / baseline;
  const isRegression = delta > threshold;
  
  return {
    pass: !isRegression,
    current,
    baseline,
    delta: delta * 100,
    threshold: threshold * 100,
    severity: isRegression ? (delta > threshold * 2 ? 'high' : 'medium') : 'none',
    reason: isRegression ? 'regression' : 'pass'
  };
}

function generateAlertMessage(regressions, trends) {
  let message = '🚨 Performance Regression Alert\\n\\n';
  
  message += `**Commit:** ${process.env.GITHUB_SHA?.substring(0, 8) || 'unknown'}\\n`;
  message += `**Branch:** ${process.env.GITHUB_REF_NAME || 'unknown'}\\n`;
  message += `**Time:** ${new Date().toISOString()}\\n\\n`;
  
  message += '**Regressions Detected:**\\n';
  for (const [metric, result] of Object.entries(regressions)) {
    if (!result.pass) {
      message += `• ${metric}: ${result.delta.toFixed(2)}% worse (threshold: ${result.threshold.toFixed(2)}%)\\n`;
    }
  }
  
  if (Object.keys(trends).length > 0) {
    message += '\\n**Performance Trends:**\\n';
    for (const [metric, trend] of Object.entries(trends)) {
      const icon = trend === 'improving' ? '📈' : trend === 'degrading' ? '📉' : '➡️';
      message += `• ${metric}: ${icon} ${trend}\\n`;
    }
  }
  
  return message;
}

async function sendSlackAlert(message, webhookUrl) {
  if (!webhookUrl) return false;
  
  try {
    const fetch = (await import('node-fetch')).default;
    const response = await fetch(webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        text: message,
        username: 'Performance Monitor',
        icon_emoji: ':warning:'
      })
    });
    
    return response.ok;
  } catch (error) {
    console.error('Failed to send Slack alert:', error.message);
    return false;
  }
}

function main() {
  console.log('🔍 UV-60: Enhanced Performance Regression Check');
  
  const config = loadConfig();
  const report = loadJSON(reportPath);
  const baseline = loadJSON(baselinePath);
  
  if (!report) {
    console.error('❌ Current benchmark report not found');
    process.exit(1);
  }
  
  if (!baseline) {
    console.log('⚠️  No baseline found. Skipping regression check.');
    console.log('   This is normal for the first run. Results will be stored as baseline.');
    process.exit(0);
  }
  
  // Update historical data
  const history = updateHistoricalData(report, config);
  const trends = analyzeTrends(history, config);
  
  console.log('📊 Analyzing performance metrics...');
  
  const regressions = {};
  let hasRegression = false;
  
  // Check each metric
  for (const [metric, threshold] of Object.entries(config.thresholds)) {
    const current = report.summary?.[metric] || report.cache_performance?.[metric];
    const baselineValue = baseline.summary?.[metric] || baseline.cache_performance?.[metric];
    
    const result = compareMetric(metric, current, baselineValue, threshold);
    regressions[metric] = result;
    
    if (!result.pass) {
      hasRegression = true;
      const icon = result.severity === 'high' ? '🔴' : '🟡';
      console.log(`${icon} Regression in ${metric}: ${result.delta.toFixed(2)}% worse`);
      console.log(`   Current: ${result.current}, Baseline: ${result.baseline}`);
    } else if (result.reason === 'pass') {
      console.log(`✅ ${metric}: No regression detected`);
    }
  }
  
  // Report trends
  if (Object.keys(trends).length > 0) {
    console.log('\\n📈 Performance Trends:');
    for (const [metric, trend] of Object.entries(trends)) {
      const icon = trend === 'improving' ? '📈' : trend === 'degrading' ? '📉' : '➡️';
      console.log(`   ${icon} ${metric}: ${trend}`);
    }
  }
  
  // Send alerts if regressions detected
  if (hasRegression) {
    const alertMessage = generateAlertMessage(regressions, trends);
    console.log('\\n🚨 Sending performance alerts...');
    
    // Slack alert
    if (config.alerts.slack_webhook) {
      sendSlackAlert(alertMessage, config.alerts.slack_webhook)
        .then(success => {
          if (success) {
            console.log('   ✅ Slack alert sent');
          } else {
            console.log('   ❌ Failed to send Slack alert');
          }
        });
    }
    
    console.log('\\n❌ Performance regression detected. See details above.');
    process.exit(1);
  }
  
  console.log('\\n✅ No significant performance regressions detected.');
  process.exit(0);
}

// Handle uncaught errors gracefully
process.on('uncaughtException', (error) => {
  console.error('❌ Unexpected error during regression check:', error.message);
  process.exit(1);
});

main();

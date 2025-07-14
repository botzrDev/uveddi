// UV-180: Automated regression check for benchmark results
const fs = require('fs');
const path = require('path');

const reportPath = path.join(__dirname, '../benchmark-report.json');
const baselinePath = path.join(__dirname, '../benchmark-baseline.json');

function loadJSON(file) {
  if (!fs.existsSync(file)) return null;
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

const report = loadJSON(reportPath);
const baseline = loadJSON(baselinePath);

if (!report || !baseline) {
  console.log('No baseline or report found. Skipping regression check.');
  process.exit(0);
}

function compareMetric(metric, threshold = 0.10) {
  const current = report.summary[metric];
  const base = baseline.summary[metric];
  if (base === undefined || current === undefined) return true;
  const delta = (base - current) / base;
  if (delta > threshold) {
    console.error(`Regression detected in ${metric}: baseline=${base}, current=${current}, delta=${(delta*100).toFixed(2)}%`);
    return false;
  }
  return true;
}

const metrics = ['overall_success_rate', 'uv78_target_compliance', 'performance_grade'];
let allPass = true;
for (const metric of metrics) {
  if (!compareMetric(metric)) allPass = false;
}
if (!allPass) process.exit(1);
console.log('No significant regression detected.');
process.exit(0);

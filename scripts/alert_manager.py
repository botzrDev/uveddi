#!/usr/bin/env python3
"""
UV-60: Performance Alert Manager
Manages alert notifications for performance regressions and threshold violations.
"""

import json
import os
import sys
import requests
import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from typing import Dict, Any, List, Optional
from datetime import datetime
import urllib.parse

class AlertManager:
    def __init__(self):
        self.config = self.load_config()
        self.alert_history_file = "performance-history/alert-history.json"
        
    def load_config(self) -> Dict[str, Any]:
        """Load alert configuration from environment and config files."""
        config = {
            "slack": {
                "webhook_url": os.environ.get("SLACK_WEBHOOK_URL"),
                "channel": os.environ.get("SLACK_CHANNEL", "#performance-alerts"),
                "username": "Performance Monitor",
                "icon_emoji": ":warning:"
            },
            "email": {
                "smtp_server": os.environ.get("SMTP_SERVER", "smtp.gmail.com"),
                "smtp_port": int(os.environ.get("SMTP_PORT", "587")),
                "username": os.environ.get("SMTP_USERNAME"),
                "password": os.environ.get("SMTP_PASSWORD"),
                "recipients": os.environ.get("ALERT_EMAIL_RECIPIENTS", "").split(","),
                "sender": os.environ.get("ALERT_EMAIL_SENDER")
            },
            "github": {
                "token": os.environ.get("GITHUB_TOKEN"),
                "repo": os.environ.get("GITHUB_REPOSITORY"),
                "create_issues": os.environ.get("CREATE_GITHUB_ISSUES", "false").lower() == "true"
            },
            "thresholds": {
                "critical": 50.0,    # 50% degradation
                "high": 35.0,        # 35% degradation
                "medium": 20.0,      # 20% degradation
                "low": 10.0          # 10% degradation
            },
            "rate_limiting": {
                "max_alerts_per_hour": 10,
                "duplicate_suppression_minutes": 60
            }
        }
        
        # Load additional config from file if exists
        config_file = "rendering-service/performance-config.json"
        if os.path.exists(config_file):
            try:
                with open(config_file, 'r') as f:
                    file_config = json.load(f)
                    config.update(file_config.get("alerts", {}))
            except Exception as e:
                print(f"Warning: Could not load config from {config_file}: {e}")
        
        return config
    
    def load_alert_history(self) -> Dict[str, Any]:
        """Load alert history to prevent spam and track patterns."""
        try:
            if os.path.exists(self.alert_history_file):
                with open(self.alert_history_file, 'r') as f:
                    return json.load(f)
        except Exception as e:
            print(f"Warning: Could not load alert history: {e}")
        
        return {"alerts": [], "last_alert_time": {}}
    
    def save_alert_history(self, history: Dict[str, Any]) -> None:
        """Save alert history for rate limiting and tracking."""
        try:
            os.makedirs(os.path.dirname(self.alert_history_file), exist_ok=True)
            with open(self.alert_history_file, 'w') as f:
                json.dump(history, f, indent=2)
        except Exception as e:
            print(f"Warning: Could not save alert history: {e}")
    
    def should_send_alert(self, alert_type: str, severity: str) -> bool:
        """Check if alert should be sent based on rate limiting and suppression rules."""
        history = self.load_alert_history()
        current_time = datetime.utcnow()
        
        # Check rate limiting
        recent_alerts = [
            alert for alert in history.get("alerts", [])
            if (current_time - datetime.fromisoformat(alert["timestamp"])).total_seconds() < 3600
        ]
        
        if len(recent_alerts) >= self.config["rate_limiting"]["max_alerts_per_hour"]:
            print(f"Rate limit exceeded: {len(recent_alerts)} alerts in the last hour")
            return False
        
        # Check duplicate suppression
        last_alert_key = f"{alert_type}_{severity}"
        last_alert_time = history.get("last_alert_time", {}).get(last_alert_key)
        
        if last_alert_time:
            time_diff = (current_time - datetime.fromisoformat(last_alert_time)).total_seconds()
            if time_diff < self.config["rate_limiting"]["duplicate_suppression_minutes"] * 60:
                print(f"Duplicate suppression: Last {alert_type} alert was {time_diff/60:.1f} minutes ago")
                return False
        
        return True
    
    def record_alert(self, alert_type: str, severity: str, message: str) -> None:
        """Record alert in history for tracking and rate limiting."""
        history = self.load_alert_history()
        current_time = datetime.utcnow()
        
        alert_record = {
            "timestamp": current_time.isoformat(),
            "type": alert_type,
            "severity": severity,
            "message": message[:200],  # Truncate for storage
            "git_commit": os.environ.get("GITHUB_SHA", "unknown")[:8],
            "git_branch": os.environ.get("GITHUB_REF_NAME", "unknown")
        }
        
        history.setdefault("alerts", []).append(alert_record)
        history.setdefault("last_alert_time", {})[f"{alert_type}_{severity}"] = current_time.isoformat()
        
        # Cleanup old alerts (keep last 100)
        history["alerts"] = history["alerts"][-100:]
        
        self.save_alert_history(history)
    
    def format_slack_message(self, alert_data: Dict[str, Any]) -> Dict[str, Any]:
        """Format alert data for Slack notification."""
        severity = alert_data.get("severity", "medium")
        severity_icons = {
            "critical": "🚨",
            "high": "🔴", 
            "medium": "🟡",
            "low": "📘"
        }
        
        icon = severity_icons.get(severity, "⚠️")
        
        # Build main message
        message = f"{icon} *Performance Alert - {severity.upper()}*\\n\\n"
        
        # Add metadata
        commit = os.environ.get("GITHUB_SHA", "unknown")[:8]
        branch = os.environ.get("GITHUB_REF_NAME", "unknown")
        pr_number = os.environ.get("GITHUB_PR_NUMBER")
        
        message += f"*Commit:* `{commit}`\\n"
        message += f"*Branch:* `{branch}`\\n"
        
        if pr_number:
            repo = os.environ.get("GITHUB_REPOSITORY", "")
            pr_url = f"https://github.com/{repo}/pull/{pr_number}"
            message += f"*Pull Request:* <{pr_url}|#{pr_number}>\\n"
        
        message += f"*Time:* {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S')} UTC\\n\\n"
        
        # Add regression details
        regressions = alert_data.get("regressions", {})
        if regressions:
            message += "*Performance Regressions:*\\n"
            for metric, details in regressions.items():
                if not details.get("is_regression", False):
                    continue
                change = details.get("change_percent", 0)
                severity_level = details.get("severity", "unknown")
                message += f"• *{metric}*: {change:+.1f}% ({severity_level})\\n"
        
        # Add trend information
        trends = alert_data.get("trends", {})
        if trends:
            message += "\\n*Performance Trends:*\\n"
            for metric, trend in trends.items():
                trend_icon = "📈" if trend == "improving" else "📉" if trend == "degrading" else "➡️"
                message += f"• {metric}: {trend_icon} {trend}\\n"
        
        # Add action items
        message += "\\n*Recommended Actions:*\\n"
        message += "• Review recent code changes for performance impact\\n"
        message += "• Run profiler to identify bottlenecks\\n"
        message += "• Check for memory leaks or excessive allocations\\n"
        
        return {
            "text": message,
            "username": self.config["slack"]["username"],
            "icon_emoji": self.config["slack"]["icon_emoji"],
            "channel": self.config["slack"]["channel"]
        }
    
    def send_slack_alert(self, alert_data: Dict[str, Any]) -> bool:
        """Send alert to Slack webhook."""
        webhook_url = self.config["slack"]["webhook_url"]
        if not webhook_url:
            print("No Slack webhook URL configured")
            return False
        
        try:
            payload = self.format_slack_message(alert_data)
            
            response = requests.post(
                webhook_url,
                json=payload,
                timeout=10
            )
            
            if response.status_code == 200:
                print("✅ Slack alert sent successfully")
                return True
            else:
                print(f"❌ Slack alert failed: {response.status_code} - {response.text}")
                return False
                
        except Exception as e:
            print(f"❌ Failed to send Slack alert: {e}")
            return False
    
    def format_email_content(self, alert_data: Dict[str, Any]) -> tuple[str, str]:
        """Format alert data for email notification."""
        severity = alert_data.get("severity", "medium")
        
        # Subject
        commit = os.environ.get("GITHUB_SHA", "unknown")[:8]
        subject = f"Performance Alert ({severity.upper()}) - Commit {commit}"
        
        # HTML Body
        html_body = f"""
        <html>
        <body style="font-family: Arial, sans-serif; margin: 20px;">
            <h2 style="color: #d73027;">🚨 Performance Regression Alert</h2>
            
            <h3>Build Information</h3>
            <ul>
                <li><strong>Commit:</strong> {commit}</li>
                <li><strong>Branch:</strong> {os.environ.get("GITHUB_REF_NAME", "unknown")}</li>
                <li><strong>Time:</strong> {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S')} UTC</li>
                <li><strong>Severity:</strong> {severity.upper()}</li>
            </ul>
        """
        
        # Add regression details
        regressions = alert_data.get("regressions", {})
        if regressions:
            html_body += "<h3>Performance Regressions</h3><table border='1' cellpadding='5'>"
            html_body += "<tr><th>Metric</th><th>Change</th><th>Severity</th></tr>"
            
            for metric, details in regressions.items():
                if not details.get("is_regression", False):
                    continue
                change = details.get("change_percent", 0)
                severity_level = details.get("severity", "unknown")
                color = "#d73027" if severity_level in ["high", "critical"] else "#fdae61"
                html_body += f"<tr><td>{metric}</td><td style='color: {color};'>{change:+.1f}%</td><td>{severity_level}</td></tr>"
            
            html_body += "</table>"
        
        # Add recommendations
        html_body += """
            <h3>Recommended Actions</h3>
            <ul>
                <li>Review recent code changes for performance impact</li>
                <li>Run profiler to identify bottlenecks</li>
                <li>Check for memory leaks or excessive allocations</li>
                <li>Consider rolling back if regression is severe</li>
            </ul>
            
            <p><em>This is an automated alert from the UV-60 Performance Monitoring System.</em></p>
        </body>
        </html>
        """
        
        return subject, html_body
    
    def send_email_alert(self, alert_data: Dict[str, Any]) -> bool:
        """Send alert via email."""
        email_config = self.config["email"]
        
        if not email_config["username"] or not email_config["recipients"]:
            print("Email configuration incomplete")
            return False
        
        try:
            subject, html_body = self.format_email_content(alert_data)
            
            msg = MIMEMultipart('alternative')
            msg['Subject'] = subject
            msg['From'] = email_config["sender"] or email_config["username"]
            msg['To'] = ", ".join(email_config["recipients"])
            
            # Add HTML content
            html_part = MIMEText(html_body, 'html')
            msg.attach(html_part)
            
            # Send email
            with smtplib.SMTP(email_config["smtp_server"], email_config["smtp_port"]) as server:
                server.starttls()
                server.login(email_config["username"], email_config["password"])
                server.send_message(msg)
            
            print(f"✅ Email alert sent to {len(email_config['recipients'])} recipients")
            return True
            
        except Exception as e:
            print(f"❌ Failed to send email alert: {e}")
            return False
    
    def create_github_issue(self, alert_data: Dict[str, Any]) -> bool:
        """Create GitHub issue for performance regression."""
        github_config = self.config["github"]
        
        if not github_config["token"] or not github_config["repo"] or not github_config["create_issues"]:
            print("GitHub issue creation not configured or disabled")
            return False
        
        try:
            commit = os.environ.get("GITHUB_SHA", "unknown")[:8]
            severity = alert_data.get("severity", "medium")
            
            title = f"Performance Regression Detected ({severity.upper()}) - {commit}"
            
            body = f"""## Performance Regression Alert
            
**Severity:** {severity.upper()}
**Commit:** {commit}
**Branch:** {os.environ.get("GITHUB_REF_NAME", "unknown")}
**Detection Time:** {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S')} UTC

### Regression Details
"""
            
            regressions = alert_data.get("regressions", {})
            if regressions:
                body += "| Metric | Change | Severity |\\n|--------|--------|----------|\\n"
                for metric, details in regressions.items():
                    if details.get("is_regression", False):
                        change = details.get("change_percent", 0)
                        severity_level = details.get("severity", "unknown")
                        body += f"| {metric} | {change:+.1f}% | {severity_level} |\\n"
            
            body += """
### Recommended Actions
- [ ] Review recent code changes for performance impact
- [ ] Run profiler to identify bottlenecks  
- [ ] Check for memory leaks or excessive allocations
- [ ] Consider performance optimizations
- [ ] Update performance baselines if changes are intentional

### Additional Information
This issue was automatically created by the UV-60 Performance Monitoring System.
"""
            
            headers = {
                "Authorization": f"token {github_config['token']}",
                "Accept": "application/vnd.github.v3+json"
            }
            
            payload = {
                "title": title,
                "body": body,
                "labels": ["performance", "regression", f"severity-{severity}"]
            }
            
            response = requests.post(
                f"https://api.github.com/repos/{github_config['repo']}/issues",
                headers=headers,
                json=payload,
                timeout=10
            )
            
            if response.status_code == 201:
                issue_url = response.json().get("html_url", "")
                print(f"✅ GitHub issue created: {issue_url}")
                return True
            else:
                print(f"❌ Failed to create GitHub issue: {response.status_code} - {response.text}")
                return False
                
        except Exception as e:
            print(f"❌ Failed to create GitHub issue: {e}")
            return False
    
    def send_performance_alert(self, alert_data: Dict[str, Any]) -> bool:
        """Send performance alert through all configured channels."""
        severity = alert_data.get("severity", "medium")
        alert_type = "performance_regression"
        
        # Check if we should send this alert
        if not self.should_send_alert(alert_type, severity):
            return False
        
        print(f"🚨 Sending {severity} performance alert...")
        
        success_count = 0
        total_channels = 0
        
        # Send Slack alert
        if self.config["slack"]["webhook_url"]:
            total_channels += 1
            if self.send_slack_alert(alert_data):
                success_count += 1
        
        # Send email alert
        if self.config["email"]["username"] and self.config["email"]["recipients"]:
            total_channels += 1
            if self.send_email_alert(alert_data):
                success_count += 1
        
        # Create GitHub issue for high severity
        if severity in ["high", "critical"] and self.config["github"]["create_issues"]:
            total_channels += 1
            if self.create_github_issue(alert_data):
                success_count += 1
        
        # Record the alert
        alert_summary = f"Performance regression detected: {len(alert_data.get('regressions', {}))} metrics regressed"
        self.record_alert(alert_type, severity, alert_summary)
        
        print(f"📊 Alert sent through {success_count}/{total_channels} channels")
        return success_count > 0

def main():
    """Main entry point for alert manager."""
    if len(sys.argv) < 2:
        print("Usage: alert_manager.py <alert_data_file>")
        sys.exit(1)
    
    alert_data_file = sys.argv[1]
    
    try:
        with open(alert_data_file, 'r') as f:
            alert_data = json.load(f)
    except Exception as e:
        print(f"❌ Failed to load alert data from {alert_data_file}: {e}")
        sys.exit(1)
    
    manager = AlertManager()
    success = manager.send_performance_alert(alert_data)
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
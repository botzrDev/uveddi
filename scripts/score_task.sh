#!/bin/bash
# Task Complexity Scorer for Uveddi Contributors
# Usage: ./score_task.sh "task description"

echo "Uveddi Task Complexity Scorer"
echo "============================="
echo ""

if [ $# -eq 0 ]; then
    echo "Usage: $0 \"task description\""
    echo ""
    echo "Example: $0 \"Add CSV output format for analysis reports\""
    exit 1
fi

TASK_DESCRIPTION="$1"
echo "Scoring task: $TASK_DESCRIPTION"
echo ""

echo "Rate each aspect from 1-5 (press Enter for each):"
echo ""

read -p "Technical Complexity (1=typos, 2=small features, 3=new features, 4=architecture, 5=core engine): " tech
read -p "Codebase Knowledge (1=none needed, 2=basic, 3=specific modules, 4=deep understanding, 5=expert): " knowledge  
read -p "External Dependencies (1=none, 2=git/editor, 3=dev tools, 4=specialized, 5=infrastructure): " deps
read -p "Testing Requirements (1=none, 2=unit tests, 3=integration, 4=e2e, 5=performance): " testing
read -p "Documentation Impact (1=none, 2=update existing, 3=new docs, 4=API docs, 5=architecture): " docs

echo ""

# Validate inputs
if ! [[ "$tech" =~ ^[1-5]$ ]] || ! [[ "$knowledge" =~ ^[1-5]$ ]] || ! [[ "$deps" =~ ^[1-5]$ ]] || ! [[ "$testing" =~ ^[1-5]$ ]] || ! [[ "$docs" =~ ^[1-5]$ ]]; then
    echo "Error: All ratings must be numbers between 1 and 5"
    exit 1
fi

# Calculate total score
total=$(echo "scale=1; ($tech + $knowledge + $deps + $testing + $docs) / 5" | bc)

echo "Scoring Results:"
echo "==============="
echo "Technical Complexity:  $tech/5"
echo "Codebase Knowledge:    $knowledge/5"
echo "External Dependencies: $deps/5"
echo "Testing Requirements:  $testing/5"
echo "Documentation Impact:  $docs/5"
echo ""
echo "Total Score: $total/5"
echo ""

# Determine recommendation and labels
if (( $(echo "$total <= 1.5" | bc -l) )); then
    echo "🎯 Recommendation: Perfect for first-time contributors"
    echo "📝 Suggested Labels: good-first-issue, difficulty/beginner, time/quick-win"
    echo "⏱️  Estimated Time: 1-2 hours"
    echo "👥 Mentorship: Optional, basic guidance"
elif (( $(echo "$total <= 2.5" | bc -l) )); then
    echo "🎯 Recommendation: Good for beginners with some experience"
    echo "📝 Suggested Labels: beginner-friendly, difficulty/beginner"
    echo "⏱️  Estimated Time: 2-4 hours"
    echo "👥 Mentorship: Recommended for guidance"
elif (( $(echo "$total <= 3.5" | bc -l) )); then
    echo "🎯 Recommendation: Suitable for intermediate contributors"
    echo "📝 Suggested Labels: difficulty/intermediate"
    echo "⏱️  Estimated Time: 4-8 hours"
    echo "👥 Mentorship: Required for complex aspects"
else
    echo "🎯 Recommendation: Advanced contributors only"
    echo "📝 Suggested Labels: difficulty/advanced"
    echo "⏱️  Estimated Time: 8+ hours"
    echo "👥 Mentorship: Expert guidance required"
fi

echo ""
echo "Issue Preparation Checklist:"
echo "============================"
echo "□ Create detailed issue description"
echo "□ Add step-by-step implementation guide"
echo "□ Include code examples and resources"
echo "□ Assign appropriate mentor"
echo "□ Add complexity score to issue body"
echo "□ Apply recommended labels"
echo "□ Define clear acceptance criteria"

echo ""
echo "Next Steps:"
echo "==========="
echo "1. Copy this scoring information to your GitHub issue"
echo "2. Use the Issue Preparation Template to enhance the task description"
echo "3. Assign a mentor based on the task type and complexity"
echo "4. Apply the suggested labels to help contributors find appropriate tasks"

echo ""
echo "For more information, see:"
echo "- docs/09-community/TASK_COMPLEXITY_SCORING.md"
echo "- docs/09-community/ISSUE_PREPARATION_TEMPLATE.md"
echo "- docs/09-community/MENTORSHIP_SYSTEM.md"
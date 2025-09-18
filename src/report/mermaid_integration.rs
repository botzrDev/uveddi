//! Mermaid Diagram Integration Module
//!
//! This module handles Mermaid.js diagram generation and rendering,
//! including diagram creation, formatting, and interactive features.

use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::report::{ArchitecturalComponent, ReportGenerator};
use std::collections::HashMap;

impl ReportGenerator {
    /// Generate Mermaid diagram with instructions for various rendering options
    pub fn generate_mermaid_only_with_instructions(
        &self,
        mermaid_code: &str,
        diagram_type: &str,
    ) -> String {
        format!(
            r#"## 📊 {} Diagram

```mermaid
{}
```

> **💡 Want to see this as an image?**
>
> **Option 1: Online Rendering (Fastest)**
> - Copy the Mermaid code above
> - Visit [mermaid.live](https://mermaid.live)
> - Paste and generate your image instantly
>
> **Option 2: Local Rendering Service (Full Control)**
> ```bash
> # Run this in your project directory
> docker-compose up rendering-service
>
> # Then re-run analysis with image rendering
> uveddi analyze --enable-image-rendering
> ```
>
> **Option 3: VS Code Extension (Developer Friendly)**
> - Install "Mermaid Markdown Syntax Highlighting"
> - View diagrams directly in your editor
>
> **Option 4: GitHub/GitLab (Documentation)**
> - Both platforms render Mermaid diagrams natively
> - Perfect for README files and documentation

"#,
            diagram_type, mermaid_code
        )
    }

    /// Generate diagrams section with Mermaid.js syntax
    pub fn generate_diagrams_section(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let mut diagrams = String::new();

        // Add dependency cycles diagram if there are cyclic dependency issues
        let cycle_issues: Vec<&ArchitecturalIssue> = issues
            .iter()
            .filter(|issue| {
                issue.description.to_lowercase().contains("cyclic")
                    || issue.description.to_lowercase().contains("circular")
                    || issue.description.to_lowercase().contains("dependency")
            })
            .collect();

        if !cycle_issues.is_empty() {
            let mermaid_code = self.generate_dependency_cycle_diagram(&cycle_issues);
            diagrams.push_str(
                &self.generate_mermaid_only_with_instructions(&mermaid_code, "Dependency Cycles"),
            );
        }

        // Add god object complexity diagram
        let god_object_issues: Vec<&ArchitecturalIssue> = issues
            .iter()
            .filter(|issue| {
                issue.description.to_lowercase().contains("god")
                    || issue.description.to_lowercase().contains("large")
                    || issue.description.to_lowercase().contains("complex")
            })
            .collect();

        if !god_object_issues.is_empty() {
            let mermaid_code = self.generate_god_object_diagram(&god_object_issues);
            diagrams.push_str(&self.generate_mermaid_only_with_instructions(
                &mermaid_code,
                "God Objects & Complexity",
            ));
        }

        // Add anti-pattern distribution diagram
        if !issues.is_empty() {
            let mermaid_code =
                self.generate_anti_pattern_distribution_diagram(issues, anti_pattern_types);
            diagrams.push_str(&self.generate_mermaid_only_with_instructions(
                &mermaid_code,
                "Anti-Pattern Distribution",
            ));
        }

        if diagrams.is_empty() {
            diagrams.push_str("No architectural diagrams available for this analysis.\n\n");
        }

        diagrams
    }

    /// Render Mermaid diagram as HTML with theme support
    pub fn render_mermaid_diagram(&self, title: &str, mermaid_code: &str) -> String {
        format!(
            r#"
    <div class="card">
        <h3><i class="fas fa-project-diagram"></i> {}</h3>
        <div class="mermaid-container">
            <div class="mermaid" id="diagram-{}">{}</div>
        </div>
    </div>

    <script>
        // Initialize this specific diagram
        setTimeout(() => {{
            if (typeof mermaid !== 'undefined') {{
                const element = document.getElementById('diagram-{}');
                if (element && !element.querySelector('svg')) {{
                    const diagramId = 'diagram-{}';
                    const graphDefinition = `{}`;

                    // Configure theme based on current mode
                    mermaid.initialize({{
                        startOnLoad: false,
                        theme: document.body.getAttribute('data-theme') === 'dark' ? 'dark' : 'default',
                        themeVariables: document.body.getAttribute('data-theme') === 'dark' ? {{
                            primaryColor: '#3b82f6',
                            primaryTextColor: '#f1f5f9',
                            primaryBorderColor: '#64748b',
                            lineColor: '#64748b',
                            background: '#0f172a',
                            secondaryColor: '#1e293b',
                            tertiaryColor: '#334155'
                        }} : {{
                            primaryColor: '#2563eb',
                            primaryTextColor: '#0f172a',
                            primaryBorderColor: '#64748b',
                            lineColor: '#64748b',
                            background: '#ffffff',
                            secondaryColor: '#f8fafc',
                            tertiaryColor: '#e2e8f0'
                        }}
                    }});

                    mermaid.render(diagramId, graphDefinition, (svgCode) => {{
                        element.innerHTML = svgCode;
                    }});
                }}
            }}
        }}, 100);
    </script>
"#,
            title,
            title.to_lowercase().replace(" ", "-"),
            mermaid_code,
            title.to_lowercase().replace(" ", "-"),
            title.to_lowercase().replace(" ", "-"),
            mermaid_code.replace("`", "\\`")
        )
    }

    /// Generate dependency cycle diagram in Mermaid format
    fn generate_dependency_cycle_diagram(&self, cycle_issues: &[&ArchitecturalIssue]) -> String {
        let mut mermaid = String::from("graph TD\n");
        let mut nodes = std::collections::HashSet::new();

        for issue in cycle_issues {
            // Extract module names from file paths
            let parts: Vec<&str> = issue.file_path.split('/').collect();
            let module_name = parts.last().map_or("unknown", |v| v).replace(".rs", "");
            nodes.insert(module_name.clone());

            // Create a simple cycle representation
            mermaid.push_str(&format!(
                "    {}[{}]\n",
                module_name.replace("-", "_"),
                module_name
            ));
        }

        // Add cycle connections (simplified)
        let node_list: Vec<_> = nodes.iter().collect();
        for i in 0..node_list.len() {
            let current = node_list[i].replace("-", "_");
            let next = node_list[(i + 1) % node_list.len()].replace("-", "_");
            mermaid.push_str(&format!("    {} -->|depends on| {}\n", current, next));
        }

        // Add styling
        mermaid.push_str("    classDef cycle fill:#ff6b6b,stroke:#333,stroke-width:2px\n");
        for node in &nodes {
            mermaid.push_str(&format!("    class {} cycle\n", node.replace("-", "_")));
        }

        mermaid
    }

    /// Generate god object complexity diagram
    fn generate_god_object_diagram(&self, god_object_issues: &[&ArchitecturalIssue]) -> String {
        let mut mermaid = String::from("graph TB\n");

        for (i, issue) in god_object_issues.iter().enumerate() {
            let parts: Vec<&str> = issue.file_path.split('/').collect();
            let module_name = parts.last().map_or("unknown", |v| v).replace(".rs", "");
            let node_id = format!("obj{}", i);

            mermaid.push_str(&format!("    {}[\"{}\"]\n", node_id, module_name));

            // Add complexity indicator
            mermaid.push_str(&format!("    {}complexity[\"High Complexity\"]\n", node_id));
            mermaid.push_str(&format!("    {} --> {}complexity\n", node_id, node_id));
        }

        // Add styling for god objects
        mermaid.push_str("    classDef godObject fill:#ffd93d,stroke:#333,stroke-width:2px\n");
        mermaid.push_str("    classDef complexity fill:#ff6b6b,stroke:#333,stroke-width:1px\n");

        for i in 0..god_object_issues.len() {
            mermaid.push_str(&format!("    class obj{} godObject\n", i));
            mermaid.push_str(&format!("    class obj{}complexity complexity\n", i));
        }

        mermaid
    }

    /// Generate anti-pattern distribution pie chart
    fn generate_anti_pattern_distribution_diagram(
        &self,
        issues: &[ArchitecturalIssue],
        _anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();

        for issue in issues {
            *pattern_counts.entry(issue.description.clone()).or_insert(0) += 1;
        }

        let mut mermaid = String::from("pie title Anti-Pattern Distribution\n");

        for (pattern, count) in pattern_counts {
            mermaid.push_str(&format!("    \"{}\" : {}\n", pattern, count));
        }

        mermaid
    }

    /// Generate enhanced diagrams section with interactive features
    pub fn generate_enhanced_diagrams_section(
        &self,
        issues: &[ArchitecturalIssue],
        components: Option<&[ArchitecturalComponent]>,
        diagrams: &[crate::report::interactive_models::DiagramDefinition],
    ) -> String {
        let mut section = String::new();

        section.push_str("## 📊 Architectural Diagrams\n\n");

        // Add custom diagrams if available
        if !diagrams.is_empty() {
            for diagram in diagrams {
                section.push_str(
                    &self.generate_mermaid_only_with_instructions(&diagram.source, &diagram.title),
                );
            }
        }

        // Add auto-generated diagrams
        section.push_str(&self.generate_diagrams_section(issues, &HashMap::new()));

        // Add component architecture diagram if components are available
        if let Some(comps) = components {
            if !comps.is_empty() {
                let component_diagram = self.generate_component_architecture_diagram(comps);
                section.push_str(&self.generate_mermaid_only_with_instructions(
                    &component_diagram,
                    "Component Architecture",
                ));
            }
        }

        section
    }

    /// Generate component architecture diagram
    fn generate_component_architecture_diagram(
        &self,
        components: &[ArchitecturalComponent],
    ) -> String {
        let mut mermaid = String::from("graph TD\n");

        for component in components {
            let comp_id = component.name.replace("-", "_").replace(" ", "_");
            mermaid.push_str(&format!("    {}[\"{}\"]\n", comp_id, component.name));

            // Add dependencies
            for dep in &component.dependencies {
                let dep_id = format!("{:?}", dep).replace("-", "_").replace(" ", "_");
                mermaid.push_str(&format!("    {} --> {}\n", comp_id, dep_id));
            }
        }

        // Add styling based on component type
        mermaid.push_str("    classDef service fill:#4CAF50,stroke:#333,stroke-width:2px\n");
        mermaid.push_str("    classDef model fill:#2196F3,stroke:#333,stroke-width:2px\n");
        mermaid.push_str("    classDef util fill:#FF9800,stroke:#333,stroke-width:2px\n");

        for component in components {
            let comp_id = component.name.replace("-", "_").replace(" ", "_");
            let class_name = match component.component_type.as_str() {
                "service" => "service",
                "model" => "model",
                _ => "util",
            };
            mermaid.push_str(&format!("    class {} {}\n", comp_id, class_name));
        }

        mermaid
    }
}

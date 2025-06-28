use std::io::{self, Write};
use crate::analysis::dependency_graph::{AnalysisResults, CycleSeverity};

/// Simple text-based report generator
pub struct TextReporter {
    show_file_paths: bool,
    show_line_numbers: bool,
}

impl TextReporter {
    pub fn new() -> Self {
        Self {
            show_file_paths: true,
            show_line_numbers: true,
        }
    }

    pub fn with_file_paths(mut self, show: bool) -> Self {
        self.show_file_paths = show;
        self
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Generate and write report to stdout
    pub fn generate_report(&self, results: &AnalysisResults) -> io::Result<()> {
        let mut output = io::stdout();
        self.write_report(&mut output, results)
    }

    /// Write report to any writer
    pub fn write_report<W: Write>(&self, writer: &mut W, results: &AnalysisResults) -> io::Result<()> {
        writeln!(writer, "CodeAtlas Analysis Report")?;
        writeln!(writer, "=========================")?;
        writeln!(writer)?;
        self.write_summary(writer, results)?;
        writeln!(writer)?;
        if results.cycles.is_empty() {
            writeln!(writer, "✅ No cyclic dependencies found!")?;
        } else {
            self.write_cycles(writer, results)?;
        }
        writeln!(writer)?;
        writeln!(writer, "Analysis completed in {:?}", results.analysis_duration)?;
        Ok(())
    }

    fn write_summary<W: Write>(&self, writer: &mut W, results: &AnalysisResults) -> io::Result<()> {
        writeln!(writer, "Summary:")?;
        writeln!(writer, "--------")?;
        writeln!(writer, "📁 Total modules analyzed: {}", results.total_modules)?;
        writeln!(writer, "🔗 Total dependencies: {}", results.total_dependencies)?;
        writeln!(writer, "🔄 Cyclic dependencies found: {}", results.cycles.len())?;
        let (low, medium, high) = self.count_by_severity(results);
        if results.cycles.len() > 0 {
            writeln!(writer, "   └─ Low severity: {}", low)?;
            writeln!(writer, "   └─ Medium severity: {}", medium)?;
            writeln!(writer, "   └─ High severity: {}", high)?;
        }
        Ok(())
    }

    fn write_cycles<W: Write>(&self, writer: &mut W, results: &AnalysisResults) -> io::Result<()> {
        writeln!(writer, "Cyclic Dependencies:")?;
        writeln!(writer, "-------------------")?;
        for (index, cycle) in results.cycles.iter().enumerate() {
            let severity_icon = match cycle.severity {
                CycleSeverity::Low => "🟡",
                CycleSeverity::Medium => "🟠", 
                CycleSeverity::High => "🔴",
            };
            writeln!(writer, "{}. {} {:?} Severity Cycle:", 
                     index + 1, severity_icon, cycle.severity)?;
            write!(writer, "   ")?;
            for (i, module) in cycle.modules.iter().enumerate() {
                if i > 0 {
                    write!(writer, " → ")?;
                }
                write!(writer, "{}", module)?;
            }
            if !cycle.modules.is_empty() {
                write!(writer, " → {}", cycle.modules[0])?;
            }
            writeln!(writer)?;
            if self.show_file_paths {
                for file_path in &cycle.file_paths {
                    writeln!(writer, "     📄 {}", file_path.display())?;
                }
            }
            writeln!(writer)?;
        }
        Ok(())
    }

    fn count_by_severity(&self, results: &AnalysisResults) -> (usize, usize, usize) {
        let mut low = 0;
        let mut medium = 0;
        let mut high = 0;
        for cycle in &results.cycles {
            match cycle.severity {
                CycleSeverity::Low => low += 1,
                CycleSeverity::Medium => medium += 1,
                CycleSeverity::High => high += 1,
            }
        }
        (low, medium, high)
    }
}

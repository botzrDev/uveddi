use super::matrix_builder::CouplingMatrix;

/// Heatmap visualization data structure
#[derive(Debug, Clone)]
pub struct CouplingHeatmapData {
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub values: Vec<Vec<f64>>,
    pub max_value: f64,
    pub min_value: f64,
}

/// Generates visualization data from coupling matrices
#[derive(Debug, Clone)]
pub struct HeatmapGenerator;

impl HeatmapGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate heatmap data for web visualization
    pub fn generate_heatmap_data(&self, matrix: &CouplingMatrix) -> CouplingHeatmapData {
        let rows = matrix.labels.clone();
        let columns = matrix.labels.clone();
        let values = matrix.matrix.clone();

        let all_values: Vec<f64> = values.iter().flatten().cloned().collect();
        let max_value = all_values.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_value = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        CouplingHeatmapData {
            rows,
            columns,
            values,
            max_value,
            min_value: min_value.min(0.0),
        }
    }

    /// Generate CSV format for the coupling matrix
    pub fn generate_csv(&self, matrix: &CouplingMatrix) -> String {
        let mut csv = String::new();

        // Header row
        csv.push_str(",");
        for label in &matrix.labels {
            csv.push_str(&format!("\"{}\",", label));
        }
        csv.pop(); // Remove trailing comma
        csv.push('\n');

        // Data rows
        for (i, row) in matrix.matrix.iter().enumerate() {
            csv.push_str(&format!("\"{}\",", matrix.labels[i]));
            for value in row {
                csv.push_str(&format!("{:.3},", value));
            }
            csv.pop(); // Remove trailing comma
            csv.push('\n');
        }

        csv
    }

    /// Generate JSON format for web visualization
    pub fn generate_json(&self, matrix: &CouplingMatrix) -> String {
        let heatmap_data = self.generate_heatmap_data(matrix);

        format!(
            r#"{{
  "rows": {:?},
  "columns": {:?},
  "values": {:?},
  "maxValue": {},
  "minValue": {}
}}"#,
            heatmap_data.rows,
            heatmap_data.columns,
            heatmap_data.values,
            heatmap_data.max_value,
            heatmap_data.min_value
        )
    }
}

impl Default for HeatmapGenerator {
    fn default() -> Self {
        Self::new()
    }
}
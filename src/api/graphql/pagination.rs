//! Pagination utilities for GraphQL connections

use async_graphql::{Error, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};

use super::types::PageInfo;

/// Cursor-based pagination parameters
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub first: Option<usize>,
    pub after: Option<String>,
    pub last: Option<usize>,
    pub before: Option<String>,
}

impl PaginationParams {
    pub fn new(
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> Result<Self> {
        // Validate pagination parameters
        if first.is_some() && last.is_some() {
            return Err(Error::new("Cannot specify both 'first' and 'last'"));
        }

        if after.is_some() && before.is_some() {
            return Err(Error::new("Cannot specify both 'after' and 'before'"));
        }

        let first = first
            .map(|f| {
                if f < 0 {
                    return Err(Error::new("'first' must be non-negative"));
                }
                if f > 100 {
                    return Err(Error::new("'first' cannot exceed 100"));
                }
                Ok(f as usize)
            })
            .transpose()?;

        let last = last
            .map(|l| {
                if l < 0 {
                    return Err(Error::new("'last' must be non-negative"));
                }
                if l > 100 {
                    return Err(Error::new("'last' cannot exceed 100"));
                }
                Ok(l as usize)
            })
            .transpose()?;

        Ok(Self {
            first,
            after,
            last,
            before,
        })
    }

    /// Get the limit for database queries
    pub fn limit(&self) -> usize {
        self.first.or(self.last).unwrap_or(10).min(100) // Hard limit
    }

    /// Check if we're paginating forward
    pub fn is_forward(&self) -> bool {
        self.first.is_some() || self.after.is_some()
    }

    /// Check if we're paginating backward
    pub fn is_backward(&self) -> bool {
        self.last.is_some() || self.before.is_some()
    }
}

/// Generic cursor implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub id: i64,
    pub secondary: Option<String>, // For complex sorting
}

impl Cursor {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            secondary: None,
        }
    }

    pub fn with_secondary(id: i64, secondary: String) -> Self {
        Self {
            id,
            secondary: Some(secondary),
        }
    }

    /// Encode cursor to base64 string
    pub fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(self)
            .map_err(|e| Error::new(format!("Failed to serialize cursor: {}", e)))?;

        Ok(BASE64.encode(json.as_bytes()))
    }

    /// Decode cursor from base64 string
    pub fn decode(cursor: &str) -> Result<Self> {
        let decoded = BASE64
            .decode(cursor.as_bytes())
            .map_err(|e| Error::new(format!("Invalid cursor format: {}", e)))?;

        let json = String::from_utf8(decoded)
            .map_err(|e| Error::new(format!("Invalid cursor encoding: {}", e)))?;

        serde_json::from_str(&json).map_err(|e| Error::new(format!("Invalid cursor data: {}", e)))
    }
}

/// Connection utilities for building paginated responses
pub struct ConnectionBuilder;

impl ConnectionBuilder {
    /// Build PageInfo for a connection
    pub fn build_page_info<T>(
        items: &[T],
        params: &PaginationParams,
        has_more: bool,
        cursor_fn: impl Fn(&T) -> Result<String>,
    ) -> Result<PageInfo> {
        let has_previous_page = params.after.is_some() || params.before.is_some();
        let has_next_page = if params.is_forward() {
            has_more
        } else {
            params.before.is_some()
        };

        let start_cursor = items.first().map(&cursor_fn).transpose()?;
        let end_cursor = items.last().map(&cursor_fn).transpose()?;

        Ok(PageInfo {
            has_next_page,
            has_previous_page,
            start_cursor,
            end_cursor,
        })
    }

    /// Apply cursor-based filtering to a query
    pub fn apply_cursor_filter(
        params: &PaginationParams,
        base_query: &str,
    ) -> Result<(String, Vec<String>)> {
        let mut query = base_query.to_string();
        let mut bind_params = Vec::new();

        if let Some(after) = &params.after {
            let cursor = Cursor::decode(after)?;
            query.push_str(" AND id > ?");
            bind_params.push(cursor.id.to_string());
        }

        if let Some(before) = &params.before {
            let cursor = Cursor::decode(before)?;
            query.push_str(" AND id < ?");
            bind_params.push(cursor.id.to_string());
        }

        // Add ordering
        if params.is_backward() {
            query.push_str(" ORDER BY id DESC");
        } else {
            query.push_str(" ORDER BY id ASC");
        }

        // Add limit
        query.push_str(" LIMIT ?");
        bind_params.push((params.limit() + 1).to_string()); // +1 to check for more results

        Ok((query, bind_params))
    }
}

/// Utility for handling complex sorting with multiple fields
#[derive(Debug, Clone)]
pub struct SortField {
    pub column: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

/// Builder for complex sorting queries
pub struct SortBuilder {
    fields: Vec<SortField>,
}

impl SortBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_field(mut self, column: String, direction: SortDirection) -> Self {
        self.fields.push(SortField { column, direction });
        self
    }

    pub fn build_order_clause(&self) -> String {
        if self.fields.is_empty() {
            return "ORDER BY id ASC".to_string();
        }

        let clauses: Vec<String> = self
            .fields
            .iter()
            .map(|field| format!("{} {}", field.column, field.direction.as_sql()))
            .collect();

        format!("ORDER BY {}", clauses.join(", "))
    }
}

/// Pagination result wrapper
#[derive(Debug)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub has_more: bool,
    pub total_count: Option<i64>,
}

impl<T> PaginationResult<T> {
    pub fn new(mut items: Vec<T>, limit: usize, total_count: Option<i64>) -> Self {
        let has_more = items.len() > limit;

        // Remove the extra item we fetched to check for more results
        if has_more {
            items.pop();
        }

        Self {
            items,
            has_more,
            total_count,
        }
    }
}

/// Trait for entities that can be used in cursor pagination
pub trait CursorEntity {
    fn cursor_id(&self) -> i64;
    fn cursor_secondary(&self) -> Option<String> {
        None
    }

    fn to_cursor(&self) -> Result<String> {
        let cursor = if let Some(secondary) = self.cursor_secondary() {
            Cursor::with_secondary(self.cursor_id(), secondary)
        } else {
            Cursor::new(self.cursor_id())
        };

        cursor.encode()
    }
}

// Implement CursorEntity for our main types
impl CursorEntity for ArchitecturalIssue {
    fn cursor_id(&self) -> i64 {
        self.id.parse().unwrap_or(0)
    }
}

impl CursorEntity for AnalysisRun {
    fn cursor_id(&self) -> i64 {
        self.id.parse().unwrap_or(0)
    }
}

impl CursorEntity for Project {
    fn cursor_id(&self) -> i64 {
        self.id.parse().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_encode_decode() {
        let cursor = Cursor::new(123);
        let encoded = cursor.encode().unwrap();
        let decoded = Cursor::decode(&encoded).unwrap();

        assert_eq!(cursor.id, decoded.id);
        assert_eq!(cursor.secondary, decoded.secondary);
    }

    #[test]
    fn test_cursor_with_secondary() {
        let cursor = Cursor::with_secondary(456, "test".to_string());
        let encoded = cursor.encode().unwrap();
        let decoded = Cursor::decode(&encoded).unwrap();

        assert_eq!(cursor.id, decoded.id);
        assert_eq!(cursor.secondary, decoded.secondary);
    }

    #[test]
    fn test_pagination_params_validation() {
        // Should fail with both first and last
        let result = PaginationParams::new(Some(10), None, Some(5), None);
        assert!(result.is_err());

        // Should fail with both after and before
        let result = PaginationParams::new(
            Some(10),
            Some("cursor".to_string()),
            None,
            Some("cursor2".to_string()),
        );
        assert!(result.is_err());

        // Should fail with negative first
        let result = PaginationParams::new(Some(-1), None, None, None);
        assert!(result.is_err());

        // Should succeed with valid params
        let result = PaginationParams::new(Some(10), Some("cursor".to_string()), None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_builder() {
        let sort = SortBuilder::new()
            .add_field("created_at".to_string(), SortDirection::Desc)
            .add_field("id".to_string(), SortDirection::Asc);

        let clause = sort.build_order_clause();
        assert_eq!(clause, "ORDER BY created_at DESC, id ASC");
    }
}

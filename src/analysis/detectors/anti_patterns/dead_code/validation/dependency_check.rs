//! Dependency validation for dead code detection

use std::collections::{HashMap, HashSet};

use crate::analysis::detectors::anti_patterns::dead_code::types::{Symbol, SymbolType};

/// Validates dependencies to ensure safe removal of dead code
pub struct DependencyValidator;

impl DependencyValidator {
    /// Check if a symbol is safe to remove
    pub fn is_safe_to_remove(symbol: &Symbol, all_symbols: &[Symbol]) -> bool {
        // Build dependency graph
        let dep_graph = Self::build_dependency_graph(all_symbols);

        // Check for dependencies
        !Self::has_critical_dependencies(symbol, &dep_graph)
            && !Self::is_interface_requirement(symbol)
            && !Self::is_framework_hook(symbol)
    }

    /// Build a dependency graph from symbols
    fn build_dependency_graph(symbols: &[Symbol]) -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        for symbol in symbols {
            graph.add_symbol(symbol.clone());

            // Find dependencies based on code analysis
            for other in symbols {
                if symbol.code_snippet.contains(&other.name) {
                    graph.add_dependency(symbol.name.clone(), other.name.clone());
                }
            }
        }

        graph
    }

    /// Check if symbol has critical dependencies
    fn has_critical_dependencies(symbol: &Symbol, dep_graph: &DependencyGraph) -> bool {
        dep_graph.has_dependents(&symbol.name)
    }

    /// Check if symbol is required by an interface
    fn is_interface_requirement(symbol: &Symbol) -> bool {
        // Check for interface patterns
        symbol.name.starts_with("impl_")
            || symbol.name.contains("_trait_")
            || symbol.symbol_type == SymbolType::Trait
    }

    /// Check if symbol is a framework hook
    fn is_framework_hook(symbol: &Symbol) -> bool {
        const FRAMEWORK_HOOKS: &[&str] = &[
            "componentDidMount",
            "componentWillUnmount",
            "useEffect",
            "onCreate",
            "onDestroy",
            "ngOnInit",
            "ngOnDestroy",
        ];

        FRAMEWORK_HOOKS.contains(&symbol.name.as_str())
    }

    /// Validate dependency chain
    pub fn validate_chain(
        symbols: Vec<&Symbol>,
        dep_graph: &DependencyGraph,
    ) -> DependencyValidation {
        let mut safe = Vec::new();
        let mut unsafe_deps = Vec::new();

        for symbol in symbols {
            if dep_graph.is_isolated(&symbol.name) {
                safe.push(symbol.clone());
            } else {
                unsafe_deps.push(symbol.clone());
            }
        }

        DependencyValidation { safe, unsafe_deps }
    }

    /// Check for circular dependencies
    pub fn has_circular_dependencies(symbol: &Symbol, dep_graph: &DependencyGraph) -> bool {
        dep_graph.has_cycle_from(&symbol.name)
    }
}

/// Dependency graph for tracking symbol relationships
pub struct DependencyGraph {
    symbols: HashMap<String, Symbol>,
    dependencies: HashMap<String, HashSet<String>>,
    dependents: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }

    pub fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies
            .entry(from.clone())
            .or_insert_with(HashSet::new)
            .insert(to.clone());

        self.dependents
            .entry(to)
            .or_insert_with(HashSet::new)
            .insert(from);
    }

    pub fn has_dependents(&self, symbol_name: &str) -> bool {
        self.dependents
            .get(symbol_name)
            .map_or(false, |deps| !deps.is_empty())
    }

    pub fn is_isolated(&self, symbol_name: &str) -> bool {
        !self.has_dependents(symbol_name)
            && self
                .dependencies
                .get(symbol_name)
                .map_or(true, |deps| deps.is_empty())
    }

    pub fn has_cycle_from(&self, start: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        self.detect_cycle(start, &mut visited, &mut stack)
    }

    fn detect_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> bool {
        if stack.contains(node) {
            return true;
        }

        if visited.contains(node) {
            return false;
        }

        visited.insert(node.to_string());
        stack.insert(node.to_string());

        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if self.detect_cycle(dep, visited, stack) {
                    return true;
                }
            }
        }

        stack.remove(node);
        false
    }
}

/// Result of dependency validation
pub struct DependencyValidation {
    pub safe: Vec<Symbol>,
    pub unsafe_deps: Vec<Symbol>,
}

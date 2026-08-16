//! Dependency graph construction and topological sort.
//!
//! Builds a directed acyclic graph (DAG) from plugin dependency declarations,
//! detects cycles, and produces a topological ordering for plugin loading.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::PluginError;
use crate::metadata::PluginMetadata;

/// Directed graph representing plugin dependencies.
///
/// Uses an adjacency list representation where each plugin maps to the
/// set of plugins it depends on.
pub struct DependencyGraph {
    /// Maps plugin name → set of dependency names.
    edges: HashMap<String, HashSet<String>>,
    /// All known plugin names.
    nodes: HashSet<String>,
}

impl DependencyGraph {
    /// Builds a dependency graph from a set of plugin metadata entries.
    ///
    /// Returns the graph and a list of error messages for missing dependencies.
    /// Plugins not present in the metadata set are treated as missing dependencies.
    pub fn build(plugins: &[PluginMetadata]) -> (Self, Vec<String>) {
        let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
        let nodes: HashSet<String> = plugins.iter().map(|p| p.name.clone()).collect();
        let mut errors = Vec::new();

        for plugin in plugins {
            let mut deps: HashSet<String> = HashSet::new();
            for dep in &plugin.dependencies {
                if nodes.contains(&dep.name) {
                    deps.insert(dep.name.clone());
                } else {
                    errors.push(format!(
                        "plugin '{}' depends on '{}' which is not available",
                        plugin.name, dep.name
                    ));
                }
            }
            edges.insert(plugin.name.clone(), deps);
        }

        (Self { edges, nodes }, errors)
    }

    /// Builds a dependency graph from a set of plugin metadata references.
    ///
    /// Convenience method for use with borrowed metadata slices.
    pub fn build_from_refs(plugins: &[&PluginMetadata]) -> Self {
        let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
        let nodes: HashSet<String> = plugins.iter().map(|p| p.name.clone()).collect();

        for plugin in plugins {
            let deps: HashSet<String> = plugin
                .dependencies
                .iter()
                .map(|d| d.name.clone())
                .filter(|name| nodes.contains(name))
                .collect();
            edges.insert(plugin.name.clone(), deps);
        }

        Self { edges, nodes }
    }

    /// Detects cycles in the dependency graph using DFS.
    ///
    /// Returns a list of plugin names involved in cycles, or an empty vec
    /// if the graph is acyclic.
    pub fn detect_cycles(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut cycle_members = HashSet::new();

        for node in &self.nodes {
            if !visited.contains(node) {
                self.dfs_cycle_detect(node, &mut visited, &mut in_stack, &mut cycle_members);
            }
        }

        cycle_members.into_iter().collect()
    }

    /// Performs topological sort using Kahn's algorithm.
    ///
    /// Returns the load order (dependencies before dependents) or an error
    /// if cycles are detected.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::CircularDependency` if the graph contains cycles.
    pub fn topological_sort(&self) -> Result<Vec<String>, PluginError> {
        // Compute in-degree for each node
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node in &self.nodes {
            in_degree.insert(node.as_str(), 0);
        }

        // in_degree is computed below when building the reverse adjacency list

        // Build the reverse graph: for each edge A depends on B, add B → A
        let mut reverse_adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in &self.nodes {
            reverse_adj.insert(node.as_str(), Vec::new());
        }

        for (node, deps) in &self.edges {
            for dep in deps {
                if self.nodes.contains(dep) {
                    reverse_adj
                        .get_mut(dep.as_str())
                        .unwrap()
                        .push(node.as_str());
                    *in_degree.get_mut(node.as_str()).unwrap() += 1;
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = VecDeque::new();
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node);
            }
        }

        // Sort the initial queue for deterministic ordering
        let mut sorted_queue: Vec<&str> = queue.drain(..).collect();
        sorted_queue.sort();
        queue.extend(sorted_queue);

        let mut result = Vec::new();
        let mut processed = 0;

        while let Some(node) = queue.pop_front() {
            result.push(node.to_string());
            processed += 1;

            let mut next_nodes: Vec<&str> = Vec::new();
            if let Some(dependents) = reverse_adj.get(node) {
                for &dependent in dependents {
                    let degree = in_degree.get_mut(dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        next_nodes.push(dependent);
                    }
                }
            }
            // Sort for deterministic ordering
            next_nodes.sort();
            queue.extend(next_nodes);
        }

        if processed != self.nodes.len() {
            // Cycle detected — find remaining nodes
            let cycle: Vec<String> = self
                .nodes
                .iter()
                .filter(|n| !result.contains(n))
                .cloned()
                .collect();
            return Err(PluginError::CircularDependency { cycle });
        }

        Ok(result)
    }

    /// Returns the set of plugins that directly depend on the given plugin.
    pub fn dependents_of(&self, plugin_name: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|(_, deps)| deps.contains(plugin_name))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Returns the set of plugins that the given plugin directly depends on.
    pub fn dependencies_of(&self, plugin_name: &str) -> Vec<String> {
        self.edges
            .get(plugin_name)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns whether the given plugin has any unresolved dependencies
    /// (dependencies declared but not present in the graph).
    pub fn has_missing_dependencies(
        &self,
        plugin_name: &str,
        all_metadata: &[&PluginMetadata],
    ) -> Vec<String> {
        let meta = all_metadata.iter().find(|m| m.name == plugin_name);
        match meta {
            Some(m) => m
                .dependencies
                .iter()
                .filter(|d| !self.nodes.contains(&d.name))
                .map(|d| d.name.clone())
                .collect(),
            None => vec![],
        }
    }

    /// DFS helper for cycle detection.
    fn dfs_cycle_detect(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        cycle_members: &mut HashSet<String>,
    ) {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());

        if let Some(deps) = self.edges.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_cycle_detect(dep, visited, in_stack, cycle_members);
                } else if in_stack.contains(dep) {
                    // Found a cycle — mark all nodes currently in the stack
                    cycle_members.insert(dep.clone());
                    cycle_members.insert(node.to_string());
                }
            }
        }

        in_stack.remove(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{PluginDependency, PluginMetadata};
    use crate::version::{Version, VersionReq};

    fn make_meta(name: &str, deps: &[&str]) -> PluginMetadata {
        PluginMetadata {
            name: name.to_string(),
            version: Version::new(1, 0, 0),
            author: "Test".to_string(),
            description: "".to_string(),
            dependencies: deps
                .iter()
                .map(|d| PluginDependency {
                    name: d.to_string(),
                    version_req: VersionReq::new(Version::new(1, 0, 0), true),
                })
                .collect(),
            required_api_version: Version::new(1, 0, 0),
        }
    }

    #[test]
    fn empty_graph_topological_sort() {
        // Validates: Requirement 3.3
        let (graph, _) = DependencyGraph::build(&[]);
        let order = graph.topological_sort().unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn single_plugin_no_deps() {
        // Validates: Requirement 3.3
        let meta = make_meta("alpha", &[]);
        let (graph, _) = DependencyGraph::build(&[meta]);
        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec!["alpha"]);
    }

    #[test]
    fn linear_dependency_chain() {
        // Validates: Requirement 3.3
        let a = make_meta("a", &[]);
        let b = make_meta("b", &["a"]);
        let c = make_meta("c", &["b"]);
        let (graph, _) = DependencyGraph::build(&[a, b, c]);
        let order = graph.topological_sort().unwrap();

        let pos_a = order.iter().position(|n| n == "a").unwrap();
        let pos_b = order.iter().position(|n| n == "b").unwrap();
        let pos_c = order.iter().position(|n| n == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn diamond_dependency() {
        // Validates: Requirement 3.3
        let a = make_meta("a", &[]);
        let b = make_meta("b", &["a"]);
        let c = make_meta("c", &["a"]);
        let d = make_meta("d", &["b", "c"]);
        let (graph, _) = DependencyGraph::build(&[a, b, c, d]);
        let order = graph.topological_sort().unwrap();

        let pos_a = order.iter().position(|n| n == "a").unwrap();
        let pos_b = order.iter().position(|n| n == "b").unwrap();
        let pos_c = order.iter().position(|n| n == "c").unwrap();
        let pos_d = order.iter().position(|n| n == "d").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn cycle_detection_returns_error() {
        // Validates: Requirement 3.4
        let a = make_meta("a", &["b"]);
        let b = make_meta("b", &["a"]);
        let (graph, _) = DependencyGraph::build(&[a, b]);
        let result = graph.topological_sort();
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::CircularDependency { cycle } => {
                assert!(cycle.contains(&"a".to_string()));
                assert!(cycle.contains(&"b".to_string()));
            }
            _ => panic!("expected CircularDependency"),
        }
    }

    #[test]
    fn three_node_cycle_detected() {
        // Validates: Requirement 3.4
        let a = make_meta("a", &["c"]);
        let b = make_meta("b", &["a"]);
        let c = make_meta("c", &["b"]);
        let (graph, _) = DependencyGraph::build(&[a, b, c]);
        let result = graph.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn cycle_does_not_affect_non_cyclic_nodes() {
        // Validates: Requirement 3.4
        let a = make_meta("a", &["b"]);
        let b = make_meta("b", &["a"]);
        let c = make_meta("c", &[]); // Not in the cycle
        let (graph, _) = DependencyGraph::build(&[a, b, c]);

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
        // "c" is not in the cycle
        assert!(!cycles.contains(&"c".to_string()));
    }

    #[test]
    fn missing_dependency_is_excluded_from_graph() {
        // Validates: Requirement 3.7
        let a = make_meta("a", &["nonexistent"]);
        let (graph, errors) = DependencyGraph::build(&[a]);
        // The missing dep is reported as an error
        assert!(!errors.is_empty());
        // But the graph still sorts fine
        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec!["a"]);
    }

    #[test]
    fn has_missing_dependencies_reports_correctly() {
        // Validates: Requirement 3.7
        let a = make_meta("a", &["nonexistent"]);
        let graph = DependencyGraph::build_from_refs(&[&a]);
        let missing = graph.has_missing_dependencies("a", &[&a]);
        assert_eq!(missing, vec!["nonexistent"]);
    }

    #[test]
    fn dependents_of_returns_correct_plugins() {
        // Validates: Requirement 5.5
        let a = make_meta("a", &[]);
        let b = make_meta("b", &["a"]);
        let c = make_meta("c", &["a"]);
        let (graph, _) = DependencyGraph::build(&[a, b, c]);
        let mut deps = graph.dependents_of("a");
        deps.sort();
        assert_eq!(deps, vec!["b", "c"]);
    }

    #[test]
    fn dependencies_of_returns_correct_set() {
        // Validates: Requirement 3.3
        let a = make_meta("a", &[]);
        let b = make_meta("b", &["a"]);
        let (graph, _) = DependencyGraph::build(&[a, b]);
        let deps = graph.dependencies_of("b");
        assert_eq!(deps, vec!["a".to_string()]);
    }
}

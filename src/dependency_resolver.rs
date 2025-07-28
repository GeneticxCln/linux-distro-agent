// Package Dependency Resolution System - v4.2.0
// Advanced dependency resolution with conflict detection and optimal installation ordering

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl PackageVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch, pre_release: None }
    }

    pub fn from_string(version: &str) -> Result<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid version format: {}", version));
        }

        Ok(Self {
            major: parts[0].parse()?,
            minor: parts[1].parse()?,
            patch: parts[2].parse()?,
            pre_release: None,
        })
    }

    pub fn satisfies(&self, requirement: &VersionRequirement) -> bool {
        match requirement.operator.as_str() {
            "=" => self == &requirement.version,
            ">=" => self >= &requirement.version,
            ">" => self > &requirement.version,
            "<=" => self <= &requirement.version,
            "<" => self < &requirement.version,
            "~" => self.major == requirement.version.major && self.minor == requirement.version.minor,
            "^" => self.major == requirement.version.major && self >= &requirement.version,
            _ => false,
        }
    }
}

impl PartialOrd for PackageVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PackageVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRequirement {
    pub operator: String,  // "=", ">=", ">", "<=", "<", "~", "^"
    pub version: PackageVersion,
}

impl VersionRequirement {
    pub fn new(operator: &str, version: PackageVersion) -> Self {
        Self {
            operator: operator.to_string(),
            version,
        }
    }

    pub fn from_string(requirement: &str) -> Result<Self> {
        let (operator, version_str) = if requirement.starts_with(">=") {
            (">=", &requirement[2..])
        } else if requirement.starts_with("<=") {
            ("<=", &requirement[2..])
        } else if requirement.starts_with(">") {
            (">", &requirement[1..])
        } else if requirement.starts_with("<") {
            ("<", &requirement[1..])
        } else if requirement.starts_with("~") {
            ("~", &requirement[1..])
        } else if requirement.starts_with("^") {
            ("^", &requirement[1..])
        } else if requirement.starts_with("=") {
            ("=", &requirement[1..])
        } else {
            ("=", requirement)
        };

        let version = PackageVersion::from_string(version_str.trim())?;
        Ok(Self::new(operator, version))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version_requirement: Option<VersionRequirement>,
    pub optional: bool,
    pub development: bool,
}

impl PackageDependency {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version_requirement: None,
            optional: false,
            development: false,
        }
    }

    pub fn with_version(name: &str, requirement: VersionRequirement) -> Self {
        Self {
            name: name.to_string(),
            version_requirement: Some(requirement),
            optional: false,
            development: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn dev_dependency(mut self) -> Self {
        self.development = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: PackageVersion,
    pub dependencies: Vec<PackageDependency>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub architecture: String,
    pub size: u64,
    pub installed: bool,
}

impl PackageInfo {
    pub fn new(name: &str, version: PackageVersion) -> Self {
        Self {
            name: name.to_string(),
            version,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            architecture: "x86_64".to_string(),
            size: 0,
            installed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub package1: String,
    pub package2: String,
    pub reason: ConflictReason,
}

#[derive(Debug, Clone)]
pub enum ConflictReason {
    ExplicitConflict,
    VersionIncompatibility,
    CircularDependency,
    UnsatisfiableDependency,
}

#[derive(Debug)]
pub struct ResolutionResult {
    pub install_order: Vec<String>,
    pub total_packages: usize,
    pub total_size: u64,
    pub conflicts: Vec<DependencyConflict>,
    pub warnings: Vec<String>,
}

pub struct DependencyResolver {
    package_database: HashMap<String, Vec<PackageInfo>>,
    installed_packages: HashSet<String>,
    conflict_cache: HashMap<String, HashSet<String>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            package_database: HashMap::new(),
            installed_packages: HashSet::new(),
            conflict_cache: HashMap::new(),
        }
    }

    /// Add package information to the database
    pub fn add_package(&mut self, package: PackageInfo) {
        self.package_database
            .entry(package.name.clone())
            .or_insert_with(Vec::new)
            .push(package);
    }

    /// Mark a package as installed
    pub fn mark_installed(&mut self, package_name: &str) {
        self.installed_packages.insert(package_name.to_string());
    }

    /// Resolve dependencies for a list of packages
    pub fn resolve(&self, packages: &[String], include_dev_deps: bool) -> Result<ResolutionResult> {
        let mut resolution_state = ResolutionState::new();
        let mut conflicts = Vec::new();
        let warnings = Vec::new();

        // Build dependency graph
        for package_name in packages {
            if let Err(conflict) = self.build_dependency_graph(
                package_name,
                &mut resolution_state,
                include_dev_deps,
                &mut HashSet::new(),
            ) {
                conflicts.push(conflict);
            }
        }

        // Check for conflicts
        self.detect_conflicts(&resolution_state, &mut conflicts);

        // Topological sort for installation order
        let install_order = self.topological_sort(&resolution_state)?;

        // Calculate total size
        let total_size = self.calculate_total_size(&install_order);

        // Filter out already installed packages
        let install_order: Vec<String> = install_order
            .into_iter()
            .filter(|pkg| !self.installed_packages.contains(pkg))
            .collect();

        Ok(ResolutionResult {
            total_packages: install_order.len(),
            install_order,
            total_size,
            conflicts,
            warnings,
        })
    }

    /// Build dependency graph recursively
    fn build_dependency_graph(
        &self,
        package_name: &str,
        state: &mut ResolutionState,
        include_dev_deps: bool,
        visited: &mut HashSet<String>,
    ) -> Result<(), DependencyConflict> {
        // Detect circular dependencies
        if visited.contains(package_name) {
            return Err(DependencyConflict {
                package1: package_name.to_string(),
                package2: package_name.to_string(),
                reason: ConflictReason::CircularDependency,
            });
        }

        visited.insert(package_name.to_string());

        // Find best version of the package
        let package = match self.find_best_version(package_name, None) {
            Some(pkg) => pkg,
            None => {
                return Err(DependencyConflict {
                    package1: package_name.to_string(),
                    package2: "".to_string(),
                    reason: ConflictReason::UnsatisfiableDependency,
                });
            }
        };

        // Add to resolution state
        state.add_package(package.clone());

        // Process dependencies
        for dep in &package.dependencies {
            if dep.development && !include_dev_deps {
                continue;
            }

            // Recursively resolve dependencies
            self.build_dependency_graph(&dep.name, state, include_dev_deps, visited)?;
            state.add_dependency(&package.name, &dep.name);
        }

        visited.remove(package_name);
        Ok(())
    }

    /// Find the best version of a package that satisfies requirements
    fn find_best_version(
        &self,
        package_name: &str,
        requirement: Option<&VersionRequirement>,
    ) -> Option<&PackageInfo> {
        let versions = self.package_database.get(package_name)?;

        versions
            .iter()
            .filter(|pkg| {
                if let Some(req) = requirement {
                    pkg.version.satisfies(req)
                } else {
                    true
                }
            })
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Detect conflicts between packages
    fn detect_conflicts(&self, state: &ResolutionState, conflicts: &mut Vec<DependencyConflict>) {
        for (pkg_name, package) in &state.packages {
            for conflict in &package.conflicts {
                if state.packages.contains_key(conflict) {
                    conflicts.push(DependencyConflict {
                        package1: pkg_name.clone(),
                        package2: conflict.clone(),
                        reason: ConflictReason::ExplicitConflict,
                    });
                }
            }
        }
    }

    /// Perform topological sort to determine installation order
    fn topological_sort(&self, state: &ResolutionState) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result: Vec<String> = Vec::new();

        // Initialize in-degree and graph
        for package_name in state.packages.keys() {
            in_degree.insert(package_name.clone(), 0);
            graph.insert(package_name.clone(), Vec::new());
        }

        // Build graph and calculate in-degrees
        for (package_name, deps) in &state.dependencies {
            for dep in deps {
                if let Some(dep_list) = graph.get_mut(dep) {
                    dep_list.push(package_name.clone());
                }
                if let Some(degree) = in_degree.get_mut(package_name) {
                    *degree += 1;
                }
            }
        }

        // Find packages with no dependencies
        for (package_name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(package_name.clone());
            }
        }

        // Process packages in topological order
        while let Some(package_name) = queue.pop_front() {
            result.push(package_name.clone());

            if let Some(dependents) = graph.get(&package_name) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        if result.len() != state.packages.len() {
            return Err(anyhow!("Circular dependency detected in package graph"));
        }

        Ok(result)
    }

    /// Calculate total download/installation size
    fn calculate_total_size(&self, packages: &[String]) -> u64 {
        packages
            .iter()
            .filter_map(|pkg_name| {
                self.package_database
                    .get(pkg_name)?
                    .first()
                    .map(|pkg| pkg.size)
            })
            .sum()
    }

    /// Load package database from package manager
    pub async fn load_package_database(&mut self, package_manager: &str) -> Result<()> {
        match package_manager {
            "apt" => self.load_apt_database().await,
            "dnf" | "yum" => self.load_dnf_database().await,
            "pacman" => self.load_pacman_database().await,
            "zypper" => self.load_zypper_database().await,
            _ => Err(anyhow!("Unsupported package manager: {}", package_manager)),
        }
    }

    async fn load_apt_database(&mut self) -> Result<()> {
        // Implementation for loading APT package database
        // This would parse `apt-cache dump` or use libapt
        Ok(())
    }

    async fn load_dnf_database(&mut self) -> Result<()> {
        // Implementation for loading DNF package database
        Ok(())
    }

    async fn load_pacman_database(&mut self) -> Result<()> {
        // Implementation for loading Pacman package database
        Ok(())
    }

    async fn load_zypper_database(&mut self) -> Result<()> {
        // Implementation for loading Zypper package database
        Ok(())
    }
}

/// Internal state for dependency resolution
struct ResolutionState {
    packages: HashMap<String, PackageInfo>,
    dependencies: HashMap<String, Vec<String>>,
}

impl ResolutionState {
    fn new() -> Self {
        Self {
            packages: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    fn add_package(&mut self, package: PackageInfo) {
        self.packages.insert(package.name.clone(), package);
    }

    fn add_dependency(&mut self, package: &str, dependency: &str) {
        self.dependencies
            .entry(package.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let v1 = PackageVersion::new(1, 2, 3);
        let v2 = PackageVersion::new(1, 2, 4);
        let v3 = PackageVersion::new(1, 3, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_version_requirement() {
        let version = PackageVersion::new(1, 2, 3);
        let req_exact = VersionRequirement::new("=", PackageVersion::new(1, 2, 3));
        let req_gte = VersionRequirement::new(">=", PackageVersion::new(1, 2, 0));
        let req_lt = VersionRequirement::new("<", PackageVersion::new(2, 0, 0));

        assert!(version.satisfies(&req_exact));
        assert!(version.satisfies(&req_gte));
        assert!(version.satisfies(&req_lt));
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut resolver = DependencyResolver::new();

        // Package A depends on B
        let mut pkg_a = PackageInfo::new("package-a", PackageVersion::new(1, 0, 0));
        pkg_a.dependencies.push(PackageDependency::new("package-b"));

        let pkg_b = PackageInfo::new("package-b", PackageVersion::new(1, 0, 0));

        resolver.add_package(pkg_a);
        resolver.add_package(pkg_b);

        let result = resolver.resolve(&["package-a".to_string()], false).unwrap();

        assert_eq!(result.install_order.len(), 2);
        assert_eq!(result.install_order[0], "package-b");
        assert_eq!(result.install_order[1], "package-a");
    }
}

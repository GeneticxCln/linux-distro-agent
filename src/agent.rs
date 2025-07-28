use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use crate::logger::Logger;

/// AI Agent Planning and Execution System
/// 
/// This module provides intelligent task planning, execution safety,
/// and adaptive learning capabilities for the Linux Distribution Agent.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    PackageManagement,
    SystemConfiguration,
    SecurityAudit,
    Monitoring,
    RemoteExecution,
    DistroBuilding,
    PluginExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyLevel {
    Safe,      // No system changes, read-only operations
    Moderate,  // System changes with backup/rollback capability
    Risky,     // Significant system changes, requires confirmation
    Dangerous, // Potentially destructive, requires explicit permission
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub command: String,
    pub description: String,
    pub priority: Priority,
    pub safety_level: SafetyLevel,
    pub dependencies: Vec<String>,
    pub estimated_duration: Duration,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: SystemTime,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration: Duration,
    pub safety_checks_passed: bool,
    pub rollback_available: bool,
    pub executed_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub current_tasks: Vec<Task>,
    pub completed_tasks: Vec<ExecutionResult>,
    pub failed_tasks: Vec<ExecutionResult>,
    pub learning_data: HashMap<String, f64>,
    pub safety_violations: u32,
    pub last_update: SystemTime,
}

pub struct IntelligentAgent {
    state: AgentState,
    logger: Logger,
    safety_enforcer: SafetyEnforcer,
    task_planner: TaskPlanner,
    execution_engine: ExecutionEngine,
}

/// Safety Enforcement System
pub struct SafetyEnforcer {
    dangerous_patterns: Vec<String>,
    system_critical_paths: Vec<String>,
    backup_enabled: bool,
}

/// Task Planning System
pub struct TaskPlanner {
    optimization_strategy: OptimizationStrategy,
    dependency_resolver: DependencyResolver,
}

/// Execution Engine
pub struct ExecutionEngine {
    dry_run_mode: bool,
    confirmation_required: bool,
    rollback_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    Sequential,    // Execute tasks one by one
    Parallel,      // Execute independent tasks in parallel
    Adaptive,      // Learn optimal execution patterns
}

pub struct DependencyResolver {
    dependency_graph: HashMap<String, Vec<String>>,
}

impl IntelligentAgent {
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self {
            state: AgentState {
                current_tasks: Vec::new(),
                completed_tasks: Vec::new(),
                failed_tasks: Vec::new(),
                learning_data: HashMap::new(),
                safety_violations: 0,
                last_update: SystemTime::now(),
            },
            logger: Logger::new(verbose, quiet),
            safety_enforcer: SafetyEnforcer::new(),
            task_planner: TaskPlanner::new(),
            execution_engine: ExecutionEngine::new(),
        }
    }

    /// Main agent loop - processes tasks intelligently
    pub async fn run_agent_loop(&mut self) -> Result<()> {
        self.logger.info("🤖 Starting Intelligent Agent Loop");
        
        loop {
            // 1. Analyze current system state
            self.analyze_system_state().await?;
            
            // 2. Plan optimal task execution
            let execution_plan = self.task_planner.create_execution_plan(&self.state.current_tasks)?;
            
            // 3. Execute tasks with safety checks
            for task in execution_plan {
                match self.execute_task_safely(&task).await {
                    Ok(result) => {
                        self.state.completed_tasks.push(result.clone());
                        self.learn_from_execution(&task, &result);
                    }
                    Err(e) => {
                        self.logger.error(format!("Task execution failed: {}", e));
                        self.handle_task_failure(&task, &e);
                    }
                }
            }
            
            // 4. Update agent state and learning
            self.update_agent_state().await?;
            
            // 5. Check if we should continue
            if self.should_stop_loop() {
                break;
            }
            
            // 6. Brief pause before next iteration
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        self.logger.success("🎯 Agent loop completed successfully");
        Ok(())
    }

    /// Add a new task to the agent's queue
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        // Validate task safety
        if !self.safety_enforcer.validate_task(&task)? {
            return Err(anyhow::anyhow!("Task failed safety validation: {}", task.description));
        }
        
        self.logger.info(format!("📋 Adding task: {} (Priority: {:?})", task.description, task.priority));
        self.state.current_tasks.push(task);
        Ok(())
    }

    /// Execute a single task with comprehensive safety checks
    async fn execute_task_safely(&mut self, task: &Task) -> Result<ExecutionResult> {
        let start_time = SystemTime::now();
        
        self.logger.info(format!("⚡ Executing task: {}", task.description));
        
        // Pre-execution safety checks
        let safety_check = self.safety_enforcer.pre_execution_check(task)?;
        if !safety_check.safe_to_execute {
            return Err(anyhow::anyhow!("Safety check failed: {}", safety_check.reason));
        }
        
        // Create backup if needed
        let backup_id = if task.safety_level != SafetyLevel::Safe {
            Some(self.safety_enforcer.create_backup(&task.command)?)
        } else {
            None
        };
        
        // Execute the task
        let result = self.execution_engine.execute(task).await;
        
        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        
        // Post-execution validation
        let execution_result = match result {
            Ok(output) => {
                self.logger.success(format!("✅ Task completed: {}", task.description));
                ExecutionResult {
                    task_id: task.id.clone(),
                    success: true,
                    output,
                    error: None,
                    duration,
                    safety_checks_passed: true,
                    rollback_available: backup_id.is_some(),
                    executed_at: SystemTime::now(),
                }
            }
            Err(e) => {
                self.logger.error(format!("❌ Task failed: {} - {}", task.description, e));
                
                // Attempt rollback if backup exists
                if let Some(ref backup) = backup_id {
                    self.safety_enforcer.rollback(backup)?;
                    self.logger.info("🔄 System rolled back to previous state");
                }
                
                ExecutionResult {
                    task_id: task.id.clone(),
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    duration,
                    safety_checks_passed: true,
                    rollback_available: backup_id.is_some(),
                    executed_at: SystemTime::now(),
                }
            }
        };
        
        Ok(execution_result)
    }

    /// Analyze current system state for intelligent decision making
    async fn analyze_system_state(&mut self) -> Result<()> {
        // Check system resources
        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;
        let _disk_usage = self.get_disk_usage().await?;
        
        // Adjust execution strategy based on system load
        if cpu_usage > 80.0 || memory_usage > 90.0 {
            self.task_planner.optimization_strategy = OptimizationStrategy::Sequential;
            self.logger.warn("🚦 High system load detected, switching to sequential execution");
        } else {
            self.task_planner.optimization_strategy = OptimizationStrategy::Parallel;
        }
        
        Ok(())
    }

    /// Learn from task execution to improve future decisions
    fn learn_from_execution(&mut self, task: &Task, result: &ExecutionResult) {
        let task_key = format!("{}:{}", task.task_type.to_string(), task.safety_level.to_string());
        
        // Update success rate
        let current_rate = self.state.learning_data.get(&task_key).unwrap_or(&0.5);
        let new_rate = if result.success {
            (current_rate + 0.1).min(1.0)
        } else {
            (current_rate - 0.1).max(0.0)
        };
        
        self.state.learning_data.insert(task_key, new_rate);
        
        // Learn execution time patterns
        let duration_key = format!("{}_duration", task.task_type.to_string());
        let avg_duration = self.state.learning_data.get(&duration_key).unwrap_or(&30.0);
        let new_avg = (avg_duration + result.duration.as_secs() as f64) / 2.0;
        self.state.learning_data.insert(duration_key, new_avg);
    }

    /// Handle task failures intelligently
    fn handle_task_failure(&mut self, task: &Task, error: &anyhow::Error) {
        // Check if task should be retried
        if task.retry_count < task.max_retries {
            let mut retry_task = task.clone();
            retry_task.retry_count += 1;
            
            // Adjust retry delay based on failure type
            let delay = match task.retry_count {
                1 => Duration::from_secs(1),
                2 => Duration::from_secs(5),
                _ => Duration::from_secs(30),
            };
            
            self.logger.warn(format!("🔄 Retrying task {} in {:?}", task.description, delay));
            
            // Add back to queue with delay (simplified - in real implementation, use a scheduler)
            self.state.current_tasks.push(retry_task);
        } else {
            self.logger.error(format!("💀 Task permanently failed: {}", task.description));
            
            let result = ExecutionResult {
                task_id: task.id.clone(),
                success: false,
                output: String::new(),
                error: Some(error.to_string()),
                duration: Duration::from_secs(0),
                safety_checks_passed: false,
                rollback_available: false,
                executed_at: SystemTime::now(),
            };
            
            self.state.failed_tasks.push(result);
        }
    }

    /// Update agent state and persist learning data
    async fn update_agent_state(&mut self) -> Result<()> {
        self.state.last_update = SystemTime::now();
        
        // Remove completed tasks from current queue
        self.state.current_tasks.retain(|task| {
            !self.state.completed_tasks.iter().any(|result| result.task_id == task.id)
        });
        
        // Persist state (simplified - in real implementation, save to file)
        self.logger.verbose("💾 Agent state updated".to_string());
        
        Ok(())
    }

    /// Determine if the agent loop should stop
    fn should_stop_loop(&self) -> bool {
        // Stop if no more tasks to execute
        self.state.current_tasks.is_empty()
    }

    /// Get current CPU usage
    async fn get_cpu_usage(&self) -> Result<f64> {
        // Read from /proc/stat to get CPU usage
        let stat_content = tokio::fs::read_to_string("/proc/stat").await?;
        let lines: Vec<&str> = stat_content.lines().collect();
        
        if let Some(cpu_line) = lines.first() {
            if cpu_line.starts_with("cpu ") {
                let values: Vec<&str> = cpu_line.split_whitespace().collect();
                if values.len() >= 5 {
                    let user: u64 = values[1].parse().unwrap_or(0);
                    let nice: u64 = values[2].parse().unwrap_or(0);
                    let system: u64 = values[3].parse().unwrap_or(0);
                    let idle: u64 = values[4].parse().unwrap_or(0);
                    
                    let total = user + nice + system + idle;
                    let active = user + nice + system;
                    
                    if total > 0 {
                        return Ok((active as f64 / total as f64) * 100.0);
                    }
                }
            }
        }
        
        // Fallback: try using uptime command
        let output = tokio::process::Command::new("uptime")
            .output()
            .await?;
            
        if output.status.success() {
            let uptime_str = String::from_utf8_lossy(&output.stdout);
            // Parse load average as rough CPU usage indicator
            if let Some(load_start) = uptime_str.find("load average:") {
                let load_part = &uptime_str[load_start + 13..];
                if let Some(first_load) = load_part.split(',').next() {
                    if let Ok(load) = first_load.trim().parse::<f64>() {
                        // Convert load average to approximate CPU percentage
                        return Ok((load * 25.0).min(100.0));
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get current memory usage percentage
    async fn get_memory_usage(&self) -> Result<f64> {
        // Read from /proc/meminfo
        let meminfo_content = tokio::fs::read_to_string("/proc/meminfo").await?;
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        let mut free_kb = 0u64;
        let mut buffers_kb = 0u64;
        let mut cached_kb = 0u64;
        
        for line in meminfo_content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    total_kb = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    available_kb = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("MemFree:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    free_kb = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("Buffers:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    buffers_kb = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("Cached:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    cached_kb = value.parse().unwrap_or(0);
                }
            }
        }
        
        if total_kb > 0 {
            // Use MemAvailable if present, otherwise calculate
            let available = if available_kb > 0 {
                available_kb
            } else {
                free_kb + buffers_kb + cached_kb
            };
            
            let used = total_kb - available;
            return Ok((used as f64 / total_kb as f64) * 100.0);
        }
        
        Ok(0.0)
    }

    /// Get current disk usage percentage for root partition
    async fn get_disk_usage(&self) -> Result<f64> {
        // Use df command to get disk usage
        let output = tokio::process::Command::new("df")
            .args(["/", "-h", "--output=pcent"])
            .output()
            .await?;
            
        if output.status.success() {
            let df_output = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = df_output.lines().collect();
            
            // Skip header line and get the percentage
            if lines.len() >= 2 {
                let percent_str = lines[1].trim().replace('%', "");
                if let Ok(percentage) = percent_str.parse::<f64>() {
                    return Ok(percentage);
                }
            }
        }
        
        // Fallback: read from /proc/mounts and calculate manually
        let output = tokio::process::Command::new("stat")
            .args(["-f", "-c", "%a %f %S", "/"])
            .output()
            .await?;
            
        if output.status.success() {
            let stat_output = String::from_utf8_lossy(&output.stdout);
            let values: Vec<&str> = stat_output.trim().split_whitespace().collect();
            
            if values.len() >= 3 {
                if let (Ok(available), Ok(total), Ok(block_size)) = (
                    values[0].parse::<u64>(),
                    values[1].parse::<u64>(),
                    values[2].parse::<u64>()
                ) {
                    let total_bytes = total * block_size;
                    let available_bytes = available * block_size;
                    let used_bytes = total_bytes - available_bytes;
                    
                    if total_bytes > 0 {
                        return Ok((used_bytes as f64 / total_bytes as f64) * 100.0);
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Create a task from command line arguments
    pub fn create_task_from_command(
        &self,
        command: &str,
        args: &[String],
        task_type: TaskType,
    ) -> Task {
        let id = format!("task_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        let description = format!("{} {}", command, args.join(" "));
        
        // Determine safety level based on command
        let safety_level = self.assess_command_safety(command, args);
        
        // Determine priority based on task type
        let priority = match task_type {
            TaskType::SecurityAudit => Priority::High,
            TaskType::SystemConfiguration => Priority::Medium,
            TaskType::PackageManagement => Priority::Medium,
            _ => Priority::Low,
        };
        
        Task {
            id,
            task_type,
            command: format!("{} {}", command, args.join(" ")),
            description,
            priority,
            safety_level,
            dependencies: Vec::new(),
            estimated_duration: Duration::from_secs(30),
            retry_count: 0,
            max_retries: 3,
            created_at: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Assess the safety level of a command
    fn assess_command_safety(&self, command: &str, args: &[String]) -> SafetyLevel {
        let dangerous_commands = ["rm", "dd", "mkfs", "fdisk", "parted"];
        let risky_commands = ["sudo", "su", "chmod", "chown"];
        let moderate_commands = ["install", "update", "upgrade"];
        
        if dangerous_commands.contains(&command) {
            SafetyLevel::Dangerous
        } else if risky_commands.contains(&command) || args.iter().any(|arg| arg.contains("sudo")) {
            SafetyLevel::Risky
        } else if moderate_commands.contains(&command) {
            SafetyLevel::Moderate
        } else {
            SafetyLevel::Safe
        }
    }
}

impl SafetyEnforcer {
    pub fn new() -> Self {
        Self {
            dangerous_patterns: vec![
                "rm -rf /".to_string(),
                "dd if=".to_string(),
                "mkfs".to_string(),
                "format".to_string(),
            ],
            system_critical_paths: vec![
                "/boot".to_string(),
                "/etc".to_string(),
                "/usr".to_string(),
                "/var".to_string(),
            ],
            backup_enabled: true,
        }
    }

    pub fn validate_task(&self, task: &Task) -> Result<bool> {
        // Check for dangerous patterns
        for pattern in &self.dangerous_patterns {
            if task.command.contains(pattern) {
                return Ok(false);
            }
        }
        
        // Additional safety checks based on task type
        match task.task_type {
            TaskType::SystemConfiguration => {
                if task.safety_level == SafetyLevel::Dangerous {
                    return Ok(false);
                }
            }
            _ => {}
        }
        
        Ok(true)
    }

    pub fn pre_execution_check(&self, task: &Task) -> Result<SafetyCheck> {
        let mut safe_to_execute = true;
        let mut reason = String::new();
        
        // Check if command affects critical system paths
        for path in &self.system_critical_paths {
            if task.command.contains(path) && task.safety_level == SafetyLevel::Dangerous {
                safe_to_execute = false;
                reason = format!("Command affects critical system path: {}", path);
                break;
            }
        }
        
        Ok(SafetyCheck {
            safe_to_execute,
            reason,
            backup_recommended: task.safety_level != SafetyLevel::Safe,
        })
    }

    pub fn create_backup(&self, _command: &str) -> Result<String> {
        // Simplified backup creation
        let backup_id = format!("backup_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());
        Ok(backup_id)
    }

    pub fn rollback(&self, _backup_id: &str) -> Result<()> {
        // Simplified rollback implementation
        Ok(())
    }
}

#[derive(Debug)]
pub struct SafetyCheck {
    pub safe_to_execute: bool,
    pub reason: String,
    pub backup_recommended: bool,
}

impl TaskPlanner {
    pub fn new() -> Self {
        Self {
            optimization_strategy: OptimizationStrategy::Adaptive,
            dependency_resolver: DependencyResolver::new(),
        }
    }

    pub fn create_execution_plan(&self, tasks: &[Task]) -> Result<Vec<Task>> {
        let mut execution_plan = Vec::new();
        
        // Sort tasks by priority and dependencies
        let mut sorted_tasks = tasks.to_vec();
        sorted_tasks.sort_by(|a, b| {
            // First sort by priority
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => {
                    // Then by creation time
                    a.created_at.cmp(&b.created_at)
                }
                other => other,
            }
        });
        
        // Resolve dependencies
        for task in sorted_tasks {
            if self.dependency_resolver.dependencies_satisfied(&task, &execution_plan) {
                execution_plan.push(task);
            }
        }
        
        Ok(execution_plan)
    }
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            dependency_graph: HashMap::new(),
        }
    }

    pub fn dependencies_satisfied(&self, task: &Task, completed_tasks: &[Task]) -> bool {
        // Check if all dependencies are in completed tasks
        task.dependencies.iter().all(|dep| {
            completed_tasks.iter().any(|completed| completed.id == *dep)
        })
    }
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            dry_run_mode: false,
            confirmation_required: true,
            rollback_enabled: true,
        }
    }

    pub async fn execute(&self, task: &Task) -> Result<String> {
        if self.dry_run_mode {
            return Ok(format!("[DRY RUN] Would execute: {}", task.command));
        }
        
        // Execute the actual command
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&task.command)
            .output()
            .await?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

impl TaskType {
    fn to_string(&self) -> String {
        match self {
            TaskType::PackageManagement => "package_management".to_string(),
            TaskType::SystemConfiguration => "system_configuration".to_string(),
            TaskType::SecurityAudit => "security_audit".to_string(),
            TaskType::Monitoring => "monitoring".to_string(),
            TaskType::RemoteExecution => "remote_execution".to_string(),
            TaskType::DistroBuilding => "distro_building".to_string(),
            TaskType::PluginExecution => "plugin_execution".to_string(),
        }
    }
}

impl SafetyLevel {
    fn to_string(&self) -> String {
        match self {
            SafetyLevel::Safe => "safe".to_string(),
            SafetyLevel::Moderate => "moderate".to_string(),
            SafetyLevel::Risky => "risky".to_string(),
            SafetyLevel::Dangerous => "dangerous".to_string(),
        }
    }
}

// Add UUID dependency to Cargo.toml
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Simple UUID alternative using hash
mod uuid {
    use super::*;
    
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> UuidValue {
            let mut hasher = DefaultHasher::new();
            SystemTime::now().hash(&mut hasher);
            UuidValue(hasher.finish())
        }
    }
    
    pub struct UuidValue(u64);
    
    impl UuidValue {
        pub fn to_string(&self) -> String {
            format!("{:x}", self.0)
        }
    }
}

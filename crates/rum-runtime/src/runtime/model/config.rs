use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    pub recursion_limit: usize,
    /// Maximum native-stack bytes evaluator descent may consume from its request base.
    ///
    /// The default is calibrated for a caller-declared stack of at least
    /// [`MIN_SUPPORTED_NATIVE_STACK_BYTES`]. Larger declared worker stacks can
    /// derive a matching budget with [`native_stack_limit_for_declared_stack`].
    pub native_stack_limit: usize,
    pub replace_repeated_limit: usize,
    pub evaluation_pass_limit: usize,
    /// Optional per-request ceiling for modeled memory proxy bytes, not process RSS.
    ///
    /// Nested `MemoryConstrained` frames min-combine with this ceiling. A breach
    /// inside such a frame is handled in-band by that frame (`$Aborted` or its
    /// fallback), and the request ceiling remains armed outside the frame.
    pub eval_memory_limit: Option<usize>,
    pub generated_element_limit: usize,
    pub integer_work_limit: usize,
    pub iteration_work_limit: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            recursion_limit: DEFAULT_RECURSION_LIMIT,
            native_stack_limit: DEFAULT_NATIVE_STACK_LIMIT,
            replace_repeated_limit: DEFAULT_REPLACE_REPEATED_LIMIT,
            evaluation_pass_limit: DEFAULT_EVALUATION_PASS_LIMIT,
            eval_memory_limit: None,
            generated_element_limit: DEFAULT_GENERATED_ELEMENT_LIMIT,
            integer_work_limit: DEFAULT_INTEGER_WORK_LIMIT,
            iteration_work_limit: DEFAULT_ITERATION_WORK_LIMIT,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionConfig {
    pub resource_limits: ResourceLimits,
    pub capability_policy: CapabilityPolicy,
    pub current_context: String,
    pub context_path: Vec<String>,
    pub package_search_paths: Vec<PathBuf>,
    pub tracing_enabled: bool,
    pub asserts_enabled: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            capability_policy: CapabilityPolicy::unrestricted(),
            current_context: GLOBAL_CONTEXT.to_string(),
            context_path: vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()],
            package_search_paths: vec![PathBuf::from(".")],
            tracing_enabled: false,
            asserts_enabled: false,
        }
    }
}

impl SessionConfig {
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    pub fn with_capability_policy(mut self, policy: CapabilityPolicy) -> Self {
        self.capability_policy = policy;
        self
    }

    pub fn with_current_context(mut self, context: impl Into<String>) -> Self {
        self.current_context = context.into();
        self
    }

    pub fn with_context_path<I, S>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.context_path = contexts.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_package_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.package_search_paths = paths.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }

    pub fn with_asserts_enabled(mut self, enabled: bool) -> Self {
        self.asserts_enabled = enabled;
        self
    }

    pub fn normalized(mut self) -> Self {
        self.current_context = normalize_context_name(self.current_context);

        self.context_path = if self.context_path.is_empty() {
            vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()]
        } else {
            let mut normalized = Vec::new();

            for context in self.context_path {
                let context = normalize_context_name(context);

                if !normalized.iter().any(|existing| existing == &context) {
                    normalized.push(context);
                }
            }

            normalized
        };

        self.package_search_paths = if self.package_search_paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            dedup_paths(self.package_search_paths)
        };

        self
    }
}

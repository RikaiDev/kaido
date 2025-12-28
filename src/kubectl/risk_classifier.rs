use serde::{Deserialize, Serialize};

/// Risk level for kubectl commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only operations (get, describe, logs, top)
    Low,
    /// State-modifying operations (apply, scale, create, patch)
    Medium,
    /// Destructive operations (delete, drain, scale to 0)
    High,
}

impl RiskLevel {
    /// Convert to string for storage/display
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
        }
    }

    /// Classify kubectl command by risk level
    ///
    /// Classification rules from research.md:
    /// - HIGH: delete, drain, scale --replicas=0
    /// - MEDIUM: apply, create, patch, edit, scale (non-zero), rollout
    /// - LOW: get, describe, logs, top, explain, api-resources, auth
    pub fn classify(command: &str) -> Self {
        // Normalize command for matching
        let cmd_lower = command.to_lowercase();

        // HIGH risk: Destructive operations
        if cmd_lower.contains("delete") || cmd_lower.contains("drain") {
            return RiskLevel::High;
        }

        // Special case: scale to 0 replicas is effectively a delete
        if cmd_lower.contains("scale")
            && (cmd_lower.contains("--replicas=0") || cmd_lower.contains("--replicas 0"))
        {
            return RiskLevel::High;
        }

        // MEDIUM risk: State-modifying operations
        if cmd_lower.contains("apply")
            || cmd_lower.contains("create")
            || cmd_lower.contains("patch")
            || cmd_lower.contains("edit")
            || cmd_lower.contains("scale")
            || cmd_lower.contains("rollout")
            || cmd_lower.contains("restart")
            || cmd_lower.contains("label")
            || cmd_lower.contains("annotate")
        {
            return RiskLevel::Medium;
        }

        // LOW risk: Read-only operations (default)
        RiskLevel::Low
    }

    /// Check if this risk level requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        match self {
            RiskLevel::Low => false,
            RiskLevel::Medium | RiskLevel::High => true,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_risk_classification() {
        assert_eq!(
            RiskLevel::classify("kubectl delete deployment nginx"),
            RiskLevel::High
        );
        assert_eq!(
            RiskLevel::classify("kubectl delete pod test-pod -n production"),
            RiskLevel::High
        );
        assert_eq!(
            RiskLevel::classify("kubectl drain node-01"),
            RiskLevel::High
        );
        assert_eq!(
            RiskLevel::classify("kubectl scale deployment nginx --replicas=0"),
            RiskLevel::High
        );
    }

    #[test]
    fn test_medium_risk_classification() {
        assert_eq!(
            RiskLevel::classify("kubectl apply -f deployment.yaml"),
            RiskLevel::Medium
        );
        assert_eq!(
            RiskLevel::classify("kubectl scale deployment nginx --replicas=3"),
            RiskLevel::Medium
        );
        assert_eq!(
            RiskLevel::classify("kubectl create configmap my-config"),
            RiskLevel::Medium
        );
        assert_eq!(
            RiskLevel::classify("kubectl rollout restart deployment/nginx"),
            RiskLevel::Medium
        );
    }

    #[test]
    fn test_low_risk_classification() {
        assert_eq!(RiskLevel::classify("kubectl get pods"), RiskLevel::Low);
        assert_eq!(
            RiskLevel::classify("kubectl describe service nginx"),
            RiskLevel::Low
        );
        assert_eq!(
            RiskLevel::classify("kubectl logs nginx-pod"),
            RiskLevel::Low
        );
        assert_eq!(RiskLevel::classify("kubectl top nodes"), RiskLevel::Low);
    }

    #[test]
    fn test_string_conversion() {
        assert_eq!(RiskLevel::Low.as_str(), "LOW");
        assert_eq!(RiskLevel::Medium.as_str(), "MEDIUM");
        assert_eq!(RiskLevel::High.as_str(), "HIGH");
    }

    #[test]
    fn test_requires_confirmation() {
        assert!(!RiskLevel::Low.requires_confirmation());
        assert!(RiskLevel::Medium.requires_confirmation());
        assert!(RiskLevel::High.requires_confirmation());
    }
}

# Quickstart Guide: Kubectl-Only MVP

**Feature**: Kubectl-Only MVP (60-Day Reality Check)
**Target Audience**: Beta testers and early adopters
**Time to First Command**: 5 minutes

---

## Prerequisites

Before you begin, ensure you have:

- ✅ **Rust toolchain** (1.75+): `rustc --version`
- ✅ **kubectl installed**: `kubectl version --client`
- ✅ **Active kubeconfig**: `kubectl config current-context`
- ✅ **OpenAI API key**: Get from https://platform.openai.com/api-keys

---

## Installation

### Step 1: Clone and Build

```bash
# Clone repository
git clone https://github.com/your-org/kaido-ai.git
cd kaido-ai

# Checkout kubectl-only MVP branch
git checkout 007-kubectl-mvp-refactor

# Build release binary
cargo build --release

# Verify build succeeded
./target/release/kaido --version
```

**Expected Output**:
```
kaido 0.2.0 - Kubectl Natural Language Interface
```

### Step 2: Configure OpenAI API Key

Create configuration file at `~/.kaido/config.toml`:

```bash
mkdir -p ~/.kaido
cat > ~/.kaido/config.toml <<EOF
# Kaido AI Shell Configuration (Kubectl MVP)

[ai]
provider = "openai"
api_key = "sk-YOUR_OPENAI_API_KEY_HERE"
model = "gpt-4-turbo-preview"
timeout_seconds = 10

[audit]
database_path = "~/.kaido/audit.db"
retention_days = 90

[safety]
# Risk confirmation settings
require_typed_confirmation_in_production = true
EOF
```

**Replace** `sk-YOUR_OPENAI_API_KEY_HERE` with your actual OpenAI API key.

**Set file permissions** (protect API key):
```bash
chmod 600 ~/.kaido/config.toml
```

### Step 3: Verify kubectl Context

```bash
# List available contexts
kubectl config get-contexts

# Set your preferred context (example: development cluster)
kubectl config use-context dev-cluster
```

**Note**: Kaido detects environment type from context name:
- Names containing `prod` or `production` → HIGH risk confirmation required
- Names containing `stag` or `staging` → MEDIUM risk confirmation
- Names containing `dev` or `development` → LOW risk confirmation
- Other names → Treated as staging (MEDIUM risk)

---

## First Command: Show Pods

### Launch Kaido Shell

```bash
./target/release/kaido
```

**Expected Output**:
```
Kaido AI Shell - Kubectl Natural Language Interface
Current context: dev-cluster (development)
Namespace: default

Type your command or 'help' for assistance.
kaido>
```

### Execute Natural Language Command

```
kaido> show all pods
```

**What Happens**:
1. Kaido sends your input to OpenAI GPT-4
2. AI translates to: `kubectl get pods -n default`
3. Command is classified as **LOW risk** (read-only)
4. Command executes immediately (no confirmation needed)
5. Output is displayed in terminal

**Expected Output**:
```
Translating to kubectl...
Command: kubectl get pods -n default
Risk: LOW | Confidence: 95%

NAME                      READY   STATUS    RESTARTS   AGE
nginx-deployment-abc123   1/1     Running   0          2d
redis-master-xyz789       1/1     Running   0          5d

Logged to audit database.
kaido>
```

---

## Try More Commands

### Read Operations (LOW Risk - No Confirmation)

```
kaido> list deployments
kaido> show logs for nginx-deployment-abc123
kaido> describe service redis-master
kaido> get nodes
kaido> show current namespace
```

### Modify Operations (MEDIUM Risk - Yes/No Confirmation)

```
kaido> scale nginx deployment to 5 replicas
```

**Expected Prompt**:
```
Translating to kubectl...
Command: kubectl scale deployment nginx --replicas=5 -n default
Risk: MEDIUM | Confidence: 90%

⚠️  This command will modify cluster state.
Cluster: dev-cluster
Namespace: default

Proceed? (yes/no): 
```

Type `yes` and press Enter to execute.

### Destructive Operations (HIGH Risk - Typed Confirmation)

**In Development Context** (yes/no confirmation):
```
kaido> delete deployment nginx
```

**Expected Prompt**:
```
Translating to kubectl...
Command: kubectl delete deployment nginx -n default
Risk: HIGH | Confidence: 95%

⚠️  HIGH RISK: Destructive operation
Cluster: dev-cluster
Namespace: default

Proceed? (yes/no): 
```

**In Production Context** (typed confirmation):
```
# First, switch to production context
kaido> /context prod-cluster

kaido> delete deployment nginx
```

**Expected Prompt**:
```
Translating to kubectl...
Command: kubectl delete deployment nginx -n production
Risk: HIGH | Confidence: 95%

🚨 HIGH RISK: Destructive operation in PRODUCTION
Cluster: prod-cluster
Namespace: production

Type "nginx" to confirm deletion: 
```

Type `nginx` exactly to execute (case-sensitive).

---

## View Command History

### Show Today's Commands

```
kaido> show history today
```

**Expected Output**:
```
Commands executed today:

ID   Time      Command                              Environment   Action
1    14:32:15  kubectl get pods -n default          dev-cluster   EXECUTED
2    14:35:20  kubectl scale deployment nginx...    dev-cluster   EXECUTED
3    14:40:10  kubectl delete deployment test       dev-cluster   CANCELLED
```

### Filter by Environment

```
kaido> show history production
```

**Lists only commands executed in production contexts.**

### Show Last Week

```
kaido> show history last week
```

**Lists all commands from the past 7 days.**

---

## Advanced: Manual kubectl Commands

If AI translation fails or you prefer direct kubectl:

```
kaido> !kubectl get pods --all-namespaces
```

**Note**: Commands prefixed with `!` bypass AI translation and execute directly. Risk classification still applies.

---

## Troubleshooting

### Error: "Invalid OpenAI API key"

**Cause**: API key in `~/.kaido/config.toml` is incorrect.

**Solution**:
1. Verify API key at https://platform.openai.com/api-keys
2. Update `api_key` field in config file
3. Restart Kaido

### Error: "kubectl context not configured"

**Cause**: No active kubectl context set.

**Solution**:
```bash
kubectl config use-context <your-context-name>
```

### Low Confidence Warning

**Example**:
```
kaido> show logs

Translating to kubectl...
Command: kubectl logs
Risk: LOW | Confidence: 40%

⚠️  Low confidence (40%) - Please review command carefully
Suggestion: Which pod? Specify pod name (e.g., kubectl logs <pod-name>)

[Edit Command] [Execute Anyway] [Cancel]
```

**Options**:
- **Edit Command**: Modify AI-generated command before execution
- **Execute Anyway**: Run as-is (useful if command is correct despite low confidence)
- **Cancel**: Abort and try rephrasing your request

**Better Input**:
```
kaido> show logs for nginx-deployment-abc123
```

### API Timeout

**Cause**: OpenAI API is slow or unreachable.

**Solution**: Kaido automatically offers fallback:
```
OpenAI request timed out. Enter kubectl command manually:
kaido> kubectl get pods
```

---

## Configuration Reference

### Full `~/.kaido/config.toml` Example

```toml
[ai]
provider = "openai"
api_key = "sk-your-api-key"
model = "gpt-4-turbo-preview"
timeout_seconds = 10

[audit]
database_path = "~/.kaido/audit.db"
retention_days = 90

[safety]
require_typed_confirmation_in_production = true

[display]
show_confidence_threshold = 70  # Show warning if confidence below this
show_reasoning = false  # Set to true to see AI reasoning for every command
```

### Environment Variables

Override config values with environment variables:

```bash
export KAIDO_OPENAI_API_KEY="sk-your-api-key"
export KAIDO_AUDIT_DB="$HOME/.kaido/audit.db"
./target/release/kaido
```

---

## Tips for Best Results

### 1. Be Specific

❌ **Vague**: `show logs`
✅ **Specific**: `show logs for nginx pod`

❌ **Vague**: `delete pods`
✅ **Specific**: `delete pod nginx-abc123`

### 2. Use Natural Language

✅ All of these work:
- `show all pods`
- `list pods`
- `get pods in default namespace`
- `what pods are running`

### 3. Include Context When Ambiguous

✅ `scale my api deployment to 5`
✅ `delete deployment named old-service`
✅ `show logs for the nginx pod`

### 4. Trust the Confidence Score

- **90-100%**: AI is very confident, likely correct
- **70-89%**: AI is fairly confident, review before executing
- **<70%**: AI is uncertain, warning displayed automatically

### 5. Use History for Debugging

If a command fails:
```
kaido> show history today
```

Review the exact kubectl command that was executed and its exit code.

---

## Next Steps

- 📖 **Read the User Guide**: `/specs/007-kubectl-mvp-refactor/spec.md`
- 🐛 **Report Issues**: https://github.com/your-org/kaido-ai/issues
- 💬 **Join Discussion**: https://discord.gg/kaido-ai
- 📊 **View Your Audit Log**: Open `~/.kaido/audit.db` with any SQLite browser

---

## Beta Testing Feedback

We need your feedback! After 30 days of use, please share:

1. **Accuracy**: How often did AI translate your command correctly?
2. **Safety**: Did confirmation dialogs prevent any mistakes?
3. **Speed**: Was <5 second response time acceptable?
4. **Value**: Would you pay $50/month for this tool?

**Feedback Form**: https://forms.gle/kaido-kubectl-beta

---

## Uninstallation

```bash
# Remove binary
rm ./target/release/kaido

# Remove configuration and audit log (optional)
rm -rf ~/.kaido/

# Remove repository (optional)
cd ..
rm -rf kaido-ai/
```

---

**Version**: 0.2.0 (Kubectl MVP)  
**Last Updated**: 2025-10-25  
**Support**: support@kaido-ai.dev



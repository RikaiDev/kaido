# Manual Testing Checklist: TUI Interface Implementation

**Purpose**: Execute manual tests for TUI interface implementation (NO automated scripts per constitution)  
**Created**: 2025-10-24  
**Feature**: [spec.md](../spec.md) | [tasks.md](../tasks.md)  
**Tester**: _______________  
**Test Date**: _______________

---

## 🎯 Testing Strategy

**CRITICAL**: Per constitution's "Manual Development Process" principle:
- ❌ NO shell scripts for testing
- ❌ NO Python test scripts
- ❌ NO automated test runners
- ✅ ONLY manual, deliberate testing one scenario at a time

**How to Use This Checklist**:
1. Execute `cargo run` manually
2. Perform each test action
3. Mark `[x]` if test passes
4. Record observations in Notes column
5. If test fails, document issue and stop

---

## Phase 0: Pre-Test Verification

### CHK001 - Build Verification
- [ ] **Action**: Run `cargo build`
- [ ] **Expected**: Compilation succeeds with ZERO warnings
- [ ] **Notes**: _______________________

### CHK002 - GGUF Model Availability
- [ ] **Action**: Check `models/` directory has at least one `.gguf` file
- [ ] **Expected**: Model file exists (e.g., `llama-3.2-3b.gguf`)
- [ ] **Notes**: _______________________

---

## Phase 1: Basic TUI Startup (US1 & US3)

### CHK003 - Clean Launch Without llama.cpp Logs [T012]
- [ ] **Action**: Run `cargo run` from terminal
- [ ] **Expected**: 
  - TUI launches immediately
  - NO llama.cpp verbose logs visible (e.g., "llama_model_loader")
  - Only clean TUI interface displays
- [ ] **Fail Criteria**: Any llama.cpp initialization logs appear
- [ ] **Notes**: _______________________

### CHK004 - Split-Panel Layout Display [T027]
- [ ] **Action**: Observe initial TUI screen
- [ ] **Expected**:
  - Two distinct panels visible
  - Left panel (~70% width) shows "Command Shell" title
  - Right panel (~30% width) shows "AI Analysis" title
  - Panels have visible borders
- [ ] **Notes**: _______________________

### CHK005 - Input Area Visibility [T028]
- [ ] **Action**: Type characters: `hello world`
- [ ] **Expected**:
  - Characters appear in left panel
  - Prompt visible: `kaido> hello world`
  - Cursor visible (underscore character)
- [ ] **Notes**: _______________________

### CHK006 - Backspace Functionality
- [ ] **Action**: Press Backspace 5 times
- [ ] **Expected**: Text becomes `kaido> hello `
- [ ] **Notes**: _______________________

### CHK007 - Terminal Resize [T029]
- [ ] **Action**: Resize terminal window (make narrower, then wider)
- [ ] **Expected**:
  - Panels maintain ~70/30 ratio
  - No visual glitches or overlap
  - Text wraps appropriately
- [ ] **Notes**: _______________________

### CHK008 - Clean Exit [T030]
- [ ] **Action**: Press Ctrl+C
- [ ] **Expected**:
  - Application exits immediately
  - Terminal restored to normal mode
  - No leftover visual artifacts
- [ ] **Notes**: _______________________

### CHK009 - Panic Recovery [T008]
- [ ] **Action**: (If safe) Modify code to force panic, run, observe cleanup
- [ ] **Expected**: Terminal restored even on panic
- [ ] **WARNING**: Only attempt if comfortable with panic testing
- [ ] **Notes**: _______________________

---

## Phase 2: AI Spinner Animation (US2)

**Prerequisites**: Restart `cargo run` for fresh session

### CHK010 - Spinner Appears Immediately [T040]
- [ ] **Action**: Type `list files` and press Enter
- [ ] **Expected**:
  - Right panel immediately shows spinner character (e.g., ⠋)
  - "Thinking..." text visible
  - Response starts within <100ms
- [ ] **Notes**: _______________________

### CHK011 - Spinner Animation Cycles [T041]
- [ ] **Action**: Observe right panel during AI processing
- [ ] **Expected**:
  - Spinner character changes continuously
  - Smooth animation (≥10 FPS visually)
  - Cycles through all 10 frames: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
- [ ] **Notes**: _______________________

### CHK012 - Spinner Stops on Completion [T042]
- [ ] **Action**: Wait for AI to complete processing
- [ ] **Expected**:
  - Spinner disappears
  - Command output appears in left panel
  - Right panel shows "Ready" or remains blank
- [ ] **Notes**: _______________________

### CHK013 - Input Disabled During Processing [T043]
- [ ] **Action**: While AI thinking, try typing another command
- [ ] **Expected**:
  - Input disabled OR queued (specify which)
  - No confusion from simultaneous commands
- [ ] **Notes**: _______________________

---

## Phase 3: Toggle AI Analysis View (US4)

**Prerequisites**: Execute a command to generate AI output

### CHK014 - Toggle to JSON View [T072]
- [ ] **Action**: 
  - Type `list files`, press Enter
  - While spinner animating, press Ctrl+T
- [ ] **Expected**:
  - Right panel switches to JSON output
  - JSON is pretty-printed with indentation
  - Shows `task` and `commands` fields
- [ ] **Notes**: _______________________

### CHK015 - Toggle Back to Spinner [T073]
- [ ] **Action**: Press Ctrl+T again
- [ ] **Expected**:
  - Right panel returns to spinner/"Thinking..." view
  - Toggle state persists
- [ ] **Notes**: _______________________

### CHK016 - Toggle State Persistence [T074]
- [ ] **Action**:
  - Toggle to JSON view (Ctrl+T)
  - Execute new command
- [ ] **Expected**: JSON view remains active for new command
- [ ] **Notes**: _______________________

### CHK017 - JSON Pretty-Printing [T075]
- [ ] **Action**: Inspect JSON output in right panel
- [ ] **Expected**:
  - Proper indentation (2 or 4 spaces)
  - Readable structure
  - No raw escaped characters
- [ ] **Notes**: _______________________

---

## Phase 4: Safety Modal Dialog (US5)

**Prerequisites**: Prepare test file for deletion

### CHK018 - Setup Test File
- [ ] **Action**: Run `touch /tmp/kaido_test.txt` in separate terminal
- [ ] **Expected**: File created successfully
- [ ] **Notes**: _______________________

### CHK019 - Modal Appears for Dangerous Command [T062]
- [ ] **Action**: Type `rm /tmp/kaido_test.txt` and press Enter
- [ ] **Expected**:
  - Modal dialog appears centered on screen
  - Red background
  - Shows command: `rm /tmp/kaido_test.txt`
  - Shows 3 options: [1] Allow Once, [2] Allow Always, [3] Deny
- [ ] **Notes**: _______________________

### CHK020 - Modal Blocks Other Input [T066]
- [ ] **Action**: While modal visible, try typing other keys
- [ ] **Expected**: Only 1/2/3 keys are captured, all others ignored
- [ ] **Notes**: _______________________

### CHK021 - Option 1: Allow Once [T063]
- [ ] **Action**: Press `1` in modal
- [ ] **Expected**:
  - Command executes once
  - Modal disappears
  - Test file deleted
  - Verify: `ls /tmp/kaido_test.txt` → file not found
- [ ] **Notes**: _______________________

### CHK022 - Option 2: Allow Always [T064]
- [ ] **Action**:
  - Recreate test file: `touch /tmp/kaido_test2.txt`
  - In Kaido: `rm /tmp/kaido_test2.txt`, modal appears
  - Press `2`
  - Restart Kaido: `cargo run`
  - Try same command again: `rm /tmp/kaido_test2.txt`
- [ ] **Expected**:
  - First time: Modal appears, press 2, file deleted
  - Second time (after restart): NO modal, executes directly
  - Allowlist persisted to `~/.config/kaido/allowlist.txt`
- [ ] **Notes**: _______________________

### CHK023 - Option 3: Deny [T065]
- [ ] **Action**:
  - Create test file: `touch /tmp/kaido_test3.txt`
  - In Kaido: `rm /tmp/kaido_test3.txt`, modal appears
  - Press `3`
- [ ] **Expected**:
  - Command cancelled
  - Modal disappears
  - File still exists: `ls /tmp/kaido_test3.txt` → file exists
  - User returns to normal prompt
- [ ] **Notes**: _______________________

---

## Phase 5: Edge Cases & Stress Tests

### CHK024 - Minimum Terminal Size [T082]
- [ ] **Action**: Resize terminal to ~60 columns width
- [ ] **Expected**:
  - Panels degrade gracefully OR switch to stacked layout
  - Text remains readable
  - No crashes or visual corruption
- [ ] **Notes**: _______________________

### CHK025 - Very Wide Terminal
- [ ] **Action**: Maximize terminal width (>200 columns)
- [ ] **Expected**:
  - Panels maintain 70/30 ratio
  - Content displays correctly
  - No unnecessary whitespace issues
- [ ] **Notes**: _______________________

### CHK026 - Rapid Command Entry [T083]
- [ ] **Action**: Type and execute 5 commands rapidly without waiting
- [ ] **Expected**:
  - Commands queued OR input blocked appropriately
  - No command loss
  - No UI corruption
- [ ] **Notes**: _______________________

### CHK027 - Very Long Output [T084]
- [ ] **Action**: Run command producing thousands of lines (e.g., `find /`)
- [ ] **Expected**:
  - Output area scrolls
  - No memory issues
  - Performance remains acceptable (<100ms frame time)
  - Output may be truncated with warning (per 100KB limit)
- [ ] **Notes**: _______________________

### CHK028 - No Model Available
- [ ] **Action**: Rename `models/` directory temporarily, restart Kaido
- [ ] **Expected**:
  - Error message displayed in TUI (not raw panic)
  - User-friendly message about missing model
  - Graceful handling
- [ ] **Notes**: _______________________

### CHK029 - Direct Shell Command (No AI)
- [ ] **Action**: Type `ls` and press Enter
- [ ] **Expected**:
  - Command executes directly without AI processing
  - No spinner animation (or brief animation)
  - Output appears in left panel
- [ ] **Notes**: _______________________

---

## Phase 6: Multi-Language & Complex Commands

### CHK030 - Chinese Input
- [ ] **Action**: Type `查詢當前資料夾的檔案`
- [ ] **Expected**:
  - Chinese characters display correctly
  - AI processes natural language
  - Command sequence generated
- [ ] **Notes**: _______________________

### CHK031 - Multi-Step Task
- [ ] **Action**: Type `連到 stage 主機並查看日誌`
- [ ] **Expected**:
  - AI generates multiple commands in sequence
  - JSON shows array of commands
  - Each command displays in output
- [ ] **Notes**: _______________________

### CHK032 - Mixed English/Chinese
- [ ] **Action**: Type `list all files in 當前目錄`
- [ ] **Expected**: AI understands mixed language input
- [ ] **Notes**: _______________________

---

## Phase 7: End-to-End Workflow [T086]

### CHK033 - Complete User Workflow
- [ ] **Action**: Execute realistic workflow:
  1. Launch Kaido: `cargo run`
  2. Check directory: `list files`
  3. Toggle JSON view: Ctrl+T
  4. Create file: `create test file with content hello`
  5. Dangerous operation: `delete the test file`
  6. Handle modal: Choose option
  7. Exit: Ctrl+C

- [ ] **Expected**:
  - All operations smooth
  - No crashes or hangs
  - Terminal restored on exit
- [ ] **Notes**: _______________________

---

## 🔍 Post-Testing Validation

### CHK034 - Zero Warnings Check
- [ ] **Action**: Run `cargo build 2>&1 | grep warning`
- [ ] **Expected**: Output is empty (no warnings)
- [ ] **Notes**: _______________________

### CHK035 - Allowlist File Integrity
- [ ] **Action**: Check `~/.config/kaido/allowlist.txt`
- [ ] **Expected**:
  - File exists and is readable
  - Contains commands from "Allow Always" tests
  - One command per line
- [ ] **Notes**: _______________________

### CHK036 - No Leftover Processes
- [ ] **Action**: Run `ps aux | grep kaido`
- [ ] **Expected**: No orphaned Kaido processes
- [ ] **Notes**: _______________________

---

## 📊 Test Summary

**Total Tests**: 36  
**Passed**: ___ / 36  
**Failed**: ___ / 36  
**Blocked**: ___ / 36  
**Skipped**: ___ / 36

**Critical Issues Found**: _______________________

**Non-Critical Issues Found**: _______________________

**Overall Status**: ☐ PASS  ☐ FAIL  ☐ PARTIAL

**Tester Signature**: _______________  
**Date Completed**: _______________

---

## 🚨 Issue Tracking

| Issue ID | Test CHK | Severity | Description | Status |
|----------|----------|----------|-------------|--------|
| ISS001   |          |          |             |        |
| ISS002   |          |          |             |        |
| ISS003   |          |          |             |        |

**Severity Legend**:
- **CRITICAL**: Blocks core functionality, violates constitution
- **HIGH**: Major UX issue or safety concern
- **MEDIUM**: Minor functionality issue
- **LOW**: Cosmetic or edge case

---

## 📝 Tester Notes

_Use this space for additional observations, suggestions, or context:_

```
[Free-form notes here]
```


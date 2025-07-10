# 🔍 **Jira Item Audit: [UV-XXX]**                │
│                                                    │
│  ## **Input Required**                             │
│  - **Issue Key**: UV-XXX                           │
│  - **Issue Title**: [Title from Jira]              │
│  - **Issue Type**: [Task/Story/Epic/Bug/Subtask]   │
│  - **Priority**: [High/Medium/Low]                 │
│  - **Description**: [Brief description of what th  │
│  issue claims to address]                          │
│                                                    │
│  ## **Audit Execution Steps**                      │
│                                                    │
│  ### **Step 1: Codebase Investigation**            │
│  ```bash                                           │
│  # Search for related implementations using        │
│  keywords from issue                               │
│  grep_file_content "[primary keyword from title]"  │
│  grep_file_content "[secondary keyword/component   │
│  name]"                                            │
│  grep_file_content "[any specific file/function    │
│  names mentioned]"                                 │
│                                                    │
│  # If specific files are mentioned in description  │
│  check them                                        │
│  open_files ["file1.rs", "file2.rs"]               │
│                                                    │
│  # Look for test coverage                          │
│  grep_file_content "test.*[issue_topic]"           │
│                                                    │
│                                                    │
│         Step 2: Quick Assessment Checklist         │
│                                                    │
│ Implementation Status (check one):                 │
│                                                    │
│  • [ ] Not Started: No related code found          │
│  • [ ] Partially Implemented: Some components      │
│    exist but incomplete                            │
│  • [ ] Fully Implemented: Complete implementation  │
│    exists                                          │
│  • [ ] Over-Implemented: More than requested       │
│    exists                                          │
│                                                    │
│ Accuracy (check one):                              │
│                                                    │
│  • [ ] Accurate: Description matches codebase      │
│    reality                                         │
│  • [ ] Outdated: Description based on old codebase │
│    state                                           │
│  • [ ] Incorrect: Description doesn't match        │
│    current architecture                            │
│  • [ ] Vague: Description too unclear to verify    │
│                                                    │
│ Test Coverage (check one):                         │
│                                                    │
│  • [ ] No Tests: No test coverage found            │
│  • [ ] Partial Tests: Some test coverage exists    │
│  • [ ] Full Tests: Comprehensive test coverage     │
│                                                    │
│              Step 3: Evidence Summary              │
│                                                    │
│ Files Found:                                       │
│                                                    │
│  • path/to/file1.rs - [brief description of what   │
│    it contains]                                    │
│  • path/to/file2.rs - [brief description of what   │
│    it contains]                                    │
│                                                    │
│ Related Code: [Brief description of existing       │
│ implementations found]                             │
│                                                    │
│ Test Coverage: [Description of tests found or lack │
│ thereof]                                           │
│                                                    │
│            Step 4: Final Recommendation            │
│                                                    │
│ Action: [KEEP/CLOSE/MERGE/UPDATE/CLARIFY]          │
│ Reasoning: [1-2 sentence explanation] Sprint       │
│ Ready: [Yes/No - can this be assigned to a sprint  │
│ as-is?]                                            │
│                                                    │
│ ────────────────────────────────────────────────── │
│                                                    │
│                  Output Template                   │
│                                                    │
│                                                    │
│  ## 🔍 **Audit Report: UV-XXX**                    │
│  **Issue**: [Title]                                │
│  **Status**: [Current] | **Recommended Action**:   │
│  [KEEP/CLOSE/MERGE/UPDATE]                         │
│                                                    │
│  ### **Findings**                                  │
│  - **Implementation**: [Not                        │
│  Started/Partial/Complete/Over-implemented]        │
│  - **Accuracy**:                                   │
│  [Accurate/Outdated/Incorrect/Vague]               │
│  - **Test Coverage**: [None/Partial/Complete]      │
│                                                    │
│  ### **Evidence**                                  │
│  - Files checked: [list]                           │
│  - Related code found: [brief description]         │
│                                                    │
│  ### **Recommendation**                            │
│  [Action] - [Reasoning in 1-2 sentences]           │
│  **Sprint Ready**: [Yes/No]                        │
│                                                    │
│                                                    │
│                                                    │
│                 Usage Instructions                 │
│                                                    │
│  1 Copy this prompt                                │
│  2 Fill in the "Input Required" section with       │
│    details from the specific Jira item             │
│  3 Execute the grep/open_files commands in Step 1  │
│  4 Complete the checklists in Step 2 based on      │
│    findings                                        │
│  5 Fill in Step 3 with evidence                    │
│  6 Make recommendation in Step 4                   │
│  7 Format final output using the template          │
│                                                    │
│                                                 
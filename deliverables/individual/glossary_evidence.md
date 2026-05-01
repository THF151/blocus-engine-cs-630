# Glossary of Evidence Types

> **Purpose**: This glossary provides you guidance for what can constitute valid evidence for contributions and reproducible counterexamples. When linking evidence, be specific and direct (e.g., commit hashes, file paths, issues, PR URLs).

## **Commits**
- **Definition**: Git commits that contain your direct contributions
- **What to Link**: 
  - Specific commit hashes (e.g., `abc1234`)
  - Commit messages that describe your work
- **How to Link**: 
  - Full URL: `https://github.com/org/repo/commit/abc1234`
  - Short reference: `Commit abc1234`
- **What Counts**: 
  - Code changes
  - Documentation updates
  - Test additions/modifications
  - Configuration changes
  - etc.

## **Tests**
- **Definition**: Test files or specific test cases you authored
- **What to Link**:
  - Test file paths (e.g., `tests/board_test.py`)
  - Specific test functions (e.g., `test_board_initialization()`)
- **How to Link**:
  - File: `tests/board_test.py`
  - Function: `tests/board_test.py::test_board_initialization`
- **What Counts**:
  - Unit tests
  - Integration tests
  - Edge case tests
  - Test fixtures/setup code
  - etc.

## **Documentation**
- **Definition**: Documentation files you wrote or significantly modified
- **What to Link**:
  - README sections
  - API documentation
  - Design documents
  - Inline code comments (for major explanations)
- **How to Link**:
  - File path: `docs/design.md`
  - Section: `docs/design.md#board-representation`
- **What Counts**:
  - Architecture decisions
  - Usage instructions
  - Explanation of complex logic
  - Known limitations

## **Pull Requests / Merge Requests**
- **Definition**: GitHub/GitLab issues you created or primarily worked on
- **What to Link**:
  - Issue number and title
  - Your specific contributions within the PR
- **How to Link**:
  - `PR #123: Add board validation logic`
  - `https://github.com/org/repo/pull/123`
- **What Counts**:
  - PRs you created
  - Significant reviews you provided
  - PRs where you were the primary author

## **Issues**
- **Definition**: GitHub/GitLab issues you created or primarily worked on
- **What to Link**:
  - Issue number and title
  - Your comments or solutions
- **How to Link**:
  - `Issue #45: Fix corner touch validation`
  - `https://github.com/org/repo/issues/45`
- **What Counts**:
  - Bug reports
  - Feature requests
  - Task breakdowns
  - Solutions you implemented

## **Files**
- **Definition**: Source code files you primarily authored
- **What to Link**:
  - File paths with line ranges if applicable
  - Key functions/classes in the file
- **How to Link**:
  - File: `src/board.py`
  - Specific: `src/board.py#L15-L45`
- **What Counts**:
  - Core implementation files
  - Configuration files
  - Build scripts you modified

## **Reproducible Counterexamples**
- **Definition**: Documentation of failures that can be reproduced by others
- **What to Include**:
  - Prompt/context used with AI tool
  - AI output that failed
  - Expected vs. actual behavior
  - Steps to reproduce
  - Fix/refinement applied
- **How to Link**:
  - Separate markdown file: `evidence/counterexamples/counterexample-1-board-validation.md`
  - Include all necessary context in the file
- **What Counts**:
  - Failed AI suggestions
  - Guidelines that didn't work
  - Edge cases that broke expectations
  - Debugging sessions with documented outcomes

---

## **Best Practices for Evidence Links**

1. **Be Specific**: Link to exact commits, functions, or sections—not just repository roots
2. **Be Persistent**: Use commit hashes (not branch names) for permanent references
3. **Be Complete**: Include enough context so others can understand the evidence without extra digging
4. **Be Verifiable**: Ensure links work and content hasn't been deleted

---

*Template version: 1.0 | Last updated: 24 February 2026*

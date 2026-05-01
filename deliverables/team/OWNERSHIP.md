# OWNERSHIP.md

> **Team Project Ownership Table (who did what)**  
> *Document individual contributions and responsibilities for the Blokus Game Engine project.*

---

## Team Information

**Team Name:** `[Your Team Name/ID]`  
**Project:** Blokus Game Engine (Classic + Duo)  
**Date:** `[Last Updated Date]`  
**Team Members:** `[Student Name 1, Student Name 2, Student Name 3, ...]`

---

## Work Package Ownership

| Package Name | Owner | Responsibilities | Acceptance Criteria | Evidence Links |
|--------------|-------|------------------|---------------------|----------------|
| `[Package Name]` | `[Owner Name]` | `[List of responsibilities]` | `[Specific acceptance criteria]` | `[Links to commits, tests, docs]` |


---

## Example Work Packages (for reference)

| Package Name | Owner | Responsibilities | Acceptance Criteria | Evidence Links |
|--------------|-------|------------------|---------------------|----------------|
| Game Board Core | `[Name]` | Implement board representation, piece placement logic, board state management | - Board can be initialized for Classic (20x20) and Duo (14x14)<br>- Pieces can be placed and removed<br>- Board state can be serialized to JSON | `[Link to commits]`, `[Link to tests]` |
| Move Validation | `[Name]` | Implement legality checker for moves, corner-touch rule enforcement | - Legal moves are correctly identified<br>- Invalid moves are correctly rejected<br>- Edge cases handled (first move, corner touches) | `[Link to commits]`, `[Link to tests]` |

> **Note:** Use these as examples only. Define additional packages relevant to your team's implementation (i.e., splitting of tasks depend on team size etc.).

---

## Instructions for Use

1. **Replace all `[...]` placeholders** with your team's specific content
2. **List all major work packages** in your project
3. **Assign primary ownership** to team members (one owner per package)
4. **Include specific acceptance criteria** that can be verified
5. **Link to evidence** (commits, tests, documentation) for each package
6. **Keep this updated** throughout the semester as work progresses
7. **Submit as `OWNERSHIP.md`** in your project repository

---

## Notes

- Each team member should have at least one primary ownership package
- Other team members can contribute to any package, but the primary owner is responsible for reviewing and ensuring correctness
- Evidence links should point to specific commits, pull requests, or files in your repository
- Update this document regularly (e.g., after each sprint or milestone)

---

*Template version: 1.0 | Last updated: 24 February 2026*

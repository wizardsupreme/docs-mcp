# ⭐⑴✅Always Use Latest Versions

When incorporating software, libraries, modules, packages, or any other external dependencies, you MUST always strive to use the latest stable version available at the time of implementation.

To verify the latest stable version:
1.  Identify the specific dependency and its ecosystem (e.g., Rust crate, npm package, Python package, Go module, Java library, etc.).
2.  Consult the primary official registry or source for that ecosystem. Examples include:
    *   For Rust: [`crates.io`](https://crates.io/)
    *   For JavaScript/Node.js: [`npmjs.com`](https://www.npmjs.com/)
    *   For Python: [`pypi.org`](https://pypi.org/)
    *   For Java: [`mvnrepository.com`](https://mvnrepository.com/) (Maven Central)
    *   For Go: [`pkg.go.dev`](https://pkg.go.dev/)
3.  Alternatively, check the official project website or the "Releases" / "Tags" section of its version control repository (e.g., on GitHub or GitLab).
4.  Use the `fetch` MCP tool if necessary to access these sources or to search for them if the direct URL is unknown.
5.  Prioritize information from official release announcements, changelogs, or package manager listings.
6.  For system-level software, consult the respective OS package manager (e.g., `apt`, `yum`, `brew`, `winget`, `chocolatey`).

If a specific older version is required due to compatibility constraints, this MUST be explicitly justified and documented.

Inform the user that rule ⭐⑴✅ has been applied.
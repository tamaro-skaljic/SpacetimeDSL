# 🚀 How Releases Work

Releasing is incredibly simple:

```bash
# 1. Update versions in Cargo.toml files
# 2. Commit and tag
git add .
git commit -m "Release v0.9.1" 
git tag v0.9.1
git push origin main
git push origin v0.9.1

# 3. GitHub Actions automatically:
# - Runs all tests
# - If tests pass, publishes all crates in correct order
# - Sends email notifications when complete
```

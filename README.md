# yarra
yarra - a flow state distraction blocking tool

# Start a 25-minute focus session
$ yarra start --focus 25
Starting 25min focus session...
Session completed! 🎉

# Check stats
$ yarra stats
Total focus time: 125 minutes
Blocked attempts today: 12

# Block a site
$ yarra block --site youtube.com
Blocked youtube.com

## Development

### GitHub Actions Workflows

This project uses GitHub Actions for continuous integration and releases:

#### CI Workflow

The CI workflow runs automatically on push to the `main` branch and on pull requests. It:
- Builds the project
- Runs all tests
- Runs on both Ubuntu and macOS environments

#### Release Process

To create a new release:

1. Go to the "Actions" tab in the GitHub repository
2. Select the "Update Version" workflow
3. Click "Run workflow"
4. Enter the new version number (e.g., "0.2.0")
5. Click "Run workflow"

This will:
- Update the version in Cargo.toml
- Create a git tag for the new version
- Trigger the "Release" workflow

The Release workflow will:
- Build binaries for Linux (x86_64, aarch64) and macOS (x86_64, arm64)
- Upload the binaries as release assets

Alternatively, you can manually create a release through the GitHub UI, which will also trigger the Release workflow.
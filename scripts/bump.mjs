// EPIC-009: bump to the next pre-release version and commit — never tag, never push.
//
//   npm run bump -- <patch|minor|major|rc> [--dry-run] [--from <version>]
//
// patch/minor/major bump the base version (pre-release suffix stripped) and append "-1".
// rc increments the numeric pre-release suffix and errors when there is none.
// The suffix is numeric-only: the MSI bundler rejects non-numeric pre-release identifiers.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

const versionedFiles = [
  "package.json",
  "package-lock.json",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
];

function parseVersion(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)(?:-(\d+))?$/);
  if (!match) {
    throw new Error(
      `Unsupported version "${version}": expected MAJOR.MINOR.PATCH with an optional numeric-only pre-release suffix (e.g. 2.4.1-1).`
    );
  }
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    suffix: match[4] === undefined ? null : Number(match[4]),
  };
}

function nextVersion(current, type) {
  const { major, minor, patch, suffix } = parseVersion(current);
  switch (type) {
    case "major":
      return `${major + 1}.0.0-1`;
    case "minor":
      return `${major}.${minor + 1}.0-1`;
    case "patch":
      return `${major}.${minor}.${patch + 1}-1`;
    case "rc":
      if (suffix === null) {
        throw new Error(`Version ${current} has no pre-release suffix to increment; use patch, minor, or major.`);
      }
      return `${major}.${minor}.${patch}-${suffix + 1}`;
    default:
      throw new Error(`Unknown bump type "${type}": expected patch, minor, major, or rc.`);
  }
}

function run(command, args) {
  execFileSync(command, args, { cwd: repoRoot, stdio: "inherit" });
}

function main() {
  const args = process.argv.slice(2);
  const dryRun = args.includes("--dry-run");
  const fromIndex = args.indexOf("--from");
  const fromVersion = fromIndex === -1 ? null : args[fromIndex + 1];
  const type = args.find((arg) => !arg.startsWith("--") && arg !== fromVersion);

  if (!type) {
    throw new Error("Usage: npm run bump -- <patch|minor|major|rc> [--dry-run] [--from <version>]");
  }
  if (fromIndex !== -1 && !fromVersion) {
    throw new Error("--from requires a version argument.");
  }

  const currentVersion =
    fromVersion ?? JSON.parse(fs.readFileSync(path.join(repoRoot, "package.json"), "utf8")).version;
  const newVersion = nextVersion(currentVersion, type);

  if (dryRun) {
    console.log(newVersion);
    return;
  }

  run("node", [path.join(__dirname, "sync-version.mjs"), newVersion]);
  run("git", ["add", "--", ...versionedFiles]);
  run("git", ["commit", "-m", `chore: bump version to ${newVersion}`]);
  console.log(`Bumped ${currentVersion} -> ${newVersion} and committed. Push when ready to build the rc.`);
}

try {
  main();
} catch (error) {
  console.error(error.message || error);
  process.exit(1);
}

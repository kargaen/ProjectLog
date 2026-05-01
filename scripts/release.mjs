import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");

const releaseLogPath = path.join(repoRoot, "RELEASE_LOG.md");
const releaseLogTemplate = [
  "# Release Log",
  "",
  "Write the next release message here as you build features.",
  "",
  "Guidelines:",
  "- Keep the first line short. Git will use it as the commit subject.",
  "- Add extra lines below for details when useful.",
  "- Remove placeholder text before releasing.",
].join("\r\n");

const releaseFiles = [
  "package.json",
  "package-lock.json",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
  "RELEASE_LOG.md",
];

function run(command, args, options = {}) {
  execFileSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
    ...options,
  });
}

function runCapture(command, args) {
  return execFileSync(command, args, {
    cwd: repoRoot,
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
  }).trim();
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function getCurrentVersion() {
  return readJson(path.join(repoRoot, "package.json")).version;
}

function bumpVersion(currentVersion, releaseType) {
  const match = currentVersion.match(/^(\d+)\.(\d+)\.(\d+)(?:-.+)?$/);
  if (!match) {
    throw new Error(`Current version is not a supported semver value: ${currentVersion}`);
  }

  let major = Number(match[1]);
  let minor = Number(match[2]);
  let patch = Number(match[3]);

  switch (releaseType) {
    case "major":
      major += 1;
      minor = 0;
      patch = 0;
      break;
    case "minor":
      minor += 1;
      patch = 0;
      break;
    case "patch":
      patch += 1;
      break;
    default:
      throw new Error("Release type must be major, minor, patch, or an explicit version.");
  }

  return `${major}.${minor}.${patch}`;
}

function isValidVersion(version) {
  return /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version);
}

function getReleaseMessage() {
  if (!fs.existsSync(releaseLogPath)) {
    throw new Error("RELEASE_LOG.md was not found.");
  }

  const raw = fs.readFileSync(releaseLogPath, "utf8").replace(/\r/g, "");
  const lines = raw.split("\n");
  const messageLines = [];

  for (const line of lines) {
    const trimmed = line.trim();

    if (!trimmed) {
      if (messageLines.length > 0 && messageLines.at(-1) !== "") {
        messageLines.push("");
      }
      continue;
    }

    if (trimmed === "# Release Log") continue;
    if (trimmed === "Guidelines:") break;
    if (trimmed === "Write the next release message here as you build features.") continue;
    if (trimmed.startsWith("- Keep the first line short")) break;
    if (trimmed.startsWith("- Add extra lines below")) break;
    if (trimmed.startsWith("- Remove placeholder text before releasing.")) break;

    messageLines.push(line.trimEnd());
  }

  while (messageLines.length > 0 && messageLines.at(-1) === "") {
    messageLines.pop();
  }

  const message = messageLines.join("\r\n").trim();
  if (!message) {
    throw new Error("RELEASE_LOG.md is empty. Add the release message before creating a new version.");
  }

  return message;
}

function backupFiles(files) {
  return new Map(
    files.map((relativePath) => {
      const fullPath = path.join(repoRoot, relativePath);
      return [fullPath, fs.existsSync(fullPath) ? fs.readFileSync(fullPath, "utf8") : null];
    })
  );
}

function restoreFiles(backup) {
  for (const [fullPath, content] of backup.entries()) {
    if (content === null) {
      fs.rmSync(fullPath, { force: true });
    } else {
      fs.writeFileSync(fullPath, content, "utf8");
    }
  }
}

async function promptForVersion(currentVersion) {
  const rl = readline.createInterface({ input, output });
  try {
    console.log(`Current version: ${currentVersion}`);
    console.log("This will sync version files, create a release commit, tag v<version>, and push both branch and tag.");
    console.log("Reply with major, minor, patch, or an explicit version like 2.4.0.");

    const entered = (await rl.question("Release type or version: ")).trim();
    if (!entered) {
      console.log("No version entered. Release cancelled.");
      process.exit(0);
    }

    const normalized = ["major", "minor", "patch"].includes(entered)
      ? bumpVersion(currentVersion, entered)
      : entered;

    if (normalized === currentVersion) {
      throw new Error("New version must differ from the current version.");
    }

    if (!isValidVersion(normalized)) {
      throw new Error("Version must look like 1.2.3 or 1.2.3-beta.1");
    }

    const confirmation = (await rl.question(`Type ${normalized} again to confirm release: `)).trim();
    if (confirmation !== normalized) {
      console.log("Confirmation did not match. Release cancelled.");
      process.exit(0);
    }

    return normalized;
  } finally {
    rl.close();
  }
}

async function main() {
  const currentVersion = getCurrentVersion();
  const requestedValue = process.argv[2]?.trim();
  const requestedVersion = requestedValue
    ? ["major", "minor", "patch"].includes(requestedValue)
      ? bumpVersion(currentVersion, requestedValue)
      : requestedValue
    : undefined;
  const newVersion = requestedVersion || (await promptForVersion(currentVersion));

  if (requestedVersion) {
    if (requestedVersion === currentVersion) {
      throw new Error("New version must differ from the current version.");
    }
    if (!isValidVersion(requestedVersion)) {
      throw new Error("Version must look like 1.2.3 or 1.2.3-beta.1");
    }
  }

  const existingTag = runCapture("git", ["tag", "-l", `v${newVersion}`]);
  if (existingTag) {
    throw new Error(`Tag v${newVersion} already exists.`);
  }

  const releaseMessage = getReleaseMessage();
  const backup = backupFiles(releaseFiles);
  const commitMessageFile = path.join(os.tmpdir(), `projectlog-release-${Date.now()}.txt`);
  let releaseCommitCreated = false;

  try {
    fs.writeFileSync(commitMessageFile, releaseMessage, "utf8");

    run("node", ["scripts/sync-version.mjs", newVersion]);
    fs.writeFileSync(releaseLogPath, releaseLogTemplate, "utf8");

    run("git", ["add", "-A"]);
    run("git", ["commit", "-F", commitMessageFile]);
    releaseCommitCreated = true;
    run("git", ["tag", `v${newVersion}`]);

    const branch = runCapture("git", ["branch", "--show-current"]);
    if (!branch) {
      throw new Error("Could not determine the current branch.");
    }

    try {
      runCapture("git", ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]);
      run("git", ["push"]);
    } catch {
      run("git", ["push", "-u", "origin", branch]);
    }

    run("git", ["push", "origin", `v${newVersion}`]);
    console.log(`Released version ${newVersion} on branch ${branch}`);
  } catch (error) {
    if (!releaseCommitCreated) {
      restoreFiles(backup);
      console.log("Release failed before commit; restored version files and release log.");
    }
    throw error;
  } finally {
    fs.rmSync(commitMessageFile, { force: true });
  }
}

main().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});

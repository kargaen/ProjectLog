import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");

const packageJsonPath = path.join(repoRoot, "package.json");
const packageLockPath = path.join(repoRoot, "package-lock.json");
const cargoTomlPath = path.join(repoRoot, "src-tauri", "Cargo.toml");
const cargoLockPath = path.join(repoRoot, "src-tauri", "Cargo.lock");

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

function assertVersion(version) {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid version: ${version}`);
  }
}

function replaceRequired(source, pattern, replacement, label) {
  if (!pattern.test(source)) {
    throw new Error(`Could not update ${label}`);
  }
  return source.replace(pattern, replacement);
}

const explicitVersion = process.argv[2]?.trim();

const packageJson = readJson(packageJsonPath);
const targetVersion = explicitVersion || packageJson.version;
assertVersion(targetVersion);

packageJson.version = targetVersion;
writeJson(packageJsonPath, packageJson);

const packageLock = readJson(packageLockPath);
packageLock.version = targetVersion;
if (packageLock.packages?.[""]) {
  packageLock.packages[""].version = targetVersion;
}
writeJson(packageLockPath, packageLock);

const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const updatedCargoToml = replaceRequired(
  cargoToml,
  /(\[package\][\s\S]*?^version = ").*?(")/m,
  `$1${targetVersion}$2`,
  "src-tauri/Cargo.toml"
);
fs.writeFileSync(cargoTomlPath, updatedCargoToml, "utf8");

const cargoLock = fs.readFileSync(cargoLockPath, "utf8");
const updatedCargoLock = replaceRequired(
  cargoLock,
  /(name = "project-log"\r?\nversion = ").*?(")/,
  `$1${targetVersion}$2`,
  "src-tauri/Cargo.lock"
);
fs.writeFileSync(cargoLockPath, updatedCargoLock, "utf8");

execSync(`git add "${cargoTomlPath}" "${cargoLockPath}"`);

console.log(`Synchronized project version to ${targetVersion}`);

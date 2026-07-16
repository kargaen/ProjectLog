// Pins scripts/bump.mjs to the EPIC-009 §3 case table. Run: node scripts/bump.test.mjs
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const bumpScript = path.join(__dirname, "bump.mjs");

const cases = [
  { from: "2.4.0", type: "patch", expect: "2.4.1-1" },
  { from: "2.4.0-4", type: "patch", expect: "2.4.1-1" },
  { from: "2.4.1-2", type: "minor", expect: "2.5.0-1" },
  { from: "2.4.0-4", type: "major", expect: "3.0.0-1" },
  { from: "2.4.1-2", type: "rc", expect: "2.4.1-3" },
  { from: "2.4.0", type: "rc", expect: null }, // error: no pre-release suffix to increment
];

let failures = 0;

for (const { from, type, expect } of cases) {
  let stdout = null;
  let error = null;
  try {
    stdout = execFileSync("node", [bumpScript, type, "--dry-run", "--from", from], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  } catch (err) {
    error = err;
  }

  if (expect === null) {
    if (error) {
      console.log(`ok    ${from} + ${type} -> error, as expected`);
    } else {
      failures += 1;
      console.error(`FAIL  ${from} + ${type} -> expected an error, got "${stdout}"`);
    }
  } else if (error) {
    failures += 1;
    console.error(`FAIL  ${from} + ${type} -> expected "${expect}", got error: ${error.message.split("\n")[0]}`);
  } else if (stdout !== expect) {
    failures += 1;
    console.error(`FAIL  ${from} + ${type} -> expected "${expect}", got "${stdout}"`);
  } else {
    console.log(`ok    ${from} + ${type} -> ${stdout}`);
  }
}

if (failures > 0) {
  console.error(`${failures} case(s) failed`);
  process.exit(1);
}
console.log(`${cases.length} cases passed`);

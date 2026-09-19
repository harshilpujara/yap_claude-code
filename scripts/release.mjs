// Usage: npm run release -- 0.1.0
// Sets the version everywhere it lives, commits the change (if any) and creates the tag v<version>.
// It never pushes: you do that yourself (see README, "Cut a release").
import { readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const version = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(version || "")) {
  console.error("Give a version like 0.1.0 (numbers only):  npm run release -- 0.1.0");
  process.exit(1);
}
const run = (cmd, args, opts = {}) => execFileSync(cmd, args, { stdio: "inherit", ...opts });
const out = (cmd, args) => execFileSync(cmd, args, { encoding: "utf8" }).trim();

if (out("git", ["status", "--porcelain"])) {
  console.error("You have uncommitted changes. Commit or discard them first, then run this again.");
  process.exit(1);
}
if (out("git", ["tag", "--list", `v${version}`])) {
  console.error(`The tag v${version} already exists. Pick a new version number.`);
  process.exit(1);
}

function rewrite(file, pattern, replacement) {
  const text = readFileSync(file, "utf8");
  if (!pattern.test(text)) throw new Error(`Could not find the version in ${file}`);
  writeFileSync(file, text.replace(pattern, replacement));
}
rewrite("package.json", /("version"\s*:\s*")[^"]+(")/, `$1${version}$2`);
rewrite("src-tauri/tauri.conf.json", /("version"\s*:\s*")[^"]+(")/, `$1${version}$2`);
rewrite("src-tauri/Cargo.toml", /^(version\s*=\s*")[^"]+(")/m, `$1${version}$2`);
run("cargo", ["update", "-p", "yapp", "--offline"], { cwd: "src-tauri" });

if (out("git", ["status", "--porcelain"])) {
  run("git", ["add", "package.json", "src-tauri/tauri.conf.json", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]);
  run("git", ["commit", "-m", `Release v${version}`]);
}
run("git", ["tag", "-a", `v${version}`, "-m", `yapp v${version}`]);
console.log(`\nDone. Now push with:\n  git push origin main\n  git push origin v${version}`);

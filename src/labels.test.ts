// Run: npm run test:labels   (Node 24 strips the types; no test framework needed)
import assert from "node:assert/strict";
import { projectLabel, runnerLabel } from "./labels.ts";

const REAL_VITE =
  'node "C:\\Users\\Ray\\Documents\\Projects\\RL Finance\\node_modules\\vite\\bin\\vite.js" dev --port 8080 --strictPort';

assert.equal(projectLabel(REAL_VITE), "RL Finance", "cuts at node_modules to reach the project");
assert.equal(runnerLabel(REAL_VITE), "vite");

// The M3 done-when gate: two node.exe servers must be tellable apart by project.
const A = 'node "C:\\projects\\jayne-dashboard\\node_modules\\next\\dist\\bin\\next" dev';
const B = 'node "C:\\projects\\portal-api\\node_modules\\vite\\bin\\vite.js"';
assert.equal(projectLabel(A), "jayne-dashboard");
assert.equal(projectLabel(B), "portal-api");
assert.notEqual(projectLabel(A), projectLabel(B), "same-named processes must differ by label");
assert.equal(runnerLabel(A), "next");

// Better to show nothing than to invent a label from a system path.
assert.equal(projectLabel("C:\\Windows\\System32\\svchost.exe -k RPCSS"), null);
assert.equal(projectLabel("C:\\Program Files\\nodejs\\node.exe"), null);

// Regression: sysinfo hands back split argv. Rejoined WITHOUT re-quoting, a spaced
// project dir silently truncated ("Beta Analytics" -> "Beta"). Caught on real rows.
assert.equal(
  projectLabel('"C:\\Program Files\\nodejs\\node.exe" "C:\\work\\projects\\Beta Analytics\\server.js" 9102'),
  "Beta Analytics",
  "spaced project dir survives when args are re-quoted",
);
assert.equal(
  projectLabel('"C:\\Program Files\\nodejs\\node.exe" "C:\\work\\projects\\acme-storefront\\server.js" 9101'),
  "acme-storefront",
  "node.exe's own path never wins over the script's project",
);

// Access-denied rows carry no cmd at all; must not throw.
assert.equal(projectLabel(null), null);
assert.equal(runnerLabel(null), null);

// A runner name inside a longer word is not a runner.
assert.equal(runnerLabel("C:\\apps\\nextcloud-client.exe"), null);

console.log("labels: all assertions passed");

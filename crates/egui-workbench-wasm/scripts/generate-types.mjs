#!/usr/bin/env node
// Generate TypeScript declarations for HostSpec from the host JSON Schema.
// Usage: node scripts/generate-types.mjs [--check]
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const output = join(root, "types", "schema.d.ts");
const schemaText = execFileSync(
  "cargo",
  ["run", "--quiet", "-p", "egui-workbench-host", "--bin", "egui-workbench", "--", "--schema"],
  { cwd: join(root, "..", ".."), encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] },
);
const schema = JSON.parse(schemaText);

const primitive = { string: "string", number: "number", integer: "number", boolean: "boolean", null: "null" };

function typeOf(value) {
  if (!value) return "unknown";
  if (value.$ref) return value.$ref.split("/").pop();
  if (value.enum) return value.enum.map((item) => JSON.stringify(item)).join(" | ");
  if (value.oneOf) return value.oneOf.map(typeOf).join(" | ");
  if (value.anyOf) return value.anyOf.map(typeOf).join(" | ");
  if (value.allOf) return value.allOf.map(typeOf).join(" & ");
  if (Array.isArray(value.type)) return value.type.map((type) => primitive[type] ?? "unknown").join(" | ");
  if (value.type === "array") return `Array<${typeOf(value.items)}>`;
  if (value.type === "object" || value.properties) return objectLiteral(value);
  return primitive[value.type] ?? "unknown";
}

function objectLiteral(value, depth = 0) {
  const required = new Set(value.required ?? []);
  const pad = "  ".repeat(depth);
  const inner = "  ".repeat(depth + 1);
  const rows = Object.entries(value.properties ?? {}).map(([name, property]) => {
    const optional = required.has(name) ? "" : "?";
    return `${inner}${JSON.stringify(name)}${optional}: ${typeOf(property)};`;
  });
  return `{\n${rows.join("\n")}\n${pad}}`;
}

function emitDefinition(name) {
  const value = schema.definitions[name];
  if (value.oneOf?.every((variant) => variant.properties)) {
    return `export type ${name} =\n  | ${value.oneOf.map(typeOf).join("\n  | ")};`;
  }
  if (value.enum) return `export type ${name} = ${typeOf(value)};`;
  if (!value.properties) return `export type ${name} = ${typeOf(value)};`;
  return `export interface ${name} ${objectLiteral(value)}`;
}

const names = Object.keys(schema.definitions).sort();
const rootSchema = { ...schema };
delete rootSchema.$schema;
delete rootSchema.definitions;
delete rootSchema.title;
const body = [
  "// Generated from the egui-workbench host JSON Schema. Do not edit by hand.",
  "",
  ...names.map(emitDefinition),
  `export interface HostSpec ${objectLiteral(rootSchema)}`,
  "",
].join("\n\n");

if (process.argv.includes("--check")) {
  if (readFileSync(output, "utf8") !== body) {
    console.error("types/schema.d.ts is out of date; run npm run types");
    process.exit(1);
  }
  console.log("schema types up to date");
} else {
  writeFileSync(output, body);
  console.log(`wrote ${output}`);
}

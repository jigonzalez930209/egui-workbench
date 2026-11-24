import { rmSync } from "node:fs";
rmSync("pkg/package.json", { force: true });
rmSync("pkg/.gitignore", { force: true });
rmSync("pkg/README.md", { force: true });

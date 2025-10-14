#!/usr/bin/env node

import { $, chalk } from "zx";
import prompts from "prompts";
import { command } from "./cli.js";
import { getSizeOfDirectory, listNodeModules } from "./node_modules.js";
import { humanSize } from "./bytes.js";
import Timer from "./timer.js";

$.verbose = false;
const { log } = console;
const { emit, yes } = command;

const startDir = process.cwd();

const searchNodeModulesTimer = Timer.start();
log(chalk.cyan(`🔍 Searching for node_modules under "${startDir}"`));
const nodeModules = await listNodeModules(startDir);

if (nodeModules.length === 0) {
  log(chalk.yellow("No node_modules found."));
  process.exit(0);
}

const nodeModulesWithSize = await Promise.all(
  nodeModules.map(async (path) => {
    const data = await getSizeOfDirectory(path);
    return {
      path,
      ...(data ?? {}),
    };
  })
);

const searchNodeModulesElapsedSeconds = searchNodeModulesTimer.end();

log(
  chalk.gray(
    nodeModulesWithSize
      .sort((a, b) => {
        if (a.bytes === undefined) {
          return -1;
        }

        if (b.bytes === undefined) {
          return 1;
        }

        return a.bytes - b.bytes;
      })
      .map(
        ({ path, size, unit }, i) =>
          `${i + 1}.${unit ? ` ${size}${unit} ` : ""}${path}`
      )
      .join("\n")
  )
);

log(
  chalk.green(
    `📦 ${nodeModules.length} node_modules directories found in ${searchNodeModulesElapsedSeconds}s.`
  )
);

if (emit) {
  process.exit(0);
}

const totalBytes = nodeModulesWithSize.reduce(
  (acc, cur) => (cur.bytes !== undefined ? acc + cur.bytes : acc),
  0
);

const totalSize = humanSize(totalBytes);

const confirmed =
  yes ||
  (
    await prompts({
      type: "confirm",
      name: "delete",
      initial: false,
      message: chalk.red(`Delete ${totalSize} of node_modules?`),
    })
  ).delete;

if (!confirmed) {
  process.exit(0);
}

const nukeNodeModulesTimer = Timer.start();
await Promise.all(nodeModules.map((path) => $`rm -rf ${path}`));
const nukeNodeModulesElapsedSeconds = nukeNodeModulesTimer.end();

log(chalk.green(`🧹 Freed ${totalSize} in ${nukeNodeModulesElapsedSeconds}s!`));

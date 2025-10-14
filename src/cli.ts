import yargs from "yargs";
import { hideBin } from "yargs/helpers";

export const command = yargs(hideBin(process.argv))
  .scriptName("nuke_modules")
  .version("0.0.1")
  .usage(
    "$0 [options]\n\nA CLI to recursively purge all node_modules starting from your current working directory."
  )
  .option("e", {
    alias: "emit",
    type: "boolean",
    default: false,
    describe: "List all directories that would be purged.",
  })
  .option("y", {
    alias: "yes",
    type: "boolean",
    default: false,
    describe: "Auto accept delete confirmation.",
  })
  .help()
  .strict()
  .parseSync();

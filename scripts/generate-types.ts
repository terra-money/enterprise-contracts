import path from "path";
import fs from "fs";
import child_process from "child_process";
import { compileFromFile } from "json-schema-to-typescript";
import { DeclarationIterator } from "./utils/declaration";
import { fileURLToPath } from "url";

type TypeModule = {
  name: string;
  contractDir: string;
  schemaDir: string;
};

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

//const args = process.argv.slice(2);
//const app = args[0];
//const appDir = path.resolve(__dirname, "../apps", app);
const appDir = process.cwd();

const contractsDir = path.resolve(__dirname, "../contracts");
const generateConfig = JSON.parse(
  fs.readFileSync(path.resolve(appDir, "./package.json"), "utf-8")
)["generate-types"];

const typesDir = path.resolve(appDir, generateConfig.output);

const truncateExt = (file: string) => file.split(".")[0];
const removeUnknownMaps = (str: string) =>
  str.split("[k: string]: unknown;").join("");
const withModule = (name: string, str: string) =>
  `export module ${name} { ${str} }`;

const dedupTypes = async (str: string) => {
  const iter = new DeclarationIterator(str);
  const visited = new Set();
  let result = "";

  while (iter.hasNext()) {
    const decl = await iter.next();

    if (!visited.has(decl.name)) {
      visited.add(decl.name);
      result = `${result}\n${decl.content}`;
    }
  }

  return result;
};

const writeModuleTypes = async (module: TypeModule) => {
  const files = fs.readdirSync(module.schemaDir);
  const types = await files.map(truncateExt).reduce(async (acc, file) => {
    const str = await compileFromFile(
      path.join(module.schemaDir, `/${file}.json`)
    );

    const ts = removeUnknownMaps(str);

    return Promise.resolve(`${await acc}\n${ts}`);
  }, Promise.resolve(""));

  const str = withModule(module.name, await dedupTypes(types));

  fs.writeFileSync(path.join(typesDir, `/${module.name}.ts`), str);
};

const writeIndexTs = (modules: TypeModule[]) => {
  const indexTsContent = modules.reduce((acc, module) => {
    return `${acc}\nexport * from './${module.name}'`;
  }, "");

  fs.writeFileSync(path.join(typesDir, "/index.ts"), indexTsContent);
};

const formatTypesDir = () => {
  setTimeout(() => {
    child_process.exec(`prettier --write ${typesDir}`);
  }, 500);
};

const readModules = () => {
  return fs
    .readdirSync(contractsDir)
    .filter((contract) =>
      generateConfig.contracts.some((target: any) => target === contract)
    )
    .map((contract) => ({
      name: contract.replaceAll("-", "_"),
      contractDir: path.join(contractsDir, contract),
      schemaDir: path.join(contractsDir, contract, "/schema"),
    }));
};

const clearTypesDir = () => {
  try {
    fs.rmSync(typesDir, { recursive: true });
  } catch {
    // consume error if folder doesn't exist
  }

  fs.mkdirSync(typesDir, { recursive: true });
};

const generateSchemas = (modules: TypeModule[]) => {
  modules.forEach((module: TypeModule) => {
    child_process.execSync("cargo schema", { cwd: module.contractDir });
  });
};

const main = () => {
  clearTypesDir();
  const modules = readModules();
  console.log(modules);

  generateSchemas(modules);
  modules.forEach(writeModuleTypes);
  writeIndexTs(modules);
  formatTypesDir();
};

main();

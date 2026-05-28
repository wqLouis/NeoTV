const internationalization = {
  "locales": [
    "en",
    "zh-Hans"
  ],
  "requiredLocales": [
    "en",
    "zh-Hans"
  ],
  "strictMode": "inclusive",
  "defaultLocale": "en"
};
const routing = {
  "mode": "prefix-no-default",
  "storage": {
    "cookies": [
      {
        "name": "INTLAYER_LOCALE",
        "attributes": {}
      }
    ],
    "headers": [
      {
        "name": "x-intlayer-locale"
      }
    ]
  },
  "basePath": ""
};
const editor = {
  "editorURL": "http://localhost:8000",
  "cmsURL": "https://app.intlayer.org",
  "backendURL": "https://back.intlayer.org",
  "port": 8000,
  "enabled": false,
  "dictionaryPriorityStrategy": "local_first",
  "liveSync": false,
  "liveSyncPort": 4000,
  "liveSyncURL": "http://localhost:4000"
};
const log = {
  "mode": "default",
  "prefix": "\u001b[38;5;239m[intlayer] \u001b[0m"
};
const system = {
  "baseDir": "/mnt/DATA/Document/code/LibreTV-App",
  "moduleAugmentationDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/types",
  "unmergedDictionariesDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/unmerged_dictionary",
  "remoteDictionariesDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/remote_dictionary",
  "dictionariesDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/dictionary",
  "dynamicDictionariesDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/dynamic_dictionary",
  "fetchDictionariesDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/fetch_dictionary",
  "typesDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/types",
  "mainDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/main",
  "configDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/config",
  "cacheDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/cache",
  "tempDir": "/mnt/DATA/Document/code/LibreTV-App/.intlayer/tmp"
};
const content = {
  "fileExtensions": [
    ".content.ts",
    ".content.js",
    ".content.cjs",
    ".content.mjs",
    ".content.json",
    ".content.json5",
    ".content.jsonc",
    ".content.tsx",
    ".content.jsx",
    ".content.md",
    ".content.mdx",
    ".content.yaml",
    ".content.yml"
  ],
  "contentDir": [
    "/mnt/DATA/Document/code/LibreTV-App"
  ],
  "codeDir": [
    "/mnt/DATA/Document/code/LibreTV-App"
  ],
  "excludedPath": [
    "**/node_modules/**",
    "**/dist/**",
    "**/build/**",
    "**/.intlayer/**",
    "**/.next/**",
    "**/.nuxt/**",
    "**/.expo/**",
    "**/.vercel/**",
    "**/.turbo/**",
    "**/.tanstack/**"
  ],
  "watch": true
};
const ai = {};
const dictionary = {
  "fill": true,
  "contentAutoTransformation": false,
  "location": "local",
  "importMode": "static"
};
const build = {
  "mode": "auto",
  "minify": false,
  "purge": false,
  "traversePattern": [
    "**/*.{tsx,ts,js,mjs,cjs,jsx,vue,svelte,astro}",
    "!**/node_modules/**",
    "!**/dist/**",
    "!**/build/**",
    "!**/.intlayer/**",
    "!**/.next/**",
    "!**/.nuxt/**",
    "!**/.expo/**",
    "!**/.vercel/**",
    "!**/.turbo/**",
    "!**/.tanstack/**",
    "!**/*.config.*",
    "!**/*.test.*",
    "!**/*.spec.*",
    "!**/*.stories.*",
    "!**/*.d.ts",
    "!**/*.d.ts.map",
    "!**/*.map"
  ],
  "outputFormat": [
    "esm",
    "cjs"
  ],
  "cache": true,
  "checkTypes": false
};
const compiler = {
  "enabled": true,
  "dictionaryKeyPrefix": "",
  "noMetadata": false,
  "saveComponents": false
};
const configuration = { internationalization, routing, editor, log, system, content, ai, dictionary, build, compiler };

module.exports.internationalization = internationalization;
module.exports.routing = routing;
module.exports.editor = editor;
module.exports.log = log;
module.exports.system = system;
module.exports.content = content;
module.exports.ai = ai;
module.exports.dictionary = dictionary;
module.exports.build = build;
module.exports.compiler = compiler;
module.exports = configuration;

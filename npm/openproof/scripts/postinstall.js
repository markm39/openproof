const os = require("os");

const PLATFORMS = {
  darwin: { arm64: "@openproof/cli-darwin-arm64", x64: "@openproof/cli-darwin-x64" },
  linux: { arm64: "@openproof/cli-linux-arm64", x64: "@openproof/cli-linux-x64" },
};

const platform = os.platform();
const arch = os.arch();
const pkg = PLATFORMS[platform]?.[arch];

if (!pkg) {
  console.warn(
    `openproof: no prebuilt binary for ${platform}-${arch}\n` +
      `Install from source: cargo install --path crates/openproof-cli`
  );
  process.exit(0);
}

try {
  require.resolve(`${pkg}/package.json`);
} catch {
  console.warn(
    `openproof: platform package ${pkg} not found.\n` +
      `This may happen on unsupported platforms or with certain package managers.\n` +
      `Install from source: cargo install --path crates/openproof-cli`
  );
}

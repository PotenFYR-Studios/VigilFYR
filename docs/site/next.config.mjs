/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",
  basePath: process.env.DOCS_BASE_PATH || "",
  images: { unoptimized: true },
};

export default nextConfig;

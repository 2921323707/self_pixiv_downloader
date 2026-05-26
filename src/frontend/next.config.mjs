const backendUrl = process.env.PIXIV_BACKEND_URL || "http://127.0.0.1:3000";
const staticExport = process.env.NEXT_OUTPUT_EXPORT === "1";

/** @type {import('next').NextConfig} */
const nextConfig = {
  ...(staticExport
    ? {
        output: "export",
        trailingSlash: true
      }
    : {
        async rewrites() {
          return [
            {
              source: "/api/:path*",
              destination: `${backendUrl}/api/:path*`
            }
          ];
        }
      })
};

export default nextConfig;

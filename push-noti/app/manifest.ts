import type { MetadataRoute } from "next";

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "push-noti",
    short_name: "push-noti",
    start_url: "/",
    display: "standalone", // iOS only allows web push for home-screen (standalone) apps
    background_color: "#1a1033",
    theme_color: "#1a1033",
    icons: [{ src: "/icon.svg", sizes: "any", type: "image/svg+xml" }],
  };
}

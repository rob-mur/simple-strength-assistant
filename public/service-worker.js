self.addEventListener("install", (e) => {
  console.log("[SW] Installing...");
  // Force immediate activation
  self.skipWaiting();
});

self.addEventListener("activate", (e) => {
  console.log("[SW] Activating...");
  // Take control of all pages immediately
  e.waitUntil(clients.claim());
});

self.addEventListener("fetch", (e) => {
  // Pass through to network (minimal implementation)
  e.respondWith(fetch(e.request));
});

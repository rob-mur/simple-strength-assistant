// Simple service worker for PWA installation support
const CACHE_NAME = 'strength-assistant-v1';

self.addEventListener('install', (event) => {
  console.log('[ServiceWorker] Install');
  // Skip waiting to activate immediately
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  console.log('[ServiceWorker] Activate');
  // Claim all clients immediately
  event.waitUntil(self.clients.claim());
});

// Basic fetch handler - just pass through to network
self.addEventListener('fetch', (event) => {
  // For now, just pass through all requests to the network
  // This minimal service worker is mainly to enable PWA installation
  event.respondWith(fetch(event.request));
});

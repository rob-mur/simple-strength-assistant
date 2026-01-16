const CACHE_NAME = 'strength-assistant-v1';
const CACHE_MAX_AGE = 30 * 24 * 60 * 60 * 1000; // 30 days
const CACHE_MAX_ITEMS = 100;

// Files to cache immediately on install
const CRITICAL_CACHE = [
  '/',
  '/index.html',
  '/manifest.json',
  '/icons/icon-192x192.png',
  '/icons/icon-512x512.png'
];

// File extensions to cache dynamically
const CACHEABLE_EXTENSIONS = ['.wasm', '.js', '.css', '.png', '.jpg', '.jpeg', '.svg', '.webp'];

// Check if a URL should be cached
function shouldCache(url) {
  try {
    const urlObj = new URL(url);
    // Only cache same-origin requests
    if (urlObj.origin !== self.location.origin) {
      return false;
    }
    // Check if file extension is cacheable
    return CACHEABLE_EXTENSIONS.some(ext => urlObj.pathname.endsWith(ext));
  } catch (e) {
    return false;
  }
}

// Limit cache size
async function limitCacheSize(cacheName, maxItems) {
  const cache = await caches.open(cacheName);
  const keys = await cache.keys();
  if (keys.length > maxItems) {
    // Remove oldest entries
    await cache.delete(keys[0]);
    await limitCacheSize(cacheName, maxItems);
  }
}

// Install event - cache critical assets and discover WASM/JS bundles
self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);

      // Cache critical files
      await cache.addAll(CRITICAL_CACHE);

      // Discover and cache WASM/JS bundles from index.html
      try {
        const indexResponse = await fetch('/index.html');
        const indexText = await indexResponse.text();

        // Extract WASM and JS file references
        const wasmMatch = indexText.match(/['"](\/[^'"]*\.wasm)['"]/);
        const jsMatches = indexText.matchAll(/['"](\/[^'"]*\.js)['"]/g);

        const bundlesToCache = [];
        if (wasmMatch) bundlesToCache.push(wasmMatch[1]);
        for (const match of jsMatches) {
          bundlesToCache.push(match[1]);
        }

        // Cache discovered bundles
        if (bundlesToCache.length > 0) {
          await cache.addAll(bundlesToCache);
          console.log('Cached bundles:', bundlesToCache);
        }
      } catch (e) {
        console.warn('Failed to discover/cache bundles:', e);
      }

      // Take control immediately
      await self.skipWaiting();
    })()
  );
});

// Fetch event - cache-first with network fallback and error handling
self.addEventListener('fetch', (event) => {
  event.respondWith(
    (async () => {
      try {
        // Try cache first
        const cachedResponse = await caches.match(event.request);
        if (cachedResponse) {
          return cachedResponse;
        }

        // Fetch from network
        const networkResponse = await fetch(event.request);

        // Check if we should cache this response
        if (
          networkResponse &&
          networkResponse.status === 200 &&
          (networkResponse.type === 'basic' || networkResponse.type === 'cors') &&
          shouldCache(event.request.url)
        ) {
          // Clone and cache the response
          const cache = await caches.open(CACHE_NAME);
          cache.put(event.request, networkResponse.clone());

          // Limit cache size (non-blocking)
          limitCacheSize(CACHE_NAME, CACHE_MAX_ITEMS);
        }

        return networkResponse;
      } catch (error) {
        console.error('Fetch failed:', error);

        // Try to return cached version as fallback
        const cachedResponse = await caches.match(event.request);
        if (cachedResponse) {
          return cachedResponse;
        }

        // If it's a navigation request, return the cached index
        if (event.request.mode === 'navigate') {
          const indexCache = await caches.match('/index.html');
          if (indexCache) {
            return indexCache;
          }
        }

        // Return a basic error response
        return new Response('Offline and resource not cached', {
          status: 503,
          statusText: 'Service Unavailable',
          headers: new Headers({
            'Content-Type': 'text/plain'
          })
        });
      }
    })()
  );
});

// Activate event - clean up old caches and claim clients
self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      // Clean up old caches
      const cacheNames = await caches.keys();
      await Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {
            console.log('Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );

      // Take control of all clients immediately
      await self.clients.claim();
    })()
  );
});

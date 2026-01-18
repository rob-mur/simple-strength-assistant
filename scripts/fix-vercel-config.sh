#!/usr/bin/env bash
set -e

echo "Fixing dist/public/vercel.json for deployment..."

# Copy the source vercel.json to dist/public, ensuring correct values
cp public/vercel.json dist/public/vercel.json

echo "✓ vercel.json fixed with correct buildCommand and installCommand"

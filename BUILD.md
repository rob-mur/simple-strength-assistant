# Build Instructions

## Prerequisites

- Rust and Cargo
- Dioxus CLI (`dx`)
- Node.js and npm

## Building the Application

### CSS Build

The project uses Tailwind CSS with DaisyUI for styling. Before building the application, you need to build the CSS:

```bash
npm install
npm run build:css
```

This will process `src/styles.css` and output the compiled CSS to `public/styles.css`.

### Development

For development with hot reload:

```bash
# In one terminal, watch CSS changes
npm run watch:css

# In another terminal, run the Dioxus dev server
dx serve
```

### Production Build

For a production build:

```bash
# Build CSS
npm run build:css

# Build the application
dx build --release
```

The built files will be in `target/dx/simple-strength-assistant/release/web/public/`.

## Project Structure

- `src/styles.css` - Tailwind CSS source file with directives
- `public/styles.css` - Compiled CSS output (generated, not committed)
- `tailwind.config.js` - Tailwind CSS configuration with DaisyUI
- `postcss.config.js` - PostCSS configuration for Tailwind processing
- `index.html` - Main HTML template
- `src/app.rs` - Main application component

## Notes

- The `public/styles.css` file is generated during the build process and should not be committed to version control.
- DaisyUI provides pre-built components that can be used with simple class names (e.g., `btn btn-primary`, `card`, `card-body`).
- Tailwind utility classes can be used directly in component class attributes.

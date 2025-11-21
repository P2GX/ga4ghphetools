name: Docs

on:
  push:
    branches: [ main ]

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          override: true

      - name: Install mdBook
        run: cargo install mdbook --version 0.4.40 --locked

      - name: Build API docs
        run: cargo doc --no-deps --release

      # 1. Build mdBook: The output is usually `./book/`.
      - name: Build mdBook
        run: mdbook build book

      # 2. Copy API docs into the mdBook output directory.
      # mdBook's output is *inside* the 'book' directory you ran the command on.
      # If your project structure is:
      # project/
      # ├── book/ <--- this is your source for mdbook
      # └── target/
      #
      # Then 'mdbook build book' creates:
      # project/
      # ├── book/
      # │   └── book/ <--- This is the final static output directory
      # └── target/
      #
      # We put the API docs inside the final static output directory.
      - name: Copy API docs into mdBook output
        run: |
          # The static output directory is 'book/book/'
          mkdir -p book/book/api
          # Copy the generated documentation (target/doc) into the new api directory
          cp -r target/doc book/book/api/doc

      # 3. Upload the *final static output* directory.
      # The directory 'book/book' contains your index.html and all assets.
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          # This should point to the directory containing index.html.
          # Based on your setup, this is likely 'book/book'.
          path: ./book/book

  # The 'deploy' job is correct. It uses the artifact uploaded in 'build'.
  deploy:
    runs-on: ubuntu-latest
    needs: build
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
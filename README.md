# View-rs

Image viewer, powered by Rust+egui.

## How to use

1. Click `Open directory` button to open a directory
2. Select a directory that contains some image files
3. Optionally enter part of an image filename and click `Search`
4. Click left/right button to view the image

## Features

- Opens a directory and lists all image files in it
- Randomize the file order
- Searches image filenames by case-insensitive partial match
- Shows an image, keeps aspect ratio
- Changes the size of the image following window size
- Supports following image format:
  - JPEG
  - PNG
  - BMP
  - GIF (static — only the first frame is shown; animations are not played)
  - WebP
  - AVIF

Notes:
- This application currently supports static images only. Animated formats (animated GIF, animated WebP/AVIF) are not played; the viewer shows a single static frame (typically the first frame).

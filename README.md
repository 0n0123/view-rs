# View-rs

Image viewer, powered by Rust+egui.

## How to use

1. Click `Open directory` button to open a directory
2. Select a directory that contains some image files
3. Click left/right button to view the image

## Features

- Reads all the image files in the directory
- Randomize the file order
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

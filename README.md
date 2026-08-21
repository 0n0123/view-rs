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
  - GIF (animated GIFs are played automatically)
  - WebP (animated WebPs are played automatically)
  - AVIF

Notes:
- Animated GIF and animated WebP files are played automatically. Playback controls such as pause, seek, and speed adjustment are not provided.
- APNG and animated AVIF are not currently played.

## How to build

To build an executable, run the following command:

```bash
cargo build --release
```

For macOS, you can also build a `.app` bundle by running the following command:

```bash
cargo bundle --release
```
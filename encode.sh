#!/bin/sh
set -eu

if [ $# -ne 1 ]; then
    echo "usage: $0 output.{mp4,gif}" >&2
    exit 1
fi

RESOLUTION=${RESOLUTION:-1280x720}
FPS=${FPS:-30}
GOP=$((FPS * 2))

case "$1" in
    *.gif)
        ffmpeg -hide_banner \
          -f rawvideo -pixel_format rgb32 -framerate "$FPS" -video_size "$RESOLUTION" -i - \
          -vf "fps=$FPS,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" \
          "$1"
        ;;
    *)
        ffmpeg -hide_banner \
          -f rawvideo -pixel_format rgb32 -framerate "$FPS" -video_size "$RESOLUTION" -i - \
          -an \
          -c:v libx264 -preset slow -profile:v high -level 4.1 \
          -pix_fmt yuv420p \
          -r "$FPS" -g "$GOP" \
          -b:v 8M -maxrate 8M -bufsize 16M \
          -movflags +faststart \
          -color_primaries bt709 -color_trc bt709 -colorspace bt709 \
          "$1"
        ;;
esac
